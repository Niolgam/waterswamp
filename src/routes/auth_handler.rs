use anyhow::Context;
use axum::{extract::State, Json};
use jsonwebtoken::{encode, Algorithm, Header};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        Claims, LoginPayload, LoginResponse, RefreshTokenPayload, RefreshTokenResponse,
        RegisterPayload, TokenType,
    },
    // Importando AMBOS os repositórios
    repositories::{auth_repository::AuthRepository, user_repository::UserRepository},
    security,
    state::AppState,
};

const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600; // 1 hora

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

    // 2. Verificar senha com Argon2id (CPU-bound, manter no spawn_blocking)
    let password_valid = tokio::task::spawn_blocking(move || {
        // USA A NOVA FUNÇÃO
        security::verify_password(&payload.password, &user.1)
    })
    .await
    .context("Falha task verificar senha")?
    .map_err(|_| AppError::InvalidPassword)?; // Trata erro de parse do hash como senha inválida genérica

    if !password_valid {
        return Err(AppError::InvalidPassword);
    }

    // 3. Gerar tokens usando AuthRepository
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

    crate::security::validate_password_strength(&payload.password).map_err(AppError::BadRequest)?;

    let user_repo = UserRepository::new(&state.db_pool_auth);

    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Conflict("Username já está em uso".to_string()));
    }

    let password_clone = payload.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || {
        // USA A NOVA FUNÇÃO
        security::hash_password(&password_clone)
    })
    .await
    .context("Falha task hash")?
    .context("Erro ao gerar hash")?;

    let user = user_repo.create(&payload.username, &password_hash).await?;

    let (access_token, refresh_token_raw) = generate_tokens(&state, user.id).await?;

    tracing::info!(user_id = %user.id, event_type = "user_registered", "Novo usuário registrado");

    Ok(Json(LoginResponse::new(
        access_token,
        refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /refresh-token
pub async fn handler_refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    payload.validate()?;

    let refresh_token_hash = hash_token(&payload.refresh_token);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    // Usando AuthRepository para validar o token
    let user_id = auth_repo
        .find_valid_refresh_token(&refresh_token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Refresh token inválido ou expirado".to_string()))?;

    // Gerar novo access token
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

    tracing::info!(user_id = %user_id, event_type = "token_refresh", "Access token renovado");

    Ok(Json(RefreshTokenResponse::new(
        access_token,
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

// Helper atualizado para usar AuthRepository
async fn generate_tokens(state: &AppState, user_id: Uuid) -> Result<(String, String), AppError> {
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
        encode(&header, &claims, &state.encoding_key).context("Falha ao gerar JWT")?;

    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);

    // Usando AuthRepository para salvar
    let auth_repo = AuthRepository::new(&state.db_pool_auth);
    auth_repo
        .save_refresh_token(user_id, &refresh_token_hash)
        .await?;

    Ok((access_token, refresh_token_raw))
}
