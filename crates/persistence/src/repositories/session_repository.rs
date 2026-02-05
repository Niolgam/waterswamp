use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{CreateSession, Session, SessionKey, SessionRevocationReason, SessionSummary};
use domain::ports::SessionRepositoryPort;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

#[derive(Clone)]
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepositoryPort for SessionRepository {
    async fn create_session(&self, session: CreateSession) -> Result<Session, RepositoryError> {
        let result = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (
                session_token_hash,
                user_id,
                user_agent,
                ip_address,
                access_token_encrypted,
                refresh_token_id,
                expires_at,
                csrf_token_hash
            )
            VALUES ($1, $2, $3, $4::INET, $5, $6, $7, $8)
            RETURNING id, session_token_hash, user_id, user_agent,
                      ip_address::TEXT as ip_address, access_token_encrypted,
                      refresh_token_id, created_at, expires_at, last_activity_at,
                      is_revoked, revoked_at, revoked_reason, csrf_token_hash
            "#,
        )
        .bind(&session.session_token_hash)
        .bind(session.user_id)
        .bind(&session.user_agent)
        .bind(&session.ip_address)
        .bind(&session.access_token_encrypted)
        .bind(session.refresh_token_id)
        .bind(session.expires_at)
        .bind(&session.csrf_token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, RepositoryError> {
        let result = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, session_token_hash, user_id, user_agent,
                   ip_address::TEXT as ip_address, access_token_encrypted,
                   refresh_token_id, created_at, expires_at, last_activity_at,
                   is_revoked, revoked_at, revoked_reason, csrf_token_hash
            FROM sessions
            WHERE session_token_hash = $1
              AND is_revoked = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn touch_session(
        &self,
        session_id: Uuid,
        extend_minutes: Option<i32>,
    ) -> Result<bool, RepositoryError> {
        let extend = extend_minutes.unwrap_or(30);

        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET last_activity_at = NOW(),
                expires_at = GREATEST(expires_at, NOW() + ($2 || ' minutes')::INTERVAL)
            WHERE id = $1
              AND is_revoked = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(session_id)
        .bind(extend.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_session(
        &self,
        session_id: Uuid,
        reason: SessionRevocationReason,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET is_revoked = TRUE,
                revoked_at = NOW(),
                revoked_reason = $2
            WHERE id = $1
              AND is_revoked = FALSE
            "#,
        )
        .bind(session_id)
        .bind(reason.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_session_by_token(
        &self,
        token_hash: &str,
        reason: SessionRevocationReason,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET is_revoked = TRUE,
                revoked_at = NOW(),
                revoked_reason = $2
            WHERE session_token_hash = $1
              AND is_revoked = FALSE
            "#,
        )
        .bind(token_hash)
        .bind(reason.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_all_user_sessions(
        &self,
        user_id: Uuid,
        reason: SessionRevocationReason,
    ) -> Result<u64, RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET is_revoked = TRUE,
                revoked_at = NOW(),
                revoked_reason = $2
            WHERE user_id = $1
              AND is_revoked = FALSE
            "#,
        )
        .bind(user_id)
        .bind(reason.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected())
    }

    async fn list_user_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Option<Uuid>,
    ) -> Result<Vec<SessionSummary>, RepositoryError> {
        let result = sqlx::query_as::<_, SessionSummary>(
            r#"
            SELECT
                id,
                user_agent,
                ip_address::TEXT as ip_address,
                created_at,
                last_activity_at,
                (id = $2) as is_current
            FROM sessions
            WHERE user_id = $1
              AND is_revoked = FALSE
              AND expires_at > NOW()
            ORDER BY last_activity_at DESC
            "#,
        )
        .bind(user_id)
        .bind(current_session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn cleanup_expired_sessions(&self) -> Result<u64, RepositoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE expires_at < NOW()
               OR (is_revoked = TRUE AND revoked_at < NOW() - INTERVAL '7 days')
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected())
    }

    async fn get_active_signing_key(&self) -> Result<Option<SessionKey>, RepositoryError> {
        let result = sqlx::query_as::<_, SessionKey>(
            r#"
            SELECT * FROM session_keys
            WHERE key_type = 'signing'
              AND is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn get_active_encryption_key(&self) -> Result<Option<SessionKey>, RepositoryError> {
        let result = sqlx::query_as::<_, SessionKey>(
            r#"
            SELECT * FROM session_keys
            WHERE key_type = 'encryption'
              AND is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn get_key_by_id(&self, key_id: &str) -> Result<Option<SessionKey>, RepositoryError> {
        let result = sqlx::query_as::<_, SessionKey>(
            r#"
            SELECT * FROM session_keys
            WHERE key_id = $1
            "#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn create_session_key(
        &self,
        key_id: &str,
        key_material: &[u8],
        key_type: &str,
    ) -> Result<SessionKey, RepositoryError> {
        // First, deactivate any existing active key of this type
        sqlx::query(
            r#"
            UPDATE session_keys
            SET is_active = FALSE
            WHERE key_type = $1 AND is_active = TRUE
            "#,
        )
        .bind(key_type)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        let result = sqlx::query_as::<_, SessionKey>(
            r#"
            INSERT INTO session_keys (key_id, key_material, key_type, is_active)
            VALUES ($1, $2, $3, TRUE)
            RETURNING *
            "#,
        )
        .bind(key_id)
        .bind(key_material)
        .bind(key_type)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn rotate_key(
        &self,
        old_key_id: Uuid,
        new_key_id: &str,
        new_key_material: &[u8],
        key_type: &str,
        reason: &str,
    ) -> Result<SessionKey, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        // Deactivate old key
        sqlx::query(
            r#"
            UPDATE session_keys
            SET is_active = FALSE
            WHERE id = $1
            "#,
        )
        .bind(old_key_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        // Create new key with reference to old one
        let result = sqlx::query_as::<_, SessionKey>(
            r#"
            INSERT INTO session_keys (key_id, key_material, key_type, is_active, rotated_from_id, rotation_reason)
            VALUES ($1, $2, $3, TRUE, $4, $5)
            RETURNING *
            "#,
        )
        .bind(new_key_id)
        .bind(new_key_material)
        .bind(key_type)
        .bind(old_key_id)
        .bind(reason)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)?;

        Ok(result)
    }

    async fn count_sessions_by_ip(&self, ip: &str) -> Result<u64, RepositoryError> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM sessions
            WHERE ip_address = $1::INET
              AND is_revoked = FALSE
              AND expires_at > NOW()
              AND created_at > NOW() - INTERVAL '1 hour'
            "#,
        )
        .bind(ip)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.0 as u64)
    }

    async fn update_session_access_token(
        &self,
        session_id: Uuid,
        new_access_token_encrypted: &str,
        new_refresh_token_id: Option<Uuid>,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET access_token_encrypted = $2,
                refresh_token_id = COALESCE($3, refresh_token_id),
                last_activity_at = NOW()
            WHERE id = $1
              AND is_revoked = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(session_id)
        .bind(new_access_token_encrypted)
        .bind(new_refresh_token_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected() > 0)
    }
}
