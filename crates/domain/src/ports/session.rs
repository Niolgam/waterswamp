use crate::errors::RepositoryError;
use crate::models::{CreateSession, Session, SessionKey, SessionRevocationReason, SessionSummary};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for session management
#[async_trait]
pub trait SessionRepositoryPort: Send + Sync {
    /// Creates a new session in the database
    async fn create_session(&self, session: CreateSession) -> Result<Session, RepositoryError>;

    /// Finds a session by its token hash (used for cookie validation)
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, RepositoryError>;

    /// Updates the last activity timestamp (sliding expiration)
    async fn touch_session(
        &self,
        session_id: Uuid,
        extend_minutes: Option<i32>,
    ) -> Result<bool, RepositoryError>;

    /// Revokes a specific session
    async fn revoke_session(
        &self,
        session_id: Uuid,
        reason: SessionRevocationReason,
    ) -> Result<bool, RepositoryError>;

    /// Revokes a session by its token hash
    async fn revoke_session_by_token(
        &self,
        token_hash: &str,
        reason: SessionRevocationReason,
    ) -> Result<bool, RepositoryError>;

    /// Revokes all sessions for a user (logout everywhere)
    async fn revoke_all_user_sessions(
        &self,
        user_id: Uuid,
        reason: SessionRevocationReason,
    ) -> Result<u64, RepositoryError>;

    /// Lists all active sessions for a user
    async fn list_user_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Option<Uuid>,
    ) -> Result<Vec<SessionSummary>, RepositoryError>;

    /// Cleans up expired and old revoked sessions
    async fn cleanup_expired_sessions(&self) -> Result<u64, RepositoryError>;

    /// Gets the active signing key for cookies
    async fn get_active_signing_key(&self) -> Result<Option<SessionKey>, RepositoryError>;

    /// Gets the active encryption key for cookies
    async fn get_active_encryption_key(&self) -> Result<Option<SessionKey>, RepositoryError>;

    /// Gets a key by its ID (for verification with older keys)
    async fn get_key_by_id(&self, key_id: &str) -> Result<Option<SessionKey>, RepositoryError>;

    /// Creates a new session key (for rotation)
    async fn create_session_key(
        &self,
        key_id: &str,
        key_material: &[u8],
        key_type: &str,
    ) -> Result<SessionKey, RepositoryError>;

    /// Rotates to a new key (deactivates old, activates new)
    async fn rotate_key(
        &self,
        old_key_id: Uuid,
        new_key_id: &str,
        new_key_material: &[u8],
        key_type: &str,
        reason: &str,
    ) -> Result<SessionKey, RepositoryError>;

    /// Counts active sessions by IP (for rate limiting/security)
    async fn count_sessions_by_ip(&self, ip: &str) -> Result<u64, RepositoryError>;

    /// Updates the encrypted access token in a session (token refresh)
    async fn update_session_access_token(
        &self,
        session_id: Uuid,
        new_access_token_encrypted: &str,
        new_refresh_token_id: Option<Uuid>,
    ) -> Result<bool, RepositoryError>;
}
