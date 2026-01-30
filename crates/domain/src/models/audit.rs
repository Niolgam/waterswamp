use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx;
use uuid::Uuid;
use validator::Validate;

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
