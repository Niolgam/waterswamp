use crate::infra::{errors::AppError, state::AppState};
use axum::{extract::State, Json};
use domain::ports::{MfaRepositoryPort, UserRepositoryPort};
use persistence::repositories::{mfa_repository::MfaRepository, user_repository::UserRepository};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::instrument;
use validator::Validate;

use domain::models::{
    MfaBackupCodesResponse, MfaDisableResponse, MfaSetupCompleteResponse, MfaSetupResponse,
    MfaStatusResponse, MfaVerifyResponse,
};

use super::contracts::{
    MfaDisableRequest, MfaRegenerateBackupCodesRequest, MfaVerifyRequest, MfaVerifySetupRequest,
};

use crate::extractors::current_user::CurrentUser;

const MFA_ISSUER: &str = "Waterswamp";

// --- HANDLERS ---

#[instrument(skip_all)]
pub async fn initiate_setup(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaSetupResponse>, AppError> {
    let response = state
        .mfa_service
        .initiate_setup(current_user.id, &current_user.username)
        .await
        .map_err(|e| {
            use application::errors::ServiceError;
            match e {
                ServiceError::BadRequest(msg) => AppError::BadRequest(msg),
                _ => AppError::Anyhow(anyhow::anyhow!(e)),
            }
        })?;

    Ok(Json(response))
}

#[instrument(skip_all)]
pub async fn verify_setup(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    Json(payload): Json<MfaVerifySetupRequest>,
) -> Result<Json<MfaSetupCompleteResponse>, AppError> {
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
                // Return 400 for invalid code during setup
                ServiceError::InvalidCredentials => {
                    AppError::BadRequest("Código ou token inválido".to_string())
                }
                _ => AppError::Anyhow(anyhow::anyhow!(e)),
            }
        })?;

    Ok(Json(response))
}

#[instrument(skip_all)]
pub async fn verify_login(
    State(state): State<AppState>,
    Json(payload): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
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

#[instrument(skip_all)]
pub async fn disable_mfa(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaDisableRequest>,
) -> Result<Json<MfaDisableResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let mfa_repo = MfaRepository::new(state.db_pool_auth.clone());
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    // 1. Verify Password
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

    // 2. Verify TOTP
    let secret_str = mfa_repo
        .get_mfa_secret(current_user.id)
        .await?
        .ok_or(AppError::BadRequest("MFA não ativo".to_string()))?;

    // FIX: Converted error to String
    let secret_bytes = Secret::Encoded(secret_str)
        .to_bytes()
        .map_err(|_| AppError::Internal("Falha ao decodificar segredo MFA".to_string()))?;

    let user_data = user_repo.find_by_id(current_user.id).await?.unwrap();

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some(MFA_ISSUER.to_string()),
        user_data.username.to_string(), // FIX: Converted Username to String
    )
    .unwrap();

    if !totp.check_current(&payload.totp_code).unwrap_or(false) {
        return Err(AppError::Unauthorized("Código inválido".to_string()));
    }

    // 3. Disable
    mfa_repo.disable_mfa(current_user.id).await?;

    Ok(Json(MfaDisableResponse {
        disabled: true,
        message: "Desativado".to_string(),
    }))
}

#[instrument(skip_all)]
pub async fn get_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaStatusResponse>, AppError> {
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

#[instrument(skip_all)]
pub async fn regenerate_backup_codes(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaRegenerateBackupCodesRequest>,
) -> Result<Json<MfaBackupCodesResponse>, AppError> {
    let mfa_repo = MfaRepository::new(state.db_pool_auth.clone());
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    // 1. Verify Password
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

    // 2. Verify TOTP
    let secret_str = mfa_repo
        .get_mfa_secret(current_user.id)
        .await?
        .ok_or(AppError::BadRequest("MFA não ativo".to_string()))?;

    // FIX: Converted error to String
    let secret_bytes = Secret::Encoded(secret_str)
        .to_bytes()
        .map_err(|_| AppError::Internal("Falha ao decodificar segredo MFA".to_string()))?;

    let user_data = user_repo.find_by_id(current_user.id).await?.unwrap();
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some(MFA_ISSUER.to_string()),
        user_data.username.to_string(), // FIX: Converted Username to String
    )
    .unwrap();

    if !totp.check_current(&payload.totp_code).unwrap_or(false) {
        return Err(AppError::Unauthorized("Código TOTP inválido".to_string()));
    }

    // 3. Regenerate
    let response = state
        .mfa_service
        .regenerate_backup_codes(current_user.id)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    Ok(Json(response))
}
