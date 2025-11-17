use anyhow::Context;
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    handlers::{
        email_verification_handler::create_verification_token,
        mfa_handler::{generate_mfa_challenge_token, MfaRequiredResponse},
    },
    state::AppState,
};
use core_services::security::{hash_password, validate_password_strength, verify_password};
use domain::models::{
    Claims, ForgotPasswordPayload, LoginPayload, LoginResponse, RefreshTokenPayload,
    RegisterPayload, ResetPasswordPayload, TokenType,
};
use persistence::repositories::{
    auth_repository::AuthRepository, mfa_repository::MfaRepository, user_repository::UserRepository,
};

const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600; // 1 hora
const REFRESH_TOKEN_EXPIRY_SECONDS: i64 = 604800; // 7 dias
const PASSWORD_RESET_EXPIRY_SECONDS: i64 = 900; // 15 minutos

/// POST /login
/// UPDATED: Now checks for MFA and returns MFA challenge if enabled
pub async fn handler_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;

    // 1. Find user by username or email
    let user: (Uuid, String, bool) = sqlx::query_as(
        "SELECT id, password_hash, mfa_enabled FROM users WHERE username = $1 OR LOWER(email) = LOWER($1)",
    )
    .bind(&payload.username)
    .fetch_optional(&state.db_pool_auth)
    .await?
    .ok_or(AppError::InvalidPassword)?;

    let user_id = user.0;
    let password_hash = user.1;
    let mfa_enabled = user.2;

    // 2. Verify password with Argon2id
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .context("Falha task verificar senha")?
            .map_err(|_| AppError::InvalidPassword)?;

    if !password_valid {
        return Err(AppError::InvalidPassword);
    }

    // 3. Check if MFA is enabled
    if mfa_enabled {
        // Generate MFA challenge token
        let mfa_token =
            generate_mfa_challenge_token(&state, user_id).context("Falha ao gerar token MFA")?;

        tracing::info!(user_id = %user_id, event_type = "mfa_challenge_issued", "MFA challenge emitido");

        return Ok(Json(serde_json::json!({
            "mfa_required": true,
            "mfa_token": mfa_token,
            "message": "Autenticação de dois fatores necessária"
        })));
    }

    // 4. Generate tokens (no MFA)
    let (access_token, refresh_token_raw) = generate_tokens(&state, user_id).await?;

    tracing::info!(user_id = %user_id, event_type = "user_login", "Usuário autenticado");

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token_raw,
        "token_type": "Bearer",
        "expires_in": ACCESS_TOKEN_EXPIRY_SECONDS
    })))
}

