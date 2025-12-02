use chrono::{DateTime, Utc};
use lazy_static;
use regex;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{Email, Username};

lazy_static::lazy_static! {
    static ref ROLE_REGEX: Regex =
        Regex::new(r"^(admin|user)$").unwrap();
}

// =============================================================================
// AUTH
// =============================================================================

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct LoginPayload {
    #[validate(length(min = 3))]
    pub username: String, // Login aceita string genérica (pode ser email ou username)
    #[validate(length(min = 6))]
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
    #[validate(length(min = 1))]
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
// USER DTOs (AQUI ESTAVA O PROBLEMA DE TIPO)
// =============================================================================

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RegisterPayload {
    // Validação automática via TryFrom no tipo
    pub username: Username,
    pub email: Email,

    #[validate(length(min = 8))]
    pub password: String,
}

/// DTO de usuário (sem informações sensíveis)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserDto {
    pub id: Uuid,
    pub username: Username, // Mudou de String para Username
    pub email: Email,       // Mudou de String para Email
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO estendido com informações de verificação e role
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserDtoExtended {
    pub id: Uuid,
    pub username: Username, // Mudou de String para Username
    pub email: Email,       // Mudou de String para Email
    pub role: String,
    pub email_verified: bool,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub mfa_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO detalhado do usuário, incluindo papéis
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailDto {
    #[serde(flatten)]
    pub user: UserDto,
    pub roles: Vec<String>,
}

// =============================================================================
// JWT CLAIMS & OUTROS
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

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PolicyRequest {
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub object: String,
    #[validate(length(min = 1))]
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserPayload {
    pub username: Username,
    pub email: Email,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1))]
    pub role: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUserPayload {
    pub username: Option<Username>,
    pub email: Option<Email>,
    #[validate(length(min = 8))]
    pub password: Option<String>,
    #[validate(length(min = 1))]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ForgotPasswordPayload {
    pub email: Email, // Mudou de String para Email
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResetPasswordPayload {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(length(min = 10))]
    pub new_password: String,
}

// =============================================================================
// EMAIL VERIFICATION & MFA MODELS
// =============================================================================

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResendVerificationPayload {
    pub email: Email, // Mudou de String para Email
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct VerifyEmailPayload {
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationResponse {
    pub verified: bool,
    pub message: String,
}

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
// =============================================================================
// AUDIT LOG - Core Types
// =============================================================================

/// Types of auditable actions in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Authentication actions
    Login,
    LoginFailed,
    Logout,
    TokenRefresh,
    PasswordReset,
    PasswordResetRequest,
    PasswordChange,

    // Email verification
    EmailVerificationSent,
    EmailVerified,

    // MFA actions
    MfaEnabled,
    MfaDisabled,
    MfaVerified,
    MfaFailed,
    MfaBackupCodeUsed,
    MfaBackupCodesRegenerated,

    // User management
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserRoleChanged,

    // Policy/Permission management
    PolicyAdded,
    PolicyRemoved,

    // Admin actions
    AdminAccessGranted,
    AdminAccessDenied,

    // Generic actions
    ResourceAccess,
    Custom(String),
}

impl AuditAction {
    pub fn as_str(&self) -> String {
        match self {
            AuditAction::Login => "login".to_string(),
            AuditAction::LoginFailed => "login_failed".to_string(),
            AuditAction::Logout => "logout".to_string(),
            AuditAction::TokenRefresh => "token_refresh".to_string(),
            AuditAction::PasswordReset => "password_reset".to_string(),
            AuditAction::PasswordResetRequest => "password_reset_request".to_string(),
            AuditAction::PasswordChange => "password_change".to_string(),
            AuditAction::EmailVerificationSent => "email_verification_sent".to_string(),
            AuditAction::EmailVerified => "email_verified".to_string(),
            AuditAction::MfaEnabled => "mfa_enabled".to_string(),
            AuditAction::MfaDisabled => "mfa_disabled".to_string(),
            AuditAction::MfaVerified => "mfa_verified".to_string(),
            AuditAction::MfaFailed => "mfa_failed".to_string(),
            AuditAction::MfaBackupCodeUsed => "mfa_backup_code_used".to_string(),
            AuditAction::MfaBackupCodesRegenerated => "mfa_backup_codes_regenerated".to_string(),
            AuditAction::UserCreated => "user_created".to_string(),
            AuditAction::UserUpdated => "user_updated".to_string(),
            AuditAction::UserDeleted => "user_deleted".to_string(),
            AuditAction::UserRoleChanged => "user_role_changed".to_string(),
            AuditAction::PolicyAdded => "policy_added".to_string(),
            AuditAction::PolicyRemoved => "policy_removed".to_string(),
            AuditAction::AdminAccessGranted => "admin_access_granted".to_string(),
            AuditAction::AdminAccessDenied => "admin_access_denied".to_string(),
            AuditAction::ResourceAccess => "resource_access".to_string(),
            AuditAction::Custom(s) => s.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "login" => AuditAction::Login,
            "login_failed" => AuditAction::LoginFailed,
            "logout" => AuditAction::Logout,
            "token_refresh" => AuditAction::TokenRefresh,
            "password_reset" => AuditAction::PasswordReset,
            "password_reset_request" => AuditAction::PasswordResetRequest,
            "password_change" => AuditAction::PasswordChange,
            "email_verification_sent" => AuditAction::EmailVerificationSent,
            "email_verified" => AuditAction::EmailVerified,
            "mfa_enabled" => AuditAction::MfaEnabled,
            "mfa_disabled" => AuditAction::MfaDisabled,
            "mfa_verified" => AuditAction::MfaVerified,
            "mfa_failed" => AuditAction::MfaFailed,
            "mfa_backup_code_used" => AuditAction::MfaBackupCodeUsed,
            "mfa_backup_codes_regenerated" => AuditAction::MfaBackupCodesRegenerated,
            "user_created" => AuditAction::UserCreated,
            "user_updated" => AuditAction::UserUpdated,
            "user_deleted" => AuditAction::UserDeleted,
            "user_role_changed" => AuditAction::UserRoleChanged,
            "policy_added" => AuditAction::PolicyAdded,
            "policy_removed" => AuditAction::PolicyRemoved,
            "admin_access_granted" => AuditAction::AdminAccessGranted,
            "admin_access_denied" => AuditAction::AdminAccessDenied,
            "resource_access" => AuditAction::ResourceAccess,
            other => AuditAction::Custom(other.to_string()),
        }
    }
}

/// Represents an audit log entry in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub action: String,
    pub resource: String,
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLogDto {
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub action: AuditAction,
    pub resource: String,
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub duration_ms: Option<i32>,
}

impl CreateAuditLogDto {
    pub fn new(action: AuditAction, resource: String) -> Self {
        Self {
            user_id: None,
            username: None,
            action,
            resource,
            method: None,
            status_code: None,
            details: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
            duration_ms: None,
        }
    }

    pub fn with_user(mut self, user_id: Uuid, username: String) -> Self {
        self.user_id = Some(user_id);
        self.username = Some(username);
        self
    }

    pub fn with_request_info(
        mut self,
        method: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.method = Some(method);
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    pub fn with_response(mut self, status_code: i32, duration_ms: i32) -> Self {
        self.status_code = Some(status_code);
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

// =============================================================================
// AUDIT LOG - Query Parameters
// =============================================================================

/// Query parameters for listing audit logs
#[derive(Debug, Deserialize, Validate)]
pub struct ListAuditLogsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,

    #[validate(range(min = 0))]
    pub offset: Option<i64>,

    /// Filter by user ID
    pub user_id: Option<Uuid>,

    /// Filter by action type
    pub action: Option<String>,

    /// Filter by resource (supports wildcards)
    pub resource: Option<String>,

    /// Filter by IP address
    pub ip_address: Option<String>,

    /// Filter by status code (e.g., 401, 403, 500)
    pub status_code: Option<i32>,

    /// Filter by minimum status code (e.g., >= 400 for errors)
    pub min_status_code: Option<i32>,

    /// Filter by maximum status code
    pub max_status_code: Option<i32>,

    /// Start date for date range filter (ISO 8601)
    pub start_date: Option<DateTime<Utc>>,

    /// End date for date range filter (ISO 8601)
    pub end_date: Option<DateTime<Utc>>,

    /// Sort order: "asc" or "desc" (default: desc)
    pub sort_order: Option<String>,

    /// Sort by field: "created_at", "action", "user_id" (default: created_at)
    pub sort_by: Option<String>,
}

/// Paginated response for audit logs
#[derive(Debug, Serialize)]
pub struct PaginatedAuditLogs {
    pub logs: Vec<AuditLogEntry>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Summary statistics for audit logs
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogStats {
    pub total_logs: i64,
    pub logs_today: i64,
    pub logs_this_week: i64,
    pub failed_logins_today: i64,
    pub unique_users_today: i64,
    pub top_actions: Vec<ActionCount>,
    pub top_resources: Vec<ResourceCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionCount {
    pub action: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceCount {
    pub resource: String,
    pub count: i64,
}
