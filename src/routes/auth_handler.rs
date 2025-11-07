use anyhow::Context;
use axum::{extract::State, Json};
use bcrypt::verify;
use jsonwebtoken::{encode, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use validator::Validate;

use crate::{
    error::AppError,
    models::{Claims, LoginPayload, LoginResponse, User},
    state::AppState,
};

pub async fn handler_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    payload.validate()?;
    let user: User = sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db_pool_auth)
        .await?
        .ok_or(AppError::InvalidPassword)?;

    let password_valid =
        tokio::task::spawn_blocking(move || verify(payload.password, &user.password_hash))
            .await
            .context("Falha ao executar tarefa de verificação de senha")?
            .map_err(|_| AppError::InvalidPassword)?;

    if !password_valid {
        return Err(AppError::InvalidPassword);
    }

    // 3. Criar token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Falha crítica no relógio do sistema")?
        .as_secs();

    let claims = Claims {
        sub: user.id,
        exp: (now + 3600) as i64,
    };

    let token = encode(&Header::default(), &claims, &state.encoding_key)
        .context("Falha ao codificar token JWT")?;

    Ok(Json(LoginResponse::new(token)))
}
