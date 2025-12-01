use sqlx::PgPool;
use uuid::Uuid;

/// Repository for MFA/TOTP operations.
pub struct MfaRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> MfaRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Saves a temporary MFA setup token with the secret.
    pub async fn save_setup_token(
        &self,
        user_id: Uuid,
        secret: &str,
        expires_in_minutes: i64,
    ) -> Result<Uuid, sqlx::Error> {
        let setup_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO mfa_setup_tokens (user_id, secret, expires_at)
            VALUES ($1, $2, NOW() + ($3 || ' minutes')::INTERVAL)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(secret)
        .bind(expires_in_minutes.to_string())
        .fetch_one(self.pool)
        .await?;

        Ok(setup_id)
    }

    /// Finds a valid setup token and returns (user_id, secret).
    pub async fn find_valid_setup_token(
        &self,
        setup_id: Uuid,
    ) -> Result<Option<(Uuid, String)>, sqlx::Error> {
        let result: Option<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT user_id, secret
            FROM mfa_setup_tokens
            WHERE id = $1
              AND verified = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(setup_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(result)
    }

    /// Marks setup token as verified and deletes it.
    pub async fn complete_setup(&self, setup_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM mfa_setup_tokens WHERE id = $1")
            .bind(setup_id)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    /// Enables MFA for a user with the secret and backup codes.
    pub async fn enable_mfa(
        &self,
        user_id: Uuid,
        secret: &str,
        backup_codes_hashed: &[String],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET mfa_enabled = TRUE,
                mfa_secret = $2,
                mfa_backup_codes = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(secret)
        .bind(backup_codes_hashed)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Disables MFA for a user.
    pub async fn disable_mfa(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET mfa_enabled = FALSE,
                mfa_secret = NULL,
                mfa_backup_codes = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Gets MFA status and secret for a user.
    pub async fn get_mfa_info(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(bool, Option<String>, Option<Vec<String>>)>, sqlx::Error> {
        let result: Option<(bool, Option<String>, Option<Vec<String>>)> = sqlx::query_as(
            r#"
            SELECT mfa_enabled, mfa_secret, mfa_backup_codes
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(result)
    }

    /// Checks if MFA is enabled for a user.
    pub async fn is_mfa_enabled(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT mfa_enabled FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.pool)
            .await
    }

    /// Gets the MFA secret for TOTP verification.
    pub async fn get_mfa_secret(&self, user_id: Uuid) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT mfa_secret FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.pool)
            .await
    }

    /// Gets backup codes for a user.
    pub async fn get_backup_codes(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Vec<String>>, sqlx::Error> {
        sqlx::query_scalar("SELECT mfa_backup_codes FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.pool)
            .await
    }

    /// Removes a used backup code from the array.
    pub async fn remove_backup_code(
        &self,
        user_id: Uuid,
        code_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET mfa_backup_codes = array_remove(mfa_backup_codes, $2),
                updated_at = NOW()
            WHERE id = $1 AND $2 = ANY(mfa_backup_codes)
            "#,
        )
        .bind(user_id)
        .bind(code_hash)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Records backup code usage for audit.
    pub async fn record_backup_code_usage(
        &self,
        user_id: Uuid,
        code_hash: &str,
        ip_address: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO mfa_backup_code_usage (user_id, code_hash, ip_address)
            VALUES ($1, $2, $3::INET)
            "#,
        )
        .bind(user_id)
        .bind(code_hash)
        .bind(ip_address)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Updates backup codes (for regeneration).
    pub async fn update_backup_codes(
        &self,
        user_id: Uuid,
        backup_codes_hashed: &[String],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET mfa_backup_codes = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(backup_codes_hashed)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Counts remaining backup codes.
    pub async fn count_backup_codes(&self, user_id: Uuid) -> Result<usize, sqlx::Error> {
        let codes: Option<Vec<String>> =
            sqlx::query_scalar("SELECT mfa_backup_codes FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(self.pool)
                .await?;

        Ok(codes.map(|c| c.len()).unwrap_or(0))
    }

    /// Cleans up expired setup tokens (for maintenance jobs).
    pub async fn cleanup_expired_setup_tokens(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM mfa_setup_tokens WHERE expires_at < NOW()")
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
