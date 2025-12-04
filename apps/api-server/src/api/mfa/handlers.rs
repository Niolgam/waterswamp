use crate::infra::{errors::AppError, state::AppState};
use axum::{extract::State, Json};
use domain::ports::{MfaRepositoryPort, UserRepositoryPort};
use persistence::repositories::{mfa_repository::MfaRepository, user_repository::UserRepository};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, TOTP};
use tracing::instrument;
use validator::Validate;

// CORREÇÃO 2: Usar tipos do Domínio para Respostas (evita mismatched types)
use domain::models::{
    MfaBackupCodesResponse, MfaDisableResponse, MfaSetupCompleteResponse, MfaSetupResponse,
    MfaStatusResponse, MfaVerifyResponse,
};

// Usar tipos locais apenas para Requests (Input DTOs)
use super::contracts::{
    MfaDisableRequest, MfaRegenerateBackupCodesRequest, MfaVerifyRequest, MfaVerifySetupRequest,
};

use crate::extractors::current_user::CurrentUser;

const MFA_ISSUER: &str = "Waterswamp";

fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

// --- HANDLERS ---

/// POST /auth/mfa/setup/initiate
#[instrument(skip_all)]
pub async fn initiate_setup(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaSetupResponse>, AppError> {
    // Retorna tipo do domínio
    // Chama o serviço (que já retorna o tipo do domínio)
    let response = state
        .mfa_service
        .initiate_setup(current_user.id, &current_user.username)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    Ok(Json(response))
}

/// POST /auth/mfa/setup/verify
#[instrument(skip_all)]
pub async fn verify_setup(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Json(payload): Json<MfaVerifySetupRequest>,
) -> Result<Json<MfaSetupCompleteResponse>, AppError> {
    // Retorna tipo do domínio
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let response = state
        .mfa_service
        .complete_setup(&payload.setup_token, &payload.totp_code)
        .await
        .map_err(|e| {
            use application::errors::ServiceError;
            match e {
                ServiceError::InvalidCredentials => {
                    AppError::Unauthorized("Código ou token inválido".to_string())
                }
                _ => AppError::Anyhow(anyhow::anyhow!(e)),
            }
        })?;

    Ok(Json(response))
}

/// POST /auth/mfa/verify (Login)
#[instrument(skip_all)]
pub async fn verify_login(
    State(state): State<AppState>,
    Json(payload): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    // Retorna tipo do domínio
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let response = state
        .mfa_service
        .verify_login(&payload.mfa_token, &payload.code)
        .await
        .map_err(|e| {
            use application::errors::ServiceError;
            match e {
                ServiceError::InvalidCredentials => {
                    AppError::Unauthorized("Código inválido ou expirado".to_string())
                }
                _ => AppError::Anyhow(anyhow::anyhow!(e)),
            }
        })?;

    Ok(Json(response))
}

/// POST /auth/mfa/disable
#[instrument(skip_all)]
pub async fn disable_mfa(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaDisableRequest>,
) -> Result<Json<MfaDisableResponse>, AppError> {
    // Retorna tipo do domínio
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    // Lógica antiga (temporária até mover disable_mfa com senha para o service)
    let mfa_repo = MfaRepository::new(state.db_pool_auth.clone());
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    let stored_hash = user_repo
        .get_password_hash(current_user.id)
        .await?
        .ok_or(AppError::NotFound("Usuário não encontrado".to_string()))?;

    let pwd = payload.password.clone();
    let password_valid = tokio::task::spawn_blocking(move || {
        core_services::security::verify_password(&pwd, &stored_hash)
    })
    .await
    .map_err(|_| AppError::Internal("Erro task".into()))??;

    if !password_valid {
        return Err(AppError::Unauthorized("Senha incorreta".to_string()));
    }

    // Validação TOTP manual (temporária)
    let secret = mfa_repo
        .get_mfa_secret(current_user.id)
        .await?
        .ok_or(AppError::BadRequest("MFA não ativo".to_string()))?;

    let user_data = user_repo.find_by_id(current_user.id).await?.unwrap();

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
        Some(MFA_ISSUER.to_string()),
        user_data.username.as_str().to_string(),
    )
    .unwrap();

    if !totp.check_current(&payload.totp_code).unwrap_or(false) {
        return Err(AppError::Unauthorized("Código inválido".to_string()));
    }

    mfa_repo.disable_mfa(current_user.id).await?;

    Ok(Json(MfaDisableResponse {
        disabled: true,
        message: "Desativado".to_string(),
    }))
}

/// GET /auth/mfa/status
#[instrument(skip_all)]
pub async fn get_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaStatusResponse>, AppError> {
    // Retorna tipo do domínio
    let mfa_repo = MfaRepository::new(state.db_pool_auth.clone());

    let enabled = mfa_repo.is_mfa_enabled(current_user.id).await?;
    let backup_remaining = if enabled {
        Some(mfa_repo.get_backup_codes(current_user.id).await?.len())
    } else {
        None
    };

    Ok(Json(MfaStatusResponse {
        enabled,
        backup_codes_remaining: backup_remaining,
    }))
}

/// POST /auth/mfa/regenerate-backup-codes
pub async fn regenerate_backup_codes(
    State(_state): State<AppState>,
    _current_user: CurrentUser,
    Json(_payload): Json<MfaRegenerateBackupCodesRequest>,
) -> Result<Json<MfaBackupCodesResponse>, AppError> {
    // Retorna tipo do domínio
    Ok(Json(MfaBackupCodesResponse {
        backup_codes: vec![],
        message: "TODO".to_string(),
    }))
}
