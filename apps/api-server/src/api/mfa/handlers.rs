use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};
use anyhow::Context;
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use core_services::security::verify_password;
use domain::models::TokenType;
use domain::ports::UserRepositoryPort;
use persistence::repositories::{mfa_repository::MfaRepository, user_repository::UserRepository};
use rand::Rng;
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

use super::contracts::*;

// Constants
const MFA_SETUP_EXPIRY_MINUTES: i64 = 15;
const BACKUP_CODES_COUNT: usize = 10;
const BACKUP_CODE_LENGTH: usize = 12;
const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600;
const REFRESH_TOKEN_EXPIRY_SECONDS: i64 = 604800; // 7 dias

// --- Helper Functions ---

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn hash_backup_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.to_uppercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn generate_backup_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();

    (0..BACKUP_CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_backup_codes() -> (Vec<String>, Vec<String>) {
    let mut plain_codes = Vec::with_capacity(BACKUP_CODES_COUNT);
    let mut hashed_codes = Vec::with_capacity(BACKUP_CODES_COUNT);

    for _ in 0..BACKUP_CODES_COUNT {
        let code = generate_backup_code();
        hashed_codes.push(hash_backup_code(&code));
        plain_codes.push(code);
    }

    (plain_codes, hashed_codes)
}

/// Helper local para gerar tokens (Access + Refresh)
async fn generate_tokens(state: &AppState, user_id: Uuid) -> Result<(String, String), AppError> {
    // 1. Generate Access Token
    let access_token = state
        .jwt_service
        .generate_token(user_id, TokenType::Access, ACCESS_TOKEN_EXPIRY_SECONDS)
        .map_err(|e| {
            error!("Erro ao gerar access token: {:?}", e);
            AppError::Anyhow(e)
        })?;

    // 2. Generate Refresh Token (Opaque)
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
    .context("Falha ao salvar refresh token")?;

    Ok((access_token, refresh_token_raw))
}

// --- Handlers ---

/// POST /auth/mfa/setup
/// Inicia o processo de configuração de MFA
pub async fn setup(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaSetupResponse>, AppError> {
    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    if mfa_repo.is_mfa_enabled(current_user.id).await? {
        return Err(AppError::BadRequest(
            "MFA já está ativado para esta conta".to_string(),
        ));
    }

    let user = user_repo
        .find_by_id(current_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Erro ao converter secret: {}", e))?,
        Some("Waterswamp".to_string()),
        user.username.as_str().to_string(),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    let qr_code_url = totp.get_url();

    let setup_id = mfa_repo
        .save_setup_token(current_user.id, &secret_base32, MFA_SETUP_EXPIRY_MINUTES)
        .await?;

    info!(user_id = %current_user.id, "Configuração de MFA iniciada");

    Ok(Json(MfaSetupResponse {
        secret: secret_base32,
        qr_code_url,
        setup_token: setup_id.to_string(),
    }))
}

/// POST /auth/mfa/verify-setup
/// Confirma a configuração e ativa o MFA
pub async fn verify_setup(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaVerifySetupRequest>,
) -> Result<Json<MfaSetupCompleteResponse>, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    let setup_id = Uuid::parse_str(&payload.setup_token)
        .map_err(|_| AppError::BadRequest("Token de setup inválido".to_string()))?;

    let (user_id, secret) = mfa_repo
        .find_valid_setup_token(setup_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Token de setup expirado ou inválido".to_string()))?;

    if user_id != current_user.id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let user = user_repo.find_by_id(user_id).await?.unwrap();

    let secret_bytes = Secret::Encoded(secret.clone())
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        user.username.as_str().to_string(),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    if !totp
        .check_current(&payload.totp_code)
        .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    {
        return Err(AppError::BadRequest(
            "Código TOTP inválido. Verifique se o código está correto.".to_string(),
        ));
    }

    let (backup_codes_plain, backup_codes_hashed) = generate_backup_codes();

    mfa_repo
        .enable_mfa(current_user.id, &secret, &backup_codes_hashed)
        .await?;

    mfa_repo.complete_setup(setup_id).await?;

    state.email_service.send_mfa_enabled_email(
        user.email.as_str().to_string(),
        &user.username.as_str().to_string(),
    );

    info!(user_id = %current_user.id, "MFA ativado com sucesso");

    Ok(Json(MfaSetupCompleteResponse {
        enabled: true,
        backup_codes: backup_codes_plain,
        message: "MFA ativado com sucesso! Guarde os códigos de backup num local seguro."
            .to_string(),
    }))
}

/// POST /auth/mfa/verify
/// Verifica código MFA durante o login
pub async fn verify(
    State(state): State<AppState>,
    Json(payload): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    // 1. Verify MFA challenge token
    let claims = state
        .jwt_service
        .verify_mfa_token(&payload.mfa_token)
        .map_err(|e| {
            warn!(error = %e, "Falha na validação do token MFA");
            AppError::Unauthorized("Token MFA inválido ou expirado".to_string())
        })?;

    let user_id = claims.sub;
    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    let (mfa_enabled, mfa_secret, backup_codes) = mfa_repo
        .get_mfa_info(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    if !mfa_enabled || mfa_secret.is_none() {
        return Err(AppError::BadRequest("MFA não está ativado".to_string()));
    }

    let secret = mfa_secret.unwrap();
    let user = user_repo.find_by_id(user_id).await?.unwrap();
    let mut backup_code_used = false;

    let code_valid = if payload.code.len() == 6 && payload.code.chars().all(|c| c.is_ascii_digit())
    {
        let secret_bytes = Secret::Encoded(secret)
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

        let totp = TOTP::new(
            TotpAlgorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("Waterswamp".to_string()),
            user.username.as_str().to_string(),
        )
        .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

        totp.check_current(&payload.code)
            .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    } else {
        let code_hash = hash_backup_code(&payload.code);

        if let Some(codes) = backup_codes {
            if codes.contains(&code_hash) {
                mfa_repo.remove_backup_code(user_id, &code_hash).await?;
                mfa_repo
                    .record_backup_code_usage(user_id, &code_hash, None)
                    .await
                    .ok();
                backup_code_used = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if !code_valid {
        warn!(user_id = %user_id, "Verificação MFA falhou");
        return Err(AppError::Unauthorized("Código MFA inválido".to_string()));
    }

    // 5. Generate tokens
    let (access_token, refresh_token) = generate_tokens(&state, user_id).await?;

    if backup_code_used {
        warn!(user_id = %user_id, "Código de backup usado");
    }

    info!(user_id = %user_id, "Verificação MFA bem-sucedida");

    Ok(Json(MfaVerifyResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_EXPIRY_SECONDS,
        backup_code_used,
    }))
}

/// POST /auth/mfa/disable
/// Desativa o MFA
pub async fn disable(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaDisableRequest>,
) -> Result<Json<MfaDisableResponse>, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    let user_with_hash: (String, Option<String>) =
        sqlx::query_as("SELECT password_hash, mfa_secret FROM users WHERE id = $1")
            .bind(current_user.id)
            .fetch_one(&state.db_pool_auth)
            .await?;

    let password_hash = user_with_hash.0.clone();
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .context("Falha task verificar senha")?
            .map_err(|_| AppError::Unauthorized("Senha incorreta".to_string()))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Senha incorreta".to_string()));
    }

    let user = user_repo.find_by_id(current_user.id).await?.unwrap();

    let secret = user_with_hash
        .1
        .ok_or_else(|| AppError::BadRequest("MFA não está ativado".to_string()))?;

    let secret_bytes = Secret::Encoded(secret)
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        user.username.as_str().to_string(),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    if !totp
        .check_current(&payload.totp_code)
        .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    {
        return Err(AppError::BadRequest("Código TOTP inválido".to_string()));
    }

    mfa_repo.disable_mfa(current_user.id).await?;

    warn!(user_id = %current_user.id, "MFA desativado");

    Ok(Json(MfaDisableResponse {
        disabled: true,
        message: "MFA foi desativado com sucesso.".to_string(),
    }))
}

/// POST /auth/mfa/regenerate-backup-codes
pub async fn regenerate_backup_codes(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaRegenerateBackupCodesRequest>,
) -> Result<Json<MfaBackupCodesResponse>, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    if !mfa_repo.is_mfa_enabled(current_user.id).await? {
        return Err(AppError::BadRequest("MFA não está ativado".to_string()));
    }

    let user_info: (String, Option<String>) =
        sqlx::query_as("SELECT password_hash, mfa_secret FROM users WHERE id = $1")
            .bind(current_user.id)
            .fetch_one(&state.db_pool_auth)
            .await?;

    let password_hash = user_info.0;
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .context("Falha task verificar senha")?
            .map_err(|_| AppError::Unauthorized("Senha incorreta".to_string()))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Senha incorreta".to_string()));
    }

    let user = user_repo.find_by_id(current_user.id).await?.unwrap();

    let secret = user_info
        .1
        .ok_or_else(|| AppError::BadRequest("MFA secret não encontrado".to_string()))?;

    let secret_bytes = Secret::Encoded(secret)
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        user.username.as_str().to_string(),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    if !totp
        .check_current(&payload.totp_code)
        .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    {
        return Err(AppError::BadRequest("Código TOTP inválido".to_string()));
    }

    let (backup_codes_plain, backup_codes_hashed) = generate_backup_codes();

    mfa_repo
        .update_backup_codes(current_user.id, &backup_codes_hashed)
        .await?;

    info!(user_id = %current_user.id, "Códigos de backup regenerados");

    Ok(Json(MfaBackupCodesResponse {
        backup_codes: backup_codes_plain,
        message: "Novos códigos de backup gerados. Os códigos anteriores foram invalidados."
            .to_string(),
    }))
}

/// GET /auth/mfa/status
pub async fn status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaStatusResponse>, AppError> {
    let mfa_repo = MfaRepository::new(&state.db_pool_auth);

    let enabled = mfa_repo.is_mfa_enabled(current_user.id).await?;

    let backup_codes_remaining = if enabled {
        Some(mfa_repo.count_backup_codes(current_user.id).await?)
    } else {
        None
    };

    Ok(Json(MfaStatusResponse {
        enabled,
        backup_codes_remaining,
    }))
}
