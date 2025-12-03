use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};
use anyhow::Context;
use axum::{extract::State, Json};
use domain::ports::UserRepositoryPort;
use persistence::repositories::{
    email_verification_repository::EmailVerificationRepository, user_repository::UserRepository,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use validator::Validate;

use super::contracts::{
    EmailVerificationResponse, ResendVerificationRequest, VerificationStatusResponse,
    VerifyEmailRequest,
};

const EMAIL_VERIFICATION_EXPIRY_HOURS: i64 = 24;
const MAX_VERIFICATION_REQUESTS_PER_HOUR: i64 = 3;

// --- Helper Functions ---

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Gera um token de verificação e salva no banco (utilitário interno)
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

// --- Handlers ---

/// POST /verify-email
/// Verifica o email do usuário usando o token fornecido.
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Json<EmailVerificationResponse>, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let token_hash = hash_token(&payload.token);
    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);

    // 1. Encontrar token válido
    let user_id = verification_repo
        .find_valid_token(&token_hash)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("Token de verificação inválido ou expirado".to_string())
        })?;

    // 2. Verificar se já foi validado
    if verification_repo.is_email_verified(user_id).await? {
        return Ok(Json(EmailVerificationResponse {
            verified: true,
            message: "Email já foi verificado anteriormente".to_string(),
        }));
    }

    // 3. Marcar token como usado
    verification_repo.mark_token_as_used(&token_hash).await?;

    // 4. Atualizar status do usuário
    verification_repo.verify_user_email(user_id).await?;

    tracing::info!(user_id = %user_id, "Email verificado com sucesso");

    Ok(Json(EmailVerificationResponse {
        verified: true,
        message: "Email verificado com sucesso! Sua conta está agora totalmente ativa.".to_string(),
    }))
}

/// POST /resend-verification
/// Reenvia o email de verificação (com rate limiting).
pub async fn resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    // 1. Encontrar usuário
    let user = match user_repo.find_by_email(&payload.email).await? {
        Some(u) => u,
        None => {
            // Retorno genérico para evitar enumeração de emails
            return Ok(Json(serde_json::json!({
                "message": "Se este email estiver registado e não verificado, um novo link de verificação foi enviado."
            })));
        }
    };

    // 2. Verificar se já validado
    if verification_repo.is_email_verified(user.id).await? {
        return Ok(Json(serde_json::json!({
            "message": "Este email já está verificado."
        })));
    }

    // 3. Rate Limiting (Regra de negócio)
    let recent_requests = verification_repo
        .count_recent_verification_requests(user.id, 60) // Últimos 60 min
        .await?;

    if recent_requests >= MAX_VERIFICATION_REQUESTS_PER_HOUR {
        return Err(AppError::BadRequest(format!(
            "Limite de {} pedidos de verificação por hora atingido. Tente novamente mais tarde.",
            MAX_VERIFICATION_REQUESTS_PER_HOUR
        )));
    }

    // 4. Invalidar tokens antigos
    verification_repo.invalidate_all_tokens(user.id).await?;

    // 5. Gerar e salvar novo token
    let verification_token = create_verification_token(&state, user.id).await?;

    // 6. Enviar email
    state.email_service.send_verification_email(
        payload.email.as_str().to_string(),
        user.username.as_str(),
        &verification_token,
    );

    tracing::info!(user_id = %user.id, "Email de verificação reenviado");

    Ok(Json(serde_json::json!({
        "message": "Se este email estiver registado e não verificado, um novo link de verificação foi enviado."
    })))
}

/// GET /verification-status
/// Retorna o status de verificação do usuário atual.
pub async fn verification_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<VerificationStatusResponse>, AppError> {
    let verification_repo = EmailVerificationRepository::new(&state.db_pool_auth);
    let is_verified = verification_repo.is_email_verified(current_user.id).await?;

    Ok(Json(VerificationStatusResponse {
        email_verified: is_verified,
    }))
}
