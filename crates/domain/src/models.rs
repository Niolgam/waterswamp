use chrono;
use lazy_static;
use regex;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx;
use uuid::Uuid;
use validator::Validate;

lazy_static::lazy_static! {
    static ref ROLE_REGEX: Regex =
        Regex::new(r"^(admin|user)$").unwrap();
}

// =============================================================================
// AUTH - Payloads e Respostas
// =============================================================================

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct LoginPayload {
    #[validate(length(min = 3, message = "Username muito curto"))]
    pub username: String,
    #[validate(length(min = 6, message = "Senha muito curta"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl LoginResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RefreshTokenPayload {
    #[validate(length(min = 1, message = "Refresh token não pode estar vazio"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl RefreshTokenResponse {
    pub fn new(access_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

// =============================================================================
// USER
// =============================================================================

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RegisterPayload {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

// =============================================================================
// JWT CLAIMS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
    PasswordReset,
}

// =============================================================================
// POLICIES (CASBIN)
// =============================================================================

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PolicyRequest {
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub object: String,
    #[validate(length(min = 1))]
    pub action: String,
}

// =============================================================================
// ADMIN - User CRUD
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

/// DTO de usuário (sem informações sensíveis)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO detalhado do usuário, incluindo papéis
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailDto {
    #[serde(flatten)]
    pub user: UserDto,
    /// Lista de papéis (ex: "admin", "user") atribuídos a este usuário
    pub roles: Vec<String>,
}

/// Resposta paginada de usuários
#[derive(Debug, Serialize)]
pub struct PaginatedUsers {
    pub users: Vec<UserDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Payload para criar usuário (admin)
#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserPayload {
    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(
        length(min = 1),
        regex(path = *ROLE_REGEX, message = "O papel deve ser 'admin' ou 'user'")
    )]
    pub role: String,
}

/// Payload para atualizar usuário
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUserPayload {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,

    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,

    #[validate(length(min = 8))]
    pub password: Option<String>,

    #[validate(
        length(min = 1),
        regex(path = *ROLE_REGEX, message = "O papel deve ser 'admin' ou 'user'")
    )]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ForgotPasswordPayload {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResetPasswordPayload {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(length(min = 10))]
    pub new_password: String,
}

// =============================================================================
// EMAIL VERIFICATION MODELS (Task 7)
// =============================================================================

/// Payload for resending verification email
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResendVerificationPayload {
    #[validate(email(message = "Email inválido"))]
    pub email: String,
}

/// Payload for verifying email with token
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct VerifyEmailPayload {
    #[validate(length(min = 1, message = "Token não pode estar vazio"))]
    pub token: String,
}

/// Response after email verification
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationResponse {
    pub verified: bool,
    pub message: String,
}

/// Extended User DTO with email verification status
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserDtoExtended {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub mfa_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// MFA/TOTP MODELS (Task 8)
// =============================================================================

/// Response for MFA setup initiation
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub setup_token: String,
}

/// Payload for verifying MFA setup
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaVerifySetupPayload {
    #[validate(length(min = 1, message = "Setup token não pode estar vazio"))]
    pub setup_token: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

/// Response after successful MFA setup
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupCompleteResponse {
    pub enabled: bool,
    pub backup_codes: Vec<String>,
    pub message: String,
}

/// Payload for MFA verification during login
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaVerifyPayload {
    #[validate(length(min = 1, message = "MFA token não pode estar vazio"))]
    pub mfa_token: String,
    #[validate(length(min = 6, max = 12, message = "Código deve ter entre 6 e 12 caracteres"))]
    pub code: String,
}

/// Login response when MFA is required
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub mfa_token: String,
    pub message: String,
}

/// Response after successful MFA verification
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

/// Payload for disabling MFA
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaDisablePayload {
    #[validate(length(min = 1, message = "Senha não pode estar vazia"))]
    pub password: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

/// Response after disabling MFA
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaDisableResponse {
    pub disabled: bool,
    pub message: String,
}

/// Payload for regenerating backup codes
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MfaRegenerateBackupCodesPayload {
    #[validate(length(min = 1, message = "Senha não pode estar vazia"))]
    pub password: String,
    #[validate(length(equal = 6, message = "Código TOTP deve ter 6 dígitos"))]
    pub totp_code: String,
}

/// Response with new backup codes
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaBackupCodesResponse {
    pub backup_codes: Vec<String>,
    pub message: String,
}

/// MFA status for user
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaStatusResponse {
    pub enabled: bool,
    pub backup_codes_remaining: Option<usize>,
}

/// Claims for MFA challenge token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaChallengeClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}
