//! Auth API Handlers
//!
//! Handlers para autenticação: login, registro, refresh token, logout,
//! forgot password e reset password.

use anyhow::Context;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

use core_services::security::{hash_password, validate_password_strength, verify_password};
use domain::models::TokenType;
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    ForgotPasswordRequest, ForgotPasswordResponse, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, MfaRequiredResponse, RefreshTokenRequest, RefreshTokenResponse,
    RegisterRequest, RegisterResponse, ResetPasswordRequest, ResetPasswordResponse,
};

// =============================================================================
// CONSTANTS
// =============================================================================

const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600; // 1 hora
const REFRESH_TOKEN_EXPIRY_SECONDS: i64 = 604800; // 7 dias
const PASSWORD_RESET_EXPIRY_SECONDS: i64 = 900; // 15 minutos
const MFA_CHALLENGE_EXPIRY_SECONDS: i64 = 300; // 5 minutos

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Gera hash SHA-256 de um token
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Gera par de tokens (access + refresh) para um usuário
async fn generate_tokens(state: &AppState, user_id: Uuid) -> Result<(String, String), AppError> {
    // 1. Gerar Access Token (JWT)
    let access_token = state
        .jwt_service
        .generate_token(user_id, TokenType::Access, ACCESS_TOKEN_EXPIRY_SECONDS)
        .map_err(|e| {
            error!("Erro ao gerar access token: {:?}", e);
            AppError::Anyhow(e)
        })?;

    // 2. Gerar Refresh Token (UUID opaco)
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);
    let family_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY_SECONDS);

    // 3. Salvar Refresh Token no banco
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
    .context("Falha ao salvar refresh token")?;

    Ok((access_token, refresh_token_raw))
}

/// Gera token de desafio MFA
fn generate_mfa_challenge_token(state: &AppState, user_id: Uuid) -> anyhow::Result<String> {
    state
        .jwt_service
        .generate_mfa_token(user_id, MFA_CHALLENGE_EXPIRY_SECONDS)
}

/// Cria token de verificação de email
async fn create_verification_token(state: &AppState, user_id: Uuid) -> anyhow::Result<String> {
    use persistence::repositories::email_verification_repository::EmailVerificationRepository;

    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);
    let verification_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&verification_token);

    verification_repo
        .save_verification_token(user_id, &token_hash, 24) // 24 horas
        .await
        .context("Falha ao salvar token de verificação")?;

    Ok(verification_token)
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /login
///
/// Autentica um usuário e retorna tokens JWT.
/// Se MFA estiver habilitado, retorna um token de desafio MFA.
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Validação de login falhou");
        AppError::Validation(e)
    })?;

    // 2. Buscar usuário por username ou email
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

    // 3. Verificar senha com Argon2id
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .context("Falha na task de verificação de senha")?
            .map_err(|_| AppError::InvalidPassword)?;

    if !password_valid {
        warn!(username = %payload.username, "Tentativa de login com senha inválida");
        return Err(AppError::InvalidPassword);
    }

    // 4. Verificar se MFA está habilitado
    if mfa_enabled {
        let mfa_token =
            generate_mfa_challenge_token(&state, user_id).context("Falha ao gerar token MFA")?;

        info!(user_id = %user_id, "MFA challenge emitido");

        return Ok(Json(serde_json::json!(MfaRequiredResponse::new(mfa_token))));
    }

    // 5. Gerar tokens (sem MFA)
    let (access_token, refresh_token) = generate_tokens(&state, user_id).await?;

    info!(user_id = %user_id, "Login realizado com sucesso");

    Ok(Json(serde_json::json!(LoginResponse::new(
        access_token,
        refresh_token,
        ACCESS_TOKEN_EXPIRY_SECONDS
    ))))
}

