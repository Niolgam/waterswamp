use anyhow::Context;
use axum::{extract::State, Json};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use uuid::Uuid;
use validator::Validate;

use crate::{error::AppError, state::AppState, web_models::CurrentUser};
use core_services::security::verify_password;
use domain::models::Claims;
use persistence::repositories::{mfa_repository::MfaRepository, user_repository::UserRepository};

use serde::{Deserialize, Serialize};

// Constants
const MFA_SETUP_EXPIRY_MINUTES: i64 = 15;
const MFA_CHALLENGE_EXPIRY_SECONDS: i64 = 300; // 5 minutes
const BACKUP_CODES_COUNT: usize = 10;
const BACKUP_CODE_LENGTH: usize = 12;

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub setup_token: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaVerifySetupPayload {
    #[validate(length(min = 1, message = "Setup token não pode estar vazio"))]
    pub setup_token: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupCompleteResponse {
    pub enabled: bool,
    pub backup_codes: Vec<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaVerifyPayload {
    #[validate(length(min = 1, message = "MFA token não pode estar vazio"))]
    pub mfa_token: String,
    #[validate(length(min = 6, max = 12, message = "Código deve ter entre 6 e 12 caracteres"))]
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub mfa_token: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaVerifyResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub backup_code_used: bool,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaDisablePayload {
    #[validate(length(min = 1, message = "Senha não pode estar vazia"))]
    pub password: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaDisableResponse {
    pub disabled: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaRegenerateBackupCodesPayload {
    #[validate(length(min = 1, message = "Senha não pode estar vazia"))]
    pub password: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaBackupCodesResponse {
    pub backup_codes: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaStatusResponse {
    pub enabled: bool,
    pub backup_codes_remaining: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaChallengeClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /auth/mfa/setup
/// Initiates MFA setup by generating a TOTP secret and QR code.
pub async fn handler_mfa_setup(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaSetupResponse>, AppError> {
    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 1. Check if MFA is already enabled
    if mfa_repo.is_mfa_enabled(current_user.id).await? {
        return Err(AppError::BadRequest(
            "MFA já está ativado para esta conta".to_string(),
        ));
    }

    // 2. Get user info for QR code
    let user = user_repo
        .find_by_id(current_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    // 3. Generate TOTP secret
    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();

    // 4. Create TOTP instance for QR code URL
    // FIXED: totp-rs 5.7.0 API requires issuer and account_name in constructor
    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Erro ao converter secret: {}", e))?,
        Some("Waterswamp".to_string()), // issuer
        user.username.clone(),          // account_name
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    // FIXED: get_url() now takes no arguments (issuer and account_name are in constructor)
    let qr_code_url = totp.get_url();

    // 5. Save setup token (temporary storage)
    let setup_id = mfa_repo
        .save_setup_token(current_user.id, &secret_base32, MFA_SETUP_EXPIRY_MINUTES)
        .await?;

    tracing::info!(user_id = %current_user.id, event_type = "mfa_setup_initiated", "Configuração de MFA iniciada");

    Ok(Json(MfaSetupResponse {
        secret: secret_base32,
        qr_code_url,
        setup_token: setup_id.to_string(),
    }))
}

/// POST /auth/mfa/verify-setup
/// Completes MFA setup by verifying the TOTP code.
pub async fn handler_mfa_verify_setup(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaVerifySetupPayload>,
) -> Result<Json<MfaSetupCompleteResponse>, AppError> {
    payload.validate()?;

    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 1. Parse setup token
    let setup_id = Uuid::parse_str(&payload.setup_token)
        .map_err(|_| AppError::BadRequest("Token de setup inválido".to_string()))?;

    // 2. Find valid setup token
    let (user_id, secret) = mfa_repo
        .find_valid_setup_token(setup_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Token de setup expirado ou inválido".to_string()))?;

    // 3. Verify it belongs to current user
    if user_id != current_user.id {
        return Err(AppError::Forbidden);
    }

    // 4. Get user info for TOTP
    let user = user_repo.find_by_id(user_id).await?.unwrap();

    // 5. Verify TOTP code
    let secret_bytes = Secret::Encoded(secret.clone())
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

    // FIXED: Include issuer and account_name
    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        user.username.clone(),
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

    // 6. Generate backup codes
    let (backup_codes_plain, backup_codes_hashed) = generate_backup_codes();

    // 7. Enable MFA
    mfa_repo
        .enable_mfa(current_user.id, &secret, &backup_codes_hashed)
        .await?;

    // 8. Complete setup (delete temporary token)
    mfa_repo.complete_setup(setup_id).await?;

    // 9. Send notification email
    state
        .email_service
        .send_mfa_enabled_email(user.email, &user.username);

    tracing::info!(user_id = %current_user.id, event_type = "mfa_enabled", "MFA ativado com sucesso");

    Ok(Json(MfaSetupCompleteResponse {
        enabled: true,
        backup_codes: backup_codes_plain,
        message: "MFA ativado com sucesso! Guarde os códigos de backup num local seguro."
            .to_string(),
    }))
}

/// POST /auth/mfa/verify
/// Verifies MFA code during login flow.
pub async fn handler_mfa_verify(
    State(state): State<AppState>,
    Json(payload): Json<MfaVerifyPayload>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    payload.validate()?;

    // 1. Decode MFA challenge token
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = 5;

    let token_data =
        decode::<MfaChallengeClaims>(&payload.mfa_token, &state.decoding_key, &validation)
            .map_err(|e| {
                tracing::warn!(error = %e, "Falha na validação do token MFA");
                AppError::Unauthorized("Token MFA inválido ou expirado".to_string())
            })?;

    let claims = token_data.claims;

    // 2. Verify token type
    if claims.token_type != "mfa_challenge" {
        return Err(AppError::Unauthorized(
            "Token inválido (tipo incorreto)".to_string(),
        ));
    }

    let user_id = claims.sub;
    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 3. Get MFA info
    let (mfa_enabled, mfa_secret, backup_codes) = mfa_repo
        .get_mfa_info(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    if !mfa_enabled || mfa_secret.is_none() {
        return Err(AppError::BadRequest("MFA não está ativado".to_string()));
    }

    let secret = mfa_secret.unwrap();

    // Get username for TOTP
    let user = user_repo.find_by_id(user_id).await?.unwrap();

    let mut backup_code_used = false;

    // 4. Check if code is TOTP (6 digits) or backup code (12 chars)
    let code_valid = if payload.code.len() == 6 && payload.code.chars().all(|c| c.is_ascii_digit())
    {
        // Try TOTP
        let secret_bytes = Secret::Encoded(secret)
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

        // FIXED: Include issuer and account_name
        let totp = TOTP::new(
            TotpAlgorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("Waterswamp".to_string()),
            user.username.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

        totp.check_current(&payload.code)
            .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    } else {
        // Try backup code
        let code_hash = hash_backup_code(&payload.code);

        if let Some(codes) = backup_codes {
            if codes.contains(&code_hash) {
                // Valid backup code - remove it
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
        tracing::warn!(user_id = %user_id, event_type = "mfa_verification_failed", "Verificação MFA falhou");
        return Err(AppError::Unauthorized("Código MFA inválido".to_string()));
    }

    // 5. Generate access and refresh tokens
    let (access_token, refresh_token) = generate_tokens(&state, user_id).await?;

    if backup_code_used {
        tracing::warn!(user_id = %user_id, event_type = "mfa_backup_code_used", "Código de backup usado");
    }

    tracing::info!(user_id = %user_id, event_type = "mfa_verification_success", "Verificação MFA bem-sucedida");

    Ok(Json(MfaVerifyResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        backup_code_used,
    }))
}

/// POST /auth/mfa/disable
/// Disables MFA for the current user.
pub async fn handler_mfa_disable(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaDisablePayload>,
) -> Result<Json<MfaDisableResponse>, AppError> {
    payload.validate()?;

    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 1. Get user's password hash and MFA secret
    let user_with_hash: (String, Option<String>) =
        sqlx::query_as("SELECT password_hash, mfa_secret FROM users WHERE id = $1")
            .bind(current_user.id)
            .fetch_one(&state.db_pool_auth)
            .await?;

    // 2. Verify password
    let password_hash = user_with_hash.0.clone();
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .context("Falha task verificar senha")?
            .map_err(|_| AppError::Unauthorized("Senha incorreta".to_string()))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Senha incorreta".to_string()));
    }

    // 3. Get username for TOTP
    let user = user_repo.find_by_id(current_user.id).await?.unwrap();

    // 4. Verify TOTP code
    let secret = user_with_hash
        .1
        .ok_or_else(|| AppError::BadRequest("MFA não está ativado".to_string()))?;

    let secret_bytes = Secret::Encoded(secret)
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

    // FIXED: Include issuer and account_name
    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        user.username,
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    if !totp
        .check_current(&payload.totp_code)
        .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    {
        return Err(AppError::BadRequest("Código TOTP inválido".to_string()));
    }

    // 5. Disable MFA
    mfa_repo.disable_mfa(current_user.id).await?;

    tracing::warn!(user_id = %current_user.id, event_type = "mfa_disabled", "MFA desativado");

    Ok(Json(MfaDisableResponse {
        disabled: true,
        message: "MFA foi desativado com sucesso.".to_string(),
    }))
}

/// POST /auth/mfa/regenerate-backup-codes
/// Regenerates backup codes for the current user.
pub async fn handler_mfa_regenerate_backup_codes(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<MfaRegenerateBackupCodesPayload>,
) -> Result<Json<MfaBackupCodesResponse>, AppError> {
    payload.validate()?;

    let mfa_repo = MfaRepository::new(&state.db_pool_auth);
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 1. Verify MFA is enabled
    if !mfa_repo.is_mfa_enabled(current_user.id).await? {
        return Err(AppError::BadRequest("MFA não está ativado".to_string()));
    }

    // 2. Get user's password hash and MFA secret
    let user_info: (String, Option<String>) =
        sqlx::query_as("SELECT password_hash, mfa_secret FROM users WHERE id = $1")
            .bind(current_user.id)
            .fetch_one(&state.db_pool_auth)
            .await?;

    // 3. Verify password
    let password_hash = user_info.0;
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .context("Falha task verificar senha")?
            .map_err(|_| AppError::Unauthorized("Senha incorreta".to_string()))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Senha incorreta".to_string()));
    }

    // 4. Get username for TOTP
    let user = user_repo.find_by_id(current_user.id).await?.unwrap();

    // 5. Verify TOTP code
    let secret = user_info
        .1
        .ok_or_else(|| AppError::BadRequest("MFA secret não encontrado".to_string()))?;

    let secret_bytes = Secret::Encoded(secret)
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("Erro ao decodificar secret: {}", e))?;

    // FIXED: Include issuer and account_name
    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        user.username,
    )
    .map_err(|e| anyhow::anyhow!("Erro ao criar TOTP: {}", e))?;

    if !totp
        .check_current(&payload.totp_code)
        .map_err(|e| anyhow::anyhow!("Erro ao verificar TOTP: {}", e))?
    {
        return Err(AppError::BadRequest("Código TOTP inválido".to_string()));
    }

    // 6. Generate new backup codes
    let (backup_codes_plain, backup_codes_hashed) = generate_backup_codes();

    // 7. Update backup codes
    mfa_repo
        .update_backup_codes(current_user.id, &backup_codes_hashed)
        .await?;

    tracing::info!(user_id = %current_user.id, event_type = "mfa_backup_codes_regenerated", "Códigos de backup regenerados");

    Ok(Json(MfaBackupCodesResponse {
        backup_codes: backup_codes_plain,
        message: "Novos códigos de backup gerados. Os códigos anteriores foram invalidados."
            .to_string(),
    }))
}

/// GET /auth/mfa/status
/// Returns MFA status for the current user.
pub async fn handler_mfa_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MfaStatusResponse>, AppError> {
    let mfa_repo = MfaRepository::new(&state.db_pool_auth);

    let enabled = mfa_repo.is_mfa_enabled(current_user.id).await?;

    let backup_codes_remaining = if enabled {
        Some(mfa_repo.count_backup_codes(current_user.id).await? as usize)
    } else {
        None
    };

    Ok(Json(MfaStatusResponse {
        enabled,
        backup_codes_remaining,
    }))
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Generates a set of backup codes.
/// Returns (plain codes for user, hashed codes for storage).
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

/// Generates a single backup code (alphanumeric, 12 chars).
fn generate_backup_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Avoid confusing chars like 0/O, 1/I
    let mut rng = rand::thread_rng();

    (0..BACKUP_CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Hashes a backup code for secure storage.
fn hash_backup_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.to_uppercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generates an MFA challenge token (used during login).
pub fn generate_mfa_challenge_token(state: &AppState, user_id: Uuid) -> anyhow::Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let claims = MfaChallengeClaims {
        sub: user_id,
        exp: now + MFA_CHALLENGE_EXPIRY_SECONDS,
        iat: now,
        token_type: "mfa_challenge".to_string(),
    };

    let header = Header::new(Algorithm::EdDSA);
    let token = encode(&header, &claims, &state.encoding_key)
        .context("Falha ao gerar MFA challenge token")?;

    Ok(token)
}

/// Helper to generate access and refresh tokens (reused from auth_handler).
async fn generate_tokens(state: &AppState, user_id: Uuid) -> Result<(String, String), AppError> {
    use chrono::{Duration, Utc};
    use domain::models::TokenType;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 1. Generate Access Token
    let claims = Claims {
        sub: user_id,
        exp: now + 3600, // 1 hour
        iat: now,
        token_type: TokenType::Access,
    };

    let header = Header::new(Algorithm::EdDSA);
    let access_token =
        encode(&header, &claims, &state.encoding_key).context("Falha ao gerar JWT")?;

    // 2. Generate Refresh Token
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(refresh_token_raw.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    let family_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::seconds(604800); // 7 days

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
