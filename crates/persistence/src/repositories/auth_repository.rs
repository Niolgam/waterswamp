use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::errors::RepositoryError;
use domain::models::RefreshToken;
use domain::ports::AuthRepositoryPort;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

#[derive(Clone)]
pub struct AuthRepository {
    pool: PgPool,
}

impl AuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRepositoryPort for AuthRepository {
    async fn save_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        family_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(family_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    async fn find_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError> {
        sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM refresh_tokens WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    // Método atômico para rotação: Revoga o antigo E insere o novo
    async fn rotate_refresh_token(
        &self,
        old_token_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
        family_id: Uuid,
        parent_hash: &str,
        user_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        // 1. Revogar antigo
        sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE id = $1")
            .bind(old_token_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        // 2. Inserir novo
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id, parent_token_hash)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(new_token_hash)
        .bind(new_expires_at)
        .bind(family_id)
        .bind(parent_hash)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)?;
        Ok(())
    }

    async fn revoke_token_family(&self, family_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE family_id = $1")
            .bind(family_id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn revoke_token(&self, token_hash: &str) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, updated_at = NOW()
            WHERE token_hash = $1 AND revoked = FALSE
            "#,
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, updated_at = NOW()
            WHERE user_id = $1 AND revoked = FALSE
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }
}