/// POST /register
///
/// Registra um novo usuário e envia email de verificação.
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Validação de registro falhou");
        AppError::Validation(e)
    })?;

    // 2. Validar caracteres do username
    if !payload
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest("Username inválido".to_string()));
    }

    // 3. Validar força da senha
    validate_password_strength(&payload.password).map_err(AppError::BadRequest)?;

    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 4. Verificar se username já existe
    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Conflict("Username já está em uso".to_string()));
    }

    // 5. Verificar se email já existe
    if user_repo.exists_by_email(&payload.email).await? {
        return Err(AppError::Conflict("Email já está em uso".to_string()));
    }

    // 6. Hash da senha (operação blocking)
    let password_clone = payload.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha na task de hash")?
        .context("Erro ao gerar hash da senha")?;

    // 7. Criar usuário
    let user = user_repo
        .create(&payload.username, &payload.email, &password_hash)
        .await?;

    // 8. Gerar tokens
    let (access_token, refresh_token) = generate_tokens(&state, user.id).await?;

    // 9. Criar e enviar token de verificação de email
    let verification_token = create_verification_token(&state, user.id)
        .await
        .context("Falha ao criar token de verificação")?;

    // 10. Enviar emails (async, não bloqueia resposta)
    state.email_service.send_verification_email(
        payload.email.clone(),
        &user.username,
        &verification_token,
    );

    state
        .email_service
        .send_welcome_email(payload.email, &user.username);

    info!(user_id = %user.id, "Novo usuário registrado");

    Ok(Json(RegisterResponse::new(
        access_token,
        refresh_token,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /refresh-token
///
/// Renova o access token usando um refresh token válido.
/// Implementa rotação de tokens para segurança.
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Validação de refresh token falhou");
        AppError::Validation(e)
    })?;

    let old_token_hash = hash_token(&payload.refresh_token);

    // 2. Iniciar transação
    let mut tx = state
        .db_pool_auth
        .begin()
        .await
        .context("Falha ao iniciar transação")?;

    // 3. Buscar token antigo
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
            // 4. Detecção de roubo de token
            if token.revoked {
                // Token já foi usado - possível roubo!
                // Revogar toda a família de tokens
                sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE family_id = $1")
                    .bind(token.family_id)
                    .execute(&mut *tx)
                    .await
                    .context("Falha ao revogar família de tokens")?;

                tx.commit().await.context("Falha ao commitar revogação")?;

                warn!(
                    user_id = %token.user_id,
                    family_id = %token.family_id,
                    "Reuso de refresh token detectado - família revogada"
                );

                return Err(AppError::Unauthorized(
                    "Sessão invalidada por segurança".to_string(),
                ));
            }

            // 5. Verificar expiração
            if token.expires_at <= Utc::now() {
                return Err(AppError::Unauthorized("Refresh token expirado".to_string()));
            }

            // 6. Revogar token antigo
            sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE id = $1")
                .bind(token.id)
                .execute(&mut *tx)
                .await
                .context("Falha ao revogar token antigo")?;

            (token.user_id, token.family_id)
        }
    };

    // 7. Gerar novo Access Token
    let access_token = state
        .jwt_service
        .generate_token(user_id, TokenType::Access, ACCESS_TOKEN_EXPIRY_SECONDS)
        .map_err(|e| {
            error!("Erro ao gerar access token: {:?}", e);
            AppError::Anyhow(e)
        })?;

    // 8. Gerar e salvar novo Refresh Token (rotação)
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
    .bind(&new_refresh_token_hash)
    .bind(new_expires_at)
    .bind(family_id)
    .bind(&old_token_hash)
    .execute(&mut *tx)
    .await
    .context("Falha ao salvar novo refresh token")?;

    // 9. Commit da transação
    tx.commit()
        .await
        .context("Falha ao commitar rotação de token")?;

    info!(user_id = %user_id, "Token rotacionado com sucesso");

    Ok(Json(RefreshTokenResponse::new(
        access_token,
        new_refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /logout
///
/// Revoga o refresh token, invalidando a sessão.
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Validação de logout falhou");
        AppError::Validation(e)
    })?;

    let refresh_token_hash = hash_token(&payload.refresh_token);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    // 2. Revogar token
    if !auth_repo.revoke_refresh_token(&refresh_token_hash).await? {
        return Err(AppError::NotFound(
            "Token não encontrado ou já revogado".to_string(),
        ));
    }

    info!("Refresh token revogado (logout)");

    Ok(Json(LogoutResponse::default()))
}

