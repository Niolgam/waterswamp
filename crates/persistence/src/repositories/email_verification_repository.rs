use sqlx::PgPool;
use uuid::Uuid;

/// Repository for email verification operations.
pub struct EmailVerificationRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> EmailVerificationRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Saves a new email verification token.
    pub async fn save_verification_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_in_hours: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, NOW() + ($3 || ' hours')::INTERVAL)
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_in_hours.to_string())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Finds a valid (not used, not expired) verification token.
    pub async fn find_valid_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let result: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT user_id
            FROM email_verification_tokens
            WHERE token_hash = $1
              AND used = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.pool)
        .await?;

        Ok(result.map(|(user_id,)| user_id))
    }

    /// Marks a verification token as used.
    pub async fn mark_token_as_used(&self, token_hash: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET used = TRUE
            WHERE token_hash = $1 AND used = FALSE
            "#,
        )
        .bind(token_hash)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Marks user's email as verified.
    pub async fn verify_user_email(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = TRUE,
                email_verified_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Checks if user's email is already verified.
    pub async fn is_email_verified(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT email_verified FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.pool)
            .await
    }

    /// Gets the count of recent verification attempts for rate limiting.
    pub async fn count_recent_verification_requests(
        &self,
        user_id: Uuid,
        minutes: i64,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM email_verification_tokens
            WHERE user_id = $1
              AND created_at > NOW() - ($2 || ' minutes')::INTERVAL
            "#,
        )
        .bind(user_id)
        .bind(minutes.to_string())
        .fetch_one(self.pool)
        .await
    }

    /// Invalidates all pending verification tokens for a user.
    pub async fn invalidate_all_tokens(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET used = TRUE
            WHERE user_id = $1 AND used = FALSE
            "#,
        )
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Verifies email atomically: finds token, marks as used, and updates user.
    /// This ensures ACID compliance - either all operations succeed or none do.
    pub async fn verify_email_atomic(&self, token_hash: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 1. Find and lock the token (SELECT FOR UPDATE prevents race conditions)
        let result: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT user_id
            FROM email_verification_tokens
            WHERE token_hash = $1
              AND used = FALSE
              AND expires_at > NOW()
            FOR UPDATE
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&mut *tx)
        .await?;

        let user_id = match result {
            Some((uid,)) => uid,
            None => {
                tx.rollback().await?;
                return Ok(None);
            }
        };

        // 2. Mark token as used
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET used = TRUE
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .execute(&mut *tx)
        .await?;

        // 3. Mark user email as verified
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = TRUE,
                email_verified_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // 4. Invalidate all other pending tokens for this user
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
            SET used = TRUE
            WHERE user_id = $1 AND used = FALSE
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Some(user_id))
    }
}