/// POST /register
/// UPDATED: Now sends email verification
pub async fn handler_register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    if !payload
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest("Username inválido".to_string()));
    }

    validate_password_strength(&payload.password).map_err(AppError::BadRequest)?;

    let user_repo = UserRepository::new(&state.db_pool_auth);

    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Conflict("Username já está em uso".to_string()));
    }
    if user_repo.exists_by_email(&payload.email).await? {
        return Err(AppError::Conflict("Email já está em uso".to_string()));
    }

    let password_clone = payload.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha task hash")?
        .context("Erro ao gerar hash")?;

    let user = user_repo
        .create(&payload.username, &payload.email, &password_hash)
        .await?;

    // Generate tokens
    let (access_token, refresh_token_raw) = generate_tokens(&state, user.id).await?;

    // NEW: Generate and send email verification token
    let verification_token = create_verification_token(&state, user.id)
        .await
        .context("Falha ao criar token de verificação")?;

    // Send verification email (instead of just welcome email)
    state.email_service.send_verification_email(
        payload.email.clone(),
        &user.username,
        &verification_token,
    );

    // Also send welcome email
    state
        .email_service
        .send_welcome_email(payload.email, &user.username);

    tracing::info!(user_id = %user.id, event_type = "user_registered", "Novo usuário registrado (email não verificado)");

    Ok(Json(LoginResponse::new(
        access_token,
        refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

// Struct para o retorno do `query_as`
#[derive(sqlx::FromRow)]
struct RefreshTokenInfo {
    id: Uuid,
    user_id: Uuid,
    revoked: bool,
    expires_at: chrono::DateTime<chrono::Utc>,
    family_id: Uuid,
}

/// POST /refresh-token
pub async fn handler_refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    let old_token_hash = hash_token(&payload.refresh_token);

    // 1. Start transaction
    let mut tx = state
        .db_pool_auth
        .begin()
        .await
        .context("Falha ao iniciar transação de refresh")?;

    let old_token = sqlx::query_as::<_, RefreshTokenInfo>(
        r#"
        SELECT id, user_id, revoked, expires_at, family_id
        FROM refresh_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(&old_token_hash)
    .fetch_optional(&mut *tx)
    .await
    .context("Falha ao buscar refresh token")?;

    let (user_id, family_id) = match old_token {
        None => {
            return Err(AppError::Unauthorized("Refresh token inválido".to_string()));
        }
        Some(token) => {
            if token.revoked {
                // Token theft detection
                sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE family_id = $1")
                    .bind(token.family_id)
                    .execute(&mut *tx)
                    .await
                    .context("Falha ao revogar família de tokens")?;

                tx.commit()
                    .await
                    .context("Falha ao commitar revogação de família")?;

                tracing::warn!(
                    user_id = %token.user_id,
                    family_id = %token.family_id,
                    event_type = "token_theft_detected",
                    "Reuso de refresh token detectado. Família de tokens revogada."
                );

                return Err(AppError::Unauthorized(
                    "Sessão invalidada por segurança".to_string(),
                ));
            }

            if token.expires_at <= Utc::now() {
                return Err(AppError::Unauthorized("Refresh token expirado".to_string()));
            }

            sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE id = $1")
                .bind(token.id)
                .execute(&mut *tx)
                .await
                .context("Falha ao revogar token antigo")?;

            (token.user_id, token.family_id)
        }
    };

    // 3. Generate new Access Token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
        token_type: TokenType::Access,
    };

    let header = Header::new(Algorithm::EdDSA);
    let access_token =
        encode(&header, &claims, &state.encoding_key).context("Falha ao codificar JWT")?;

    // 4. Generate and save new Refresh Token
    let new_refresh_token_raw = Uuid::new_v4().to_string();
    let new_refresh_token_hash = hash_token(&new_refresh_token_raw);

    let new_expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY_SECONDS);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id, parent_token_hash)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind(new_refresh_token_hash)
    .bind(new_expires_at)
    .bind(family_id)
    .bind(&old_token_hash)
    .execute(&mut *tx)
    .await
    .context("Falha ao salvar novo refresh token")?;

    // 5. Commit transaction
    tx.commit()
        .await
        .context("Falha ao commitar rotação de token")?;

    tracing::info!(user_id = %user_id, event_type = "token_refresh_rotated", "Token rotacionado com sucesso");

    Ok(Json(LoginResponse::new(
        access_token,
        new_refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /logout
pub async fn handler_logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;
    let refresh_token_hash = hash_token(&payload.refresh_token);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    if !auth_repo.revoke_refresh_token(&refresh_token_hash).await? {
        return Err(AppError::NotFound(
            "Token não encontrado ou já revogado".to_string(),
        ));
    }

    tracing::info!(event_type = "user_logout", "Refresh token revogado");

    Ok(Json(serde_json::json!({"message": "Logout realizado"})))
}

/// POST /forgot-password
pub async fn handler_forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;
    let user_repo = UserRepository::new(&state.db_pool_auth);

    let user = user_repo.find_by_email(&payload.email).await?;

    if let Some(user) = user {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let claims = Claims {
            sub: user.id,
            exp: now + PASSWORD_RESET_EXPIRY_SECONDS,
            iat: now,
            token_type: TokenType::PasswordReset,
        };

        let header = Header::new(Algorithm::EdDSA);
        let token =
            encode(&header, &claims, &state.encoding_key).context("Falha ao gerar JWT de reset")?;

        state
            .email_service
            .send_password_reset_email(payload.email, &user.username, &token);
    } else {
        tracing::info!(email = %payload.email, event_type = "forgot_password_attempt", "Tentativa de reset para email não existente");
    }

    Ok(Json(serde_json::json!({
        "message": "Se este email estiver registado, um link de redefinição de senha foi enviado."
    })))
}

/// POST /reset-password
pub async fn handler_reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;

    validate_password_strength(&payload.new_password).map_err(AppError::BadRequest)?;

    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = 5;

    let token_data =
        decode::<Claims>(&payload.token, &state.decoding_key, &validation).map_err(|e| {
            tracing::warn!(error = %e, "Falha na validação do token de reset");
            AppError::Unauthorized("Token inválido ou expirado".to_string())
        })?;

    let claims = token_data.claims;

    if claims.token_type != TokenType::PasswordReset {
        return Err(AppError::Unauthorized(
            "Token inválido (tipo incorreto)".to_string(),
        ));
    }

    let user_id = claims.sub;

    let password_clone = payload.new_password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha task hash (reset)")?
        .context("Erro ao gerar hash (reset)")?;

    let user_repo = UserRepository::new(&state.db_pool_auth);
    user_repo
        .update_password(user_id, &password_hash)
        .await
        .context("Falha ao atualizar senha no DB")?;

    let auth_repo = AuthRepository::new(&state.db_pool_auth);
    auth_repo
        .revoke_all_user_tokens(user_id)
        .await
        .context("Falha ao revogar refresh tokens após reset")?;

    tracing::info!(user_id = %user_id, event_type = "password_reset_success", "Senha redefinida com sucesso");

    Ok(Json(
        serde_json::json!({"message": "Senha atualizada com sucesso"}),
    ))
}

// --- Helpers ---

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn generate_tokens(state: &AppState, user_id: Uuid) -> Result<(String, String), AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 1. Generate Access Token
    let claims = Claims {
        sub: user_id,
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
        token_type: TokenType::Access,
    };

    let header = Header::new(Algorithm::EdDSA);
    let access_token =
        encode(&header, &claims, &state.encoding_key).context("Falha ao gerar JWT")?;

    // 2. Generate Refresh Token
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);
    let family_id = Uuid::new_v4();

    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY_SECONDS);

    // 3. Save Refresh Token
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id, parent_token_hash)
        VALUES ($1, $2, $3, $4, NULL)
        "#,
    )
    .bind(user_id)
    .bind(&refresh_token_hash)
    .bind(expires_at)
    .bind(family_id)
    .execute(&state.db_pool_auth)
    .await
    .context("Falha ao salvar refresh token inicial")?;

    Ok((access_token, refresh_token_raw))
}
