use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core_services::field_encryption;
use domain::errors::RepositoryError;
use domain::ports::MfaRepositoryPort;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

#[derive(Clone)]
pub struct MfaRepository {
    pool: PgPool,
    encryption_key: [u8; 32],
}

impl MfaRepository {
    pub fn new(pool: PgPool, encryption_key: [u8; 32]) -> Self {
        Self {
            pool,
            encryption_key,
        }
    }

    fn encrypt(&self, plaintext: &str) -> Result<String, RepositoryError> {
        field_encryption::encrypt_field(plaintext, &self.encryption_key)
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, RepositoryError> {
        field_encryption::decrypt_field(ciphertext, &self.encryption_key)
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))
    }
}

#[async_trait]
impl MfaRepositoryPort for MfaRepository {
    async fn save_setup_token(
        &self,
        user_id: Uuid,
        secret: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        let secret_encrypted = self.encrypt(secret)?;

        sqlx::query(
            r#"
            INSERT INTO mfa_setup_tokens (user_id, secret, token_hash, expires_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(secret_encrypted)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    async fn find_setup_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<(Uuid, String)>, RepositoryError> {
        let result: Option<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT user_id, secret
            FROM mfa_setup_tokens
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        match result {
            None => Ok(None),
            Some((user_id, secret_encrypted)) => {
                let secret = self.decrypt(&secret_encrypted)?;
                Ok(Some((user_id, secret)))
            }
        }
    }

    async fn delete_setup_token(&self, token_hash: &str) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM mfa_setup_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn enable_mfa(&self, user_id: Uuid, secret: &str) -> Result<(), RepositoryError> {
        let secret_encrypted = self.encrypt(secret)?;
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = $1 WHERE id = $2")
            .bind(secret_encrypted)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        sqlx::query("DELETE FROM mfa_setup_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)?;
        Ok(())
    }

    async fn disable_mfa(&self, user_id: Uuid) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        sqlx::query("DELETE FROM mfa_backup_codes WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)?;
        Ok(())
    }

    async fn save_backup_codes(
        &self,
        user_id: Uuid,
        codes: &[String],
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        sqlx::query("DELETE FROM mfa_backup_codes WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        for code in codes {
            sqlx::query("INSERT INTO mfa_backup_codes (user_id, code_hash) VALUES ($1, $2)")
                .bind(user_id)
                .bind(code)
                .execute(&mut *tx)
                .await
                .map_err(map_db_error)?;
        }

        tx.commit().await.map_err(map_db_error)?;
        Ok(())
    }

    async fn get_backup_codes(&self, user_id: Uuid) -> Result<Vec<String>, RepositoryError> {
        let codes: Vec<(String,)> = sqlx::query_as(
            "SELECT code_hash FROM mfa_backup_codes WHERE user_id = $1 AND used = FALSE",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(codes.into_iter().map(|(c,)| c).collect())
    }

    async fn verify_and_consume_backup_code(
        &self,
        user_id: Uuid,
        code_hash: &str,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            "UPDATE mfa_backup_codes SET used = TRUE, used_at = NOW() \
             WHERE user_id = $1 AND code_hash = $2 AND used = FALSE",
        )
        .bind(user_id)
        .bind(code_hash)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_mfa_secret(&self, user_id: Uuid) -> Result<Option<String>, RepositoryError> {
        let raw: Option<String> =
            sqlx::query_scalar("SELECT mfa_secret FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_error)?;

        match raw {
            None => Ok(None),
            Some(encrypted) => Ok(Some(self.decrypt(&encrypted)?)),
        }
    }

    async fn is_mfa_enabled(&self, user_id: Uuid) -> Result<bool, RepositoryError> {
        sqlx::query_scalar("SELECT mfa_enabled FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
            .map(|opt| opt.unwrap_or(false))
    }
}
