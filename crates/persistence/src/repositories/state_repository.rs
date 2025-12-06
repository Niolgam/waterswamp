use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::RepositoryError;
use domain::models::{CreateStateDto, State, UpdateStateDto};
use domain::ports::StateRepositoryPort;

pub struct StateRepository {
    pool: PgPool,
}

impl StateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl StateRepositoryPort for StateRepository {
    async fn create(&self, dto: &CreateStateDto) -> Result<State, RepositoryError> {
        let state = sqlx::query_as::<_, State>(
            r#"
            INSERT INTO states (id, name, code, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING id, name, code, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&dto.name)
        .bind(&dto.code)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(state)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<State>, RepositoryError> {
        let state = sqlx::query_as::<_, State>(
            r#"
            SELECT id, name, code, created_at, updated_at
            FROM states
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(state)
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<State>, RepositoryError> {
        let state = sqlx::query_as::<_, State>(
            r#"
            SELECT id, name, code, created_at, updated_at
            FROM states
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(state)
    }

    async fn list_all(&self) -> Result<Vec<State>, RepositoryError> {
        let states = sqlx::query_as::<_, State>(
            r#"
            SELECT id, name, code, created_at, updated_at
            FROM states
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(states)
    }

    async fn update(&self, id: Uuid, dto: &UpdateStateDto) -> Result<State, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::Database("State not found".to_string()))?;

        let state = sqlx::query_as::<_, State>(
            r#"
            UPDATE states
            SET name = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, code, created_at, updated_at
            "#,
        )
        .bind(dto.name.as_ref().unwrap_or(&existing.name))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(state)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM states WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM states WHERE code = $1)")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(exists)
    }
}
