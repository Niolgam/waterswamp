use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{UserDto, UserDtoExtended, UserLoginInfo};
use domain::ports::UserRepositoryPort;
use domain::value_objects::{Email, Username};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// REMOVIDO: Lifetime <'a> do impl e do Trait
#[async_trait]
impl UserRepositoryPort for UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, RepositoryError> {
        sqlx::query_as::<_, UserDto>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        // ALTERADO: &self.pool (referência para o owned) em vez de self.pool (que já era ref)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_extended_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<UserDtoExtended>, RepositoryError> {
        sqlx::query_as::<_, UserDtoExtended>(
            r#"
            SELECT
                id, username, email, role,
                email_verified, email_verified_at, mfa_enabled,
                is_banned, banned_at, banned_reason,
                created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn get_password_hash(&self, id: Uuid) -> Result<Option<String>, RepositoryError> {
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn mark_email_unverified(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE users SET email_verified = false, email_verified_at = NULL WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn find_by_username(
        &self,
        username: &Username,
    ) -> Result<Option<UserDto>, RepositoryError> {
        sqlx::query_as::<_, UserDto>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE username = $1",
        )
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<UserDto>, RepositoryError> {
        sqlx::query_as::<_, UserDto>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE LOWER(email) = LOWER($1)",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_for_login(&self, identifier: &str) -> Result<Option<UserLoginInfo>, RepositoryError> {
        sqlx::query_as::<_, UserLoginInfo>(
            "SELECT id, username, password_hash, mfa_enabled FROM users WHERE username = $1 OR LOWER(email) = LOWER($1)",
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1))")
            .bind(email.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_username(&self, username: &Username) -> Result<bool, RepositoryError> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(username.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_email_excluding(
        &self,
        email: &Email,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1) AND id != $2)",
        )
        .bind(email.as_str())
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_username_excluding(
        &self,
        username: &Username,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)")
            .bind(username.as_str())
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn create(
        &self,
        username: &Username,
        email: &Email,
        password_hash: &str,
    ) -> Result<UserDto, RepositoryError> {
        sqlx::query_as::<_, UserDto>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, created_at, updated_at
            "#,
        )
        .bind(username.as_str())
        .bind(email.as_str())
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_username(
        &self,
        id: Uuid,
        new_username: &Username,
    ) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_username.as_str())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn update_email(&self, id: Uuid, new_email: &Email) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_email.as_str())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: &str,
    ) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_password_hash)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn update_role(&self, id: Uuid, new_role: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_role)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn ban_user(&self, id: Uuid, reason: Option<&str>) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET is_banned = TRUE, banned_at = NOW(), banned_reason = $2, updated_at = NOW()
            WHERE id = $1 AND is_banned = FALSE
            "#,
        )
        .bind(id)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn unban_user(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET is_banned = FALSE, banned_at = NULL, banned_reason = NULL, updated_at = NOW()
            WHERE id = $1 AND is_banned = TRUE
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn is_banned(&self, id: Uuid) -> Result<bool, RepositoryError> {
        sqlx::query_scalar("SELECT is_banned FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
            .map(|opt| opt.unwrap_or(false))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<UserDto>, i64), RepositoryError> {
        let mut query_str =
            "SELECT id, username, email, created_at, updated_at FROM users".to_string();
        let mut count_str = "SELECT COUNT(*) FROM users".to_string();
        let mut params_count = 0;
        let mut conditions = Vec::new();

        if let Some(_) = search {
            params_count += 1;
            conditions.push(format!(
                "(username ILIKE ${0} OR email ILIKE ${0})",
                params_count
            ));
        }

        if !conditions.is_empty() {
            let where_clause = format!(" WHERE {}", conditions.join(" AND "));
            query_str.push_str(&where_clause);
            count_str.push_str(&where_clause);
        }

        query_str.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            params_count + 1,
            params_count + 2
        ));

        let mut query = sqlx::query_as::<_, UserDto>(&query_str);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_str);

        if let Some(s) = search {
            let search_fmt = format!("%{}%", s);
            query = query.bind(search_fmt.clone());
            count_query = count_query.bind(search_fmt);
        }

        query = query.bind(limit).bind(offset);

        let users = query.fetch_all(&self.pool).await.map_err(map_db_error)?;
        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        Ok((users, total))
    }
}
