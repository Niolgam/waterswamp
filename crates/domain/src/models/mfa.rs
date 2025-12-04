use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub setup_token: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaVerifySetupPayload {
    #[validate(length(min = 1))]
    pub setup_token: String,
    #[validate(length(equal = 6))]
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
    #[validate(length(min = 1))]
    pub mfa_token: String,
    #[validate(length(min = 6, max = 12))]
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

impl MfaVerifyResponse {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        backup_code_used: bool,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            backup_code_used,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaDisablePayload {
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(length(equal = 6))]
    pub totp_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaDisableResponse {
    pub disabled: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaRegenerateBackupCodesPayload {
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(length(equal = 6))]
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
