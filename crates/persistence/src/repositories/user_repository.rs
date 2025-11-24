use domain::models::{UserDto, UserDtoExtended};
use sqlx::PgPool;
use uuid::Uuid;

/// Repositório para operações relacionadas a usuários.
pub struct UserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    /// Cria uma nova instância do repositório.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Busca um usuário pelo ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, sqlx::Error> {
        sqlx::query_as::<_, UserDto>(
            // ⭐ ATUALIZADO: Adicionado 'email'
            "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn find_extended_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<UserDtoExtended>, sqlx::Error> {
        // Ensure you select 'role' here. Assuming 'role' column exists in users table.
        sqlx::query_as::<_, UserDtoExtended>(
            r#"
            SELECT 
                id, username, email, role, 
                email_verified, email_verified_at, mfa_enabled, 
                created_at, updated_at 
            FROM users 
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn get_password_hash(&self, id: Uuid) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    /// Mark email as unverified (helper for update_email)
    pub async fn mark_email_unverified(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET email_verified = false, email_verified_at = NULL WHERE id = $1",
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// Busca um usuário pelo Username.
    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserDto>, sqlx::Error> {
        sqlx::query_as::<_, UserDto>(
            // ⭐ ATUALIZADO: Adicionado 'email'
            "SELECT id, username, email, created_at, updated_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserDto>, sqlx::Error> {
        sqlx::query_as::<_, UserDto>(
            "SELECT id, username, email, created_at, updated_at FROM users WHERE LOWER(email) = LOWER($1)",
        )
        .bind(email)
        .fetch_optional(self.pool)
        .await
    }

    /// ⭐ NOVO: Verifica se um email já existe (case-insensitive).
    pub async fn exists_by_email(&self, email: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1))")
            .bind(email)
            .fetch_one(self.pool)
            .await
    }

    /// Verifica se um username já existe.
    pub async fn exists_by_username(&self, username: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(username)
            .fetch_one(self.pool)
            .await
    }

    /// ⭐ NOVO: Verifica se um email já existe, excluindo um ID.
    pub async fn exists_by_email_excluding(
        &self,
        email: &str,
        exclude_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1) AND id != $2)",
        )
        .bind(email)
        .bind(exclude_id)
        .fetch_one(self.pool)
        .await
    }

    /// Verifica se um username já existe, excluindo um ID específico (útil para updates).
    pub async fn exists_by_username_excluding(
        &self,
        username: &str,
        exclude_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)")
            .bind(username)
            .bind(exclude_id)
            .fetch_one(self.pool)
            .await
    }

    /// Cria um novo usuário.
    // ⭐ ATUALIZADO: Adicionada assinatura 'email'
    pub async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<UserDto, sqlx::Error> {
        sqlx::query_as::<_, UserDto>(
            // ⭐ ATUALIZADO: Adicionados 'email'
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, created_at, updated_at
            "#,
        )
        .bind(username)
        .bind(email) // Adicionado
        .bind(password_hash)
        .fetch_one(self.pool)
        .await
    }

    /// Atualiza o username de um usuário.
    pub async fn update_username(&self, id: Uuid, new_username: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_username)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// ⭐ NOVO: Atualiza o email de um usuário.
    pub async fn update_email(&self, id: Uuid, new_email: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_email)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Atualiza o hash da senha de um usuário.
    pub async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_password_hash)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Deleta um usuário. Retorna true se deletou, false se não encontrou.
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Lista usuários com paginação e busca opcional.
    /// Retorna uma tupla: (Vec<UserDto>, total_count)
    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<&String>,
    ) -> Result<(Vec<UserDto>, i64), sqlx::Error> {
        // ⭐ ATUALIZADO: Adicionado 'email'
        let mut query_str =
            "SELECT id, username, email, created_at, updated_at FROM users".to_string();
        let mut count_str = "SELECT COUNT(*) FROM users".to_string();

        if let Some(s) = search {
            // ⭐ ATUALIZADO: Busca por username OU email
            let where_clause = format!(" WHERE username ILIKE '%{0}%' OR email ILIKE '%{0}%'", s);
            query_str.push_str(&where_clause);
            count_str.push_str(&where_clause);
        }

        query_str.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");

        let users = sqlx::query_as::<_, UserDto>(&query_str)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?;

        let total: i64 = sqlx::query_scalar(&count_str).fetch_one(self.pool).await?;

        Ok((users, total))
    }
}
