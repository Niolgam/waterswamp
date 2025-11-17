use anyhow::Context;
use axum::{extract::State, Json};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use validator::Validate;

use crate::{error::AppError, state::AppState, web_models::CurrentUser};
use persistence::repositories::{
    email_verification_repository::EmailVerificationRepository, user_repository::UserRepository,
};

// Re-export from domain models (these should be added to domain/src/models.rs)
use serde::{Deserialize, Serialize};

const EMAIL_VERIFICATION_EXPIRY_HOURS: i64 = 24;
const MAX_VERIFICATION_REQUESTS_PER_HOUR: i64 = 3;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResendVerificationPayload {
    #[validate(email(message = "Email inválido"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct VerifyEmailPayload {
    #[validate(length(min = 1, message = "Token não pode estar vazio"))]
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationResponse {
    pub verified: bool,
    pub message: String,
}

/// POST /verify-email
/// Verifies user email with the provided token.
pub async fn handler_verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailPayload>,
) -> Result<Json<EmailVerificationResponse>, AppError> {
    payload.validate()?;

    let token_hash = hash_token(&payload.token);
    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);

    // 1. Find valid token
    let user_id = verification_repo
        .find_valid_token(&token_hash)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("Token de verificação inválido ou expirado".to_string())
        })?;

    // 2. Check if already verified
    if verification_repo.is_email_verified(user_id).await? {
        return Ok(Json(EmailVerificationResponse {
            verified: true,
            message: "Email já foi verificado anteriormente".to_string(),
        }));
    }

    // 3. Mark token as used
    verification_repo.mark_token_as_used(&token_hash).await?;

    // 4. Verify user's email
    verification_repo.verify_user_email(user_id).await?;

    tracing::info!(user_id = %user_id, event_type = "email_verified", "Email verificado com sucesso");

    Ok(Json(EmailVerificationResponse {
        verified: true,
        message: "Email verificado com sucesso! Sua conta está agora totalmente ativa.".to_string(),
    }))
}

/// POST /resend-verification
/// Resends verification email (with rate limiting).
pub async fn handler_resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;

    let user_repo = UserRepository::new(&state.db_pool_auth);
    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);

    // 1. Find user by email
    let user = match user_repo.find_by_email(&payload.email).await? {
        Some(u) => u,
        None => {
            // Don't reveal if email exists
            return Ok(Json(serde_json::json!({
                "message": "Se este email estiver registado e não verificado, um novo link de verificação foi enviado."
            })));
        }
    };

    // 2. Check if already verified
    if verification_repo.is_email_verified(user.id).await? {
        return Ok(Json(serde_json::json!({
            "message": "Este email já está verificado."
        })));
    }

    // 3. Rate limiting: Check recent requests
    let recent_requests = verification_repo
        .count_recent_verification_requests(user.id, 60) // Last 60 minutes
        .await?;

    if recent_requests >= MAX_VERIFICATION_REQUESTS_PER_HOUR {
        return Err(AppError::BadRequest(format!(
            "Limite de {} pedidos de verificação por hora atingido. Tente novamente mais tarde.",
            MAX_VERIFICATION_REQUESTS_PER_HOUR
        )));
    }

    // 4. Invalidate old tokens
    verification_repo.invalidate_all_tokens(user.id).await?;

    // 5. Generate new verification token
    let verification_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&verification_token);

    // 6. Save token
    verification_repo
        .save_verification_token(user.id, &token_hash, EMAIL_VERIFICATION_EXPIRY_HOURS)
        .await?;

    // 7. Send verification email
    state.email_service.send_verification_email(
        payload.email.clone(),
        &user.username,
        &verification_token,
    );

    tracing::info!(user_id = %user.id, event_type = "verification_email_resent", "Email de verificação reenviado");

    Ok(Json(serde_json::json!({
        "message": "Se este email estiver registado e não verificado, um novo link de verificação foi enviado."
    })))
}

/// GET /verification-status
/// Returns the email verification status for the current user.
pub async fn handler_verification_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);
    let is_verified = verification_repo.is_email_verified(current_user.id).await?;

    Ok(Json(serde_json::json!({
        "email_verified": is_verified
    })))
}

// --- Helper Functions ---

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generates a verification token and saves it to the database.
/// Called during user registration.
pub async fn create_verification_token(state: &AppState, user_id: Uuid) -> anyhow::Result<String> {
    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);

    let verification_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&verification_token);

    verification_repo
        .save_verification_token(user_id, &token_hash, EMAIL_VERIFICATION_EXPIRY_HOURS)
        .await
        .context("Falha ao salvar token de verificação")?;

    Ok(verification_token)
}
