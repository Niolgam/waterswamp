use anyhow::Context;
use axum::{extract::State, Json};
use bcrypt::verify;
use jsonwebtoken::{encode, Header};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        Claims, LoginPayload, LoginResponse, RefreshTokenPayload, RefreshTokenResponse,
        RegisterPayload, TokenType, User,
    },
    state::AppState,
};

// Constantes de expiração
const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600; // 1 hora
                                               // const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30; // 30 dias (usado diretamente na query SQL)

/// POST /login
/// Autentica usuário e retorna access token + refresh token
pub async fn handler_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;

    // 1. Buscar usuário
    let user: User = sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db_pool_auth)
        .await?
        .ok_or(AppError::InvalidPassword)?;

    // 2. Verificar senha
    let password_valid =
        tokio::task::spawn_blocking(move || verify(payload.password, &user.password_hash))
            .await
            .context("Falha ao executar tarefa de verificação de senha")?
            .map_err(|_| AppError::InvalidPassword)?;

    if !password_valid {
        return Err(AppError::InvalidPassword);
    }

    // 3. Gerar access token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Falha crítica no relógio do sistema")?
        .as_secs() as i64;

    let access_claims = Claims {
        sub: user.id,
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
        token_type: TokenType::Access,
    };

    let access_token = encode(&Header::default(), &access_claims, &state.encoding_key)
        .context("Falha ao codificar access token JWT")?;

    // 4. Gerar refresh token
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);

    // 5. Armazenar refresh token no banco usando PostgreSQL interval
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '30 days')
        "#,
    )
    .bind(user.id)
    .bind(&refresh_token_hash)
    .execute(&state.db_pool_auth)
    .await?;

    tracing::info!(
        user_id = %user.id,
        event_type = "user_login",
        "Usuário autenticado com sucesso"
    );

    Ok(Json(LoginResponse::new(
        access_token,
        refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /refresh-token
/// Renova access token usando refresh token válido
pub async fn handler_refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    payload.validate()?;

    let refresh_token_hash = hash_token(&payload.refresh_token);

    // 1. Buscar e validar refresh token em uma query
    // A validação de expiração e revogação é feita no SQL
    let user_id: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id
        FROM refresh_tokens
        WHERE token_hash = $1
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(&refresh_token_hash)
    .fetch_optional(&state.db_pool_auth)
    .await?;

    let (user_id,) = user_id.ok_or_else(|| {
        tracing::warn!("Tentativa de usar refresh token inválido, revogado ou expirado");
        AppError::Unauthorized("Refresh token inválido".to_string())
    })?;

    // 2. Gerar novo access token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Falha crítica no relógio do sistema")?
        .as_secs() as i64;

    let access_claims = Claims {
        sub: user_id,
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
        token_type: TokenType::Access,
    };

    let access_token = encode(&Header::default(), &access_claims, &state.encoding_key)
        .context("Falha ao codificar access token JWT")?;

    tracing::info!(
        user_id = %user_id,
        event_type = "token_refresh",
        "Access token renovado com sucesso"
    );

    Ok(Json(RefreshTokenResponse::new(
        access_token,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}

/// POST /logout
/// Revoga refresh token do usuário atual
pub async fn handler_logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;

    let refresh_token_hash = hash_token(&payload.refresh_token);

    // Revogar o refresh token
    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE, updated_at = NOW()
        WHERE token_hash = $1 AND revoked = FALSE
        "#,
    )
    .bind(&refresh_token_hash)
    .execute(&state.db_pool_auth)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Refresh token não encontrado ou já revogado".to_string(),
        ));
    }

    tracing::info!(
        event_type = "user_logout",
        "Refresh token revogado (logout)"
    );

    Ok(Json(serde_json::json!({
        "message": "Logout realizado com sucesso"
    })))
}

/// Helper: Hash de token usando SHA-256
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// POST /register
/// Cria um novo usuário e retorna tokens
pub async fn handler_register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    // 1. Validar payload básico (tamanho, required)
    payload.validate()?;

    // 2. Validar formato do username
    // Adicionado '-' para permitir UUIDs ou nomes compostos comuns
    if !payload
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest(
            "Username inválido (apenas letras, números, '_' e '-')".to_string(),
        ));
    }

    // 3. Validar força da senha
    if let Err(e) = crate::security::validate_password_strength(&payload.password) {
        return Err(AppError::BadRequest(e));
    }

    // 4. Verificar se username já existe
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
        .bind(&payload.username)
        .fetch_one(&state.db_pool_auth)
        .await?;

    if exists {
        return Err(AppError::Conflict("Username já está em uso".to_string()));
    }

    // 5. Hash da senha (CPU-bound, executado em thread separada)
    let password_clone = payload.password.clone();
    let password_hash =
        tokio::task::spawn_blocking(move || bcrypt::hash(password_clone, bcrypt::DEFAULT_COST))
            .await
            .context("Falha ao executar hash de senha")?
            .context("Erro ao gerar hash bcrypt")?;

    // 6. Inserir usuário no banco
    let user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&state.db_pool_auth)
    .await?;

    // 7. Gerar access token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Falha crítica no relógio do sistema")?
        .as_secs() as i64;

    let access_claims = Claims {
        sub: user_id,
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
        token_type: TokenType::Access,
    };

    let access_token = encode(&Header::default(), &access_claims, &state.encoding_key)
        .context("Falha ao codificar access token JWT")?;

    // 8. Gerar refresh token inicial
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '30 days')
        "#,
    )
    .bind(user_id)
    .bind(&refresh_token_hash)
    .execute(&state.db_pool_auth)
    .await?;

    tracing::info!(
        user_id = %user_id,
        username = %payload.username,
        event_type = "user_registered",
        "Novo usuário registrado com sucesso"
    );

    Ok(Json(LoginResponse::new(
        access_token,
        refresh_token_raw,
        ACCESS_TOKEN_EXPIRY_SECONDS,
    )))
}
