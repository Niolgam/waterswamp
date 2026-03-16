use async_trait::async_trait;
use core_services::field_encryption;
use domain::errors::RepositoryError;
use domain::models::{UserDto, UserDtoExtended, UserLoginInfo};
use domain::ports::UserRepositoryPort;
use domain::value_objects::{Email, Username};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ---------------------------------------------------------------------------
// Raw DB row types — email column contains AES-256-GCM ciphertext at rest.
// We do NOT use `query_as::<_, UserDto>()` directly because the Email value
// object would reject the ciphertext during TryFrom validation.
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct RawUserRow {
    id: Uuid,
    username: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct RawUserExtendedRow {
    id: Uuid,
    username: String,
    email: String,
    role: String,
    email_verified: bool,
    email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    mfa_enabled: bool,
    is_banned: bool,
    banned_at: Option<chrono::DateTime<chrono::Utc>>,
    banned_reason: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
    encryption_key: [u8; 32],
}

impl UserRepository {
    pub fn new(pool: PgPool, encryption_key: [u8; 32]) -> Self {
        Self {
            pool,
            encryption_key,
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn encrypt_email(&self, email: &str) -> Result<String, RepositoryError> {
        field_encryption::encrypt_field(email, &self.encryption_key)
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))
    }

    /// Decrypts email ciphertext. Falls back to returning the raw value during
    /// the migration period when rows may still contain plaintext.
    fn decrypt_email(&self, raw: &str) -> Result<String, RepositoryError> {
        match field_encryption::decrypt_field(raw, &self.encryption_key) {
            Ok(plain) => Ok(plain),
            // If decryption fails the value is likely a legacy plaintext email.
            Err(_) => Ok(raw.to_string()),
        }
    }

    fn email_index(&self, email: &str) -> String {
        field_encryption::blind_index(email, &self.encryption_key)
    }

    fn to_user_dto(&self, row: RawUserRow) -> Result<UserDto, RepositoryError> {
        let email_plain = self.decrypt_email(&row.email)?;
        let email = Email::try_from(email_plain)
            .map_err(|e| RepositoryError::InvalidData(e))?;
        let username = Username::try_from(row.username)
            .map_err(|e| RepositoryError::InvalidData(e))?;
        Ok(UserDto {
            id: row.id,
            username,
            email,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    fn to_user_dto_extended(&self, row: RawUserExtendedRow) -> Result<UserDtoExtended, RepositoryError> {
        let email_plain = self.decrypt_email(&row.email)?;
        let email = Email::try_from(email_plain)
            .map_err(|e| RepositoryError::InvalidData(e))?;
        let username = Username::try_from(row.username)
            .map_err(|e| RepositoryError::InvalidData(e))?;
        Ok(UserDtoExtended {
            id: row.id,
            username,
            email,
            role: row.role,
            email_verified: row.email_verified,
            email_verified_at: row.email_verified_at,
            mfa_enabled: row.mfa_enabled,
            is_banned: row.is_banned,
            banned_at: row.banned_at,
            banned_reason: row.banned_reason,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// ---------------------------------------------------------------------------

#[async_trait]
impl UserRepositoryPort for UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, RepositoryError> {
        let row: Option<RawUserRow> = sqlx::query_as(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        row.map(|r| self.to_user_dto(r)).transpose()
    }

    async fn find_extended_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<UserDtoExtended>, RepositoryError> {
        let row: Option<RawUserExtendedRow> = sqlx::query_as(
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
        .map_err(map_db_error)?;

        row.map(|r| self.to_user_dto_extended(r)).transpose()
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
        let row: Option<RawUserRow> = sqlx::query_as(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE username = $1",
        )
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        row.map(|r| self.to_user_dto(r)).transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<UserDto>, RepositoryError> {
        let idx = self.email_index(email.as_str());
        let row: Option<RawUserRow> = sqlx::query_as(
            "SELECT id, username, email, created_at, updated_at \
             FROM users WHERE email_index = $1",
        )
        .bind(&idx)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        row.map(|r| self.to_user_dto(r)).transpose()
    }

    async fn find_for_login(&self, identifier: &str) -> Result<Option<UserLoginInfo>, RepositoryError> {
        // Compute the email blind index so we can match encrypted emails.
        let idx = self.email_index(identifier);
        sqlx::query_as::<_, UserLoginInfo>(
            "SELECT id, username, password_hash, mfa_enabled \
             FROM users WHERE username = $1 OR email_index = $2",
        )
        .bind(identifier)
        .bind(&idx)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let idx = self.email_index(email.as_str());
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email_index = $1)",
        )
        .bind(&idx)
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
        let idx = self.email_index(email.as_str());
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email_index = $1 AND id != $2)",
        )
        .bind(&idx)
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
        let email_encrypted = self.encrypt_email(email.as_str())?;
        let email_idx = self.email_index(email.as_str());

        let row: RawUserRow = sqlx::query_as(
            r#"
            INSERT INTO users (username, email, email_index, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, created_at, updated_at
            "#,
        )
        .bind(username.as_str())
        .bind(&email_encrypted)
        .bind(&email_idx)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.to_user_dto(row)
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
        let email_encrypted = self.encrypt_email(new_email.as_str())?;
        let email_idx = self.email_index(new_email.as_str());

        sqlx::query(
            "UPDATE users SET email = $1, email_index = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(&email_encrypted)
        .bind(&email_idx)
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

    async fn ban_user(&self, id: Uuid, reason: Option<String>) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET is_banned = TRUE, banned_at = NOW(), banned_reason = $2, updated_at = NOW()
            WHERE id = $1 AND is_banned = FALSE
            "#,
        )
        .bind(id)
        .bind(reason.as_deref())
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
        // Email search is disabled while the column is encrypted.
        // Search is applied to username only.
        let mut query_str =
            "SELECT id, username, email, created_at, updated_at FROM users".to_string();
        let mut count_str = "SELECT COUNT(*) FROM users".to_string();
        let mut params_count = 0;

        if search.is_some() {
            params_count += 1;
            let clause = format!(" WHERE username ILIKE ${}", params_count);
            query_str.push_str(&clause);
            count_str.push_str(&clause);
        }

        query_str.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            params_count + 1,
            params_count + 2
        ));

        let mut query = sqlx::query_as::<_, RawUserRow>(&query_str);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_str);

        if let Some(s) = search {
            let search_fmt = format!("%{}%", s);
            query = query.bind(search_fmt.clone());
            count_query = count_query.bind(search_fmt);
        }

        query = query.bind(limit).bind(offset);

        let raw_rows = query.fetch_all(&self.pool).await.map_err(map_db_error)?;
        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        let users: Result<Vec<UserDto>, RepositoryError> =
            raw_rows.into_iter().map(|r| self.to_user_dto(r)).collect();

        Ok((users?, total))
    }
}
