use sqlx::PgPool;
use uuid::Uuid;

/// Repositório para operações relacionadas a autenticação e tokens.
pub struct AuthRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AuthRepository<'a> {
    /// Cria uma nova instância do repositório.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Salva um novo refresh token no banco de dados.
    /// A expiração é definida para 30 dias a partir de agora pelo banco.
    pub async fn save_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, NOW() + INTERVAL '30 days')
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Busca um refresh token válido (não revogado e não expirado) pelo hash.
    /// Retorna o ID do usuário se encontrado.
    pub async fn find_valid_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let result: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT user_id
            FROM refresh_tokens
            WHERE token_hash = $1
              AND revoked = FALSE
              AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.pool)
        .await?;

        Ok(result.map(|(user_id,)| user_id))
    }

    /// Revoga um refresh token específico pelo seu hash.
    /// Retorna true se o token foi encontrado e revogado, false caso contrário.
    pub async fn revoke_refresh_token(&self, token_hash: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, updated_at = NOW()
            WHERE token_hash = $1 AND revoked = FALSE
            "#,
        )
        .bind(token_hash)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoga todos os refresh tokens de um usuário específico.
    pub async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, updated_at = NOW()
            WHERE user_id = $1 AND revoked = FALSE
            "#,
        )
        .bind(user_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
