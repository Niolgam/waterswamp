use anyhow::Context;
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, Header};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

use crate::{error::AppError, state::AppState};
use core_services::security::{hash_password, validate_password_strength, verify_password};
use domain::models::{
    Claims, LoginPayload, LoginResponse, RefreshTokenPayload, RegisterPayload, TokenType,
};
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};

const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600; // 1 hora
const REFRESH_TOKEN_EXPIRY_SECONDS: i64 = 604800; // 7 dias

/// POST /login
pub async fn handler_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    let user: (Uuid, String) =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1")
            .bind(&payload.username)
            .fetch_optional(&state.db_pool_auth)
            .await?
            .ok_or(AppError::InvalidPassword)?;

    // 2. Verificar senha com Argon2id
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &user.1))
            .await
            .context("Falha task verificar senha")?
            .map_err(|_| AppError::InvalidPassword)?;

    if !password_valid {
        return Err(AppError::InvalidPassword);
    }

    // 3. Gerar tokens (agora usa o helper modificado)
    let (access_token, refresh_token_raw) = generate_tokens(&state, user.0).await?;

    tracing::info!(user_id = %user.0, event_type = "user_login", "Usuário autenticado");

    Ok(Json(LoginResponse::new(
        access_token,
        refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /register
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

    let password_clone = payload.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha task hash")?
        .context("Erro ao gerar hash")?;

    let user = user_repo.create(&payload.username, &password_hash).await?;

    // Gerar tokens (agora usa o helper modificado)
    let (access_token, refresh_token_raw) = generate_tokens(&state, user.id).await?;

    tracing::info!(user_id = %user.id, event_type = "user_registered", "Novo usuário registrado");

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
/// ATUALIZADO: Implementa Rotação de Token (3.2) e Detecção de Roubo (3.3)
pub async fn handler_refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    let old_token_hash = hash_token(&payload.refresh_token);

    // 1. Iniciar transação
    let mut tx = state
        .db_pool_auth
        .begin()
        .await
        .context("Falha ao iniciar transação de refresh")?;

    // --- CORREÇÃO: Ligar com referência `&old_token_hash` ---
    let old_token = sqlx::query_as::<_, RefreshTokenInfo>(
        r#"
        SELECT id, user_id, revoked, expires_at, family_id
        FROM refresh_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(&old_token_hash) // <-- MUDANÇA AQUI
    .fetch_optional(&mut *tx)
    .await
    .context("Falha ao buscar refresh token")?;

    let (user_id, family_id) = match old_token {
        None => {
            return Err(AppError::Unauthorized("Refresh token inválido".to_string()));
        }
        Some(token) => {
            if token.revoked {
                // --- DETECÇÃO DE ROUBO! (Subtarefa 3.3) ---
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

    // 3. Gerar novo Access Token (lógica existente)
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

    // 4. Gerar e salvar novo Refresh Token (Subtarefa 3.2)
    let new_refresh_token_raw = Uuid::new_v4().to_string();
    let new_refresh_token_hash = hash_token(&new_refresh_token_raw);

    let new_expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY_SECONDS);

    // --- CORREÇÃO: Ligar com referência `&old_token_hash` ---
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id, parent_token_hash)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind(new_refresh_token_hash) // new_refresh_token_hash é movido aqui (OK, só usa 1 vez)
    .bind(new_expires_at)
    .bind(family_id)
    .bind(&old_token_hash) // <-- MUDANÇA AQUI
    .execute(&mut *tx)
    .await
    .context("Falha ao salvar novo refresh token")?;

    // 5. Commitar transação
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

    // 1. Gerar Access Token
    let claims = Claims {
        sub: user_id,
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
        token_type: TokenType::Access,
    };

    let header = Header::new(Algorithm::EdDSA);
    let access_token =
        encode(&header, &claims, &state.encoding_key).context("Falha ao gerar JWT")?;

    // 2. Gerar Refresh Token (Início de uma nova família)
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);
    let family_id = Uuid::new_v4();

    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY_SECONDS);

    // 3. Salvar Refresh Token no DB
    // --- CORREÇÃO: Ligar com referência (por consistência) ---
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id, parent_token_hash)
        VALUES ($1, $2, $3, $4, NULL)
        "#,
    )
    .bind(user_id)
    .bind(&refresh_token_hash) // <-- MUDANÇA AQUI (para &str)
    .bind(expires_at)
    .bind(family_id)
    .execute(&state.db_pool_auth)
    .await
    .context("Falha ao salvar refresh token inicial")?;

    Ok((access_token, refresh_token_raw))
}