/// POST /forgot-password
///
/// Solicita reset de senha. Envia email se o endereço existir.
/// Sempre retorna sucesso para evitar enumeração de usuários.
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Validação de forgot password falhou");
        AppError::Validation(e)
    })?;

    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 2. Buscar usuário pelo email
    let user = user_repo.find_by_email(&payload.email).await?;

    if let Some(user) = user {
        // 3. Gerar token de reset (JWT)
        let token = state
            .jwt_service
            .generate_token(
                user.id,
                TokenType::PasswordReset,
                PASSWORD_RESET_EXPIRY_SECONDS,
            )
            .map_err(|e| {
                error!("Erro ao gerar reset token: {:?}", e);
                AppError::Anyhow(e)
            })?;

        // 4. Enviar email (async)
        state.email_service.send_password_reset_email(
            payload.email.clone(),
            &user.username,
            &token,
        );

        info!(user_id = %user.id, "Email de reset de senha enviado");
    } else {
        info!(email = %payload.email, "Tentativa de reset para email não existente");
    }

    // Sempre retorna sucesso (evita enumeração)
    Ok(Json(ForgotPasswordResponse::default()))
}

/// POST /reset-password
///
/// Redefine a senha usando um token válido.
/// Revoga todas as sessões existentes do usuário.
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Validação de reset password falhou");
        AppError::Validation(e)
    })?;

    // 2. Validar força da nova senha
    validate_password_strength(&payload.new_password).map_err(AppError::BadRequest)?;

    // 3. Verificar token de reset
    let claims = state
        .jwt_service
        .verify_token(&payload.token, TokenType::PasswordReset)
        .map_err(|_| AppError::Unauthorized("Token inválido ou expirado".to_string()))?;

    let user_id = claims.sub;

    // 4. Hash da nova senha (operação blocking)
    let password_clone = payload.new_password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha na task de hash")?
        .context("Erro ao gerar hash da senha")?;

    // 5. Atualizar senha no banco
    let user_repo = UserRepository::new(&state.db_pool_auth);
    user_repo
        .update_password(user_id, &password_hash)
        .await
        .context("Falha ao atualizar senha")?;

    // 6. Revogar todos os refresh tokens do usuário
    let auth_repo = AuthRepository::new(&state.db_pool_auth);
    auth_repo
        .revoke_all_user_tokens(user_id)
        .await
        .context("Falha ao revogar tokens")?;

    info!(user_id = %user_id, "Senha redefinida com sucesso");

    Ok(Json(ResetPasswordResponse::default()))
}

// =============================================================================
// INTERNAL TYPES
// =============================================================================

/// Struct para deserialização do refresh token do banco
#[derive(sqlx::FromRow)]
struct RefreshTokenInfo {
    id: Uuid,
    user_id: Uuid,
    revoked: bool,
    expires_at: chrono::DateTime<chrono::Utc>,
    family_id: Uuid,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token() {
        let token = "test-token-123";
        let hash = hash_token(token);

        // Hash SHA-256 deve ter 64 caracteres hexadecimais
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Mesmo token sempre gera o mesmo hash
        let hash2 = hash_token(token);
        assert_eq!(hash, hash2);

        // Tokens diferentes geram hashes diferentes
        let hash3 = hash_token("different-token");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_constants() {
        assert_eq!(ACCESS_TOKEN_EXPIRY_SECONDS, 3600);
        assert_eq!(REFRESH_TOKEN_EXPIRY_SECONDS, 604800);
        assert_eq!(PASSWORD_RESET_EXPIRY_SECONDS, 900);
        assert_eq!(MFA_CHALLENGE_EXPIRY_SECONDS, 300);
    }
}
