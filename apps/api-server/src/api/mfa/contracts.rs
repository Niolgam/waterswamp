use serde::{Deserialize, Serialize};
use validator::Validate;

// --- Setup ---

#[derive(Debug, Serialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub setup_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct MfaVerifySetupRequest {
    #[validate(length(min = 1, message = "Setup token não pode estar vazio"))]
    pub setup_token: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaSetupCompleteResponse {
    pub enabled: bool,
    pub backup_codes: Vec<String>,
    pub message: String,
}

// --- Verify (Login Step 2) ---

#[derive(Debug, Deserialize, Validate)]
pub struct MfaVerifyRequest {
    #[validate(length(min = 1, message = "MFA token não pode estar vazio"))]
    pub mfa_token: String,
    #[validate(length(min = 6, max = 12, message = "Código deve ter entre 6 e 12 caracteres"))]
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaVerifyResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub backup_code_used: bool,
}

// --- Management ---

#[derive(Debug, Deserialize, Validate)]
pub struct MfaDisableRequest {
    #[validate(length(min = 1, message = "Senha não pode estar vazia"))]
    pub password: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaDisableResponse {
    pub disabled: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct MfaRegenerateBackupCodesRequest {
    #[validate(length(min = 1, message = "Senha não pode estar vazia"))]
    pub password: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaBackupCodesResponse {
    pub backup_codes: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MfaStatusResponse {
    pub enabled: bool,
    pub backup_codes_remaining: Option<usize>,
}
