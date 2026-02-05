use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a user session stored in the database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub session_token_hash: String,
    pub user_id: Uuid,
    pub user_agent: Option<String>,
    /// IP address stored as string (from INET column cast to TEXT)
    pub ip_address: Option<String>,
    pub access_token_encrypted: String,
    pub refresh_token_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub csrf_token_hash: String,
}

/// Data required to create a new session
#[derive(Debug, Clone)]
pub struct CreateSession {
    pub session_token_hash: String,
    pub user_id: Uuid,
    pub user_agent: Option<String>,
    /// IP address as string (will be cast to INET in SQL)
    pub ip_address: Option<String>,
    pub access_token_encrypted: String,
    pub refresh_token_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub csrf_token_hash: String,
}

/// Session with user information for middleware
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub access_token: String, // Decrypted
    pub expires_at: DateTime<Utc>,
    pub csrf_token_hash: String,
}

/// Summary of a user's session for listing
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SessionSummary {
    pub id: Uuid,
    pub user_agent: Option<String>,
    /// IP address as string
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub is_current: bool,
}

/// Represents a session encryption/signing key
#[derive(Debug, Clone, FromRow)]
pub struct SessionKey {
    pub id: Uuid,
    pub key_id: String,
    pub key_material: Vec<u8>,
    pub key_type: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Reason for session revocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRevocationReason {
    /// User logged out manually
    UserLogout,
    /// User logged out from all devices
    UserLogoutAll,
    /// Session expired
    Expired,
    /// Security concern (e.g., password change)
    SecurityConcern,
    /// Admin revoked the session
    AdminRevoked,
    /// Suspicious activity detected
    SuspiciousActivity,
}

impl SessionRevocationReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserLogout => "user_logout",
            Self::UserLogoutAll => "user_logout_all",
            Self::Expired => "expired",
            Self::SecurityConcern => "security_concern",
            Self::AdminRevoked => "admin_revoked",
            Self::SuspiciousActivity => "suspicious_activity",
        }
    }
}

impl std::fmt::Display for SessionRevocationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
