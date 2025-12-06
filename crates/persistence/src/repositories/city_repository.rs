use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::RepositoryError;
use domain::models::{City, CreateCityDto, UpdateCityDto};
use domain::ports::CityRepositoryPort;

pub struct CityRepository {
    pool: PgPool,
}

impl CityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl CityRepositoryPort for CityRepository {
    async fn create(&self, dto: &CreateCityDto) -> Result<City, RepositoryError> {
        let city = sqlx::query_as::<_, City>(
            r#"
            INSERT INTO cities (id, name, state_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING id, name, state_id, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&dto.name)
        .bind(&dto.state_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(city)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<City>, RepositoryError> {
        let city = sqlx::query_as::<_, City>(
            r#"
            SELECT id, name, state_id, created_at, updated_at
            FROM cities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(city)
    }

    async fn list_by_state(&self, state_id: Uuid) -> Result<Vec<City>, RepositoryError> {
        let cities = sqlx::query_as::<_, City>(
            r#"
            SELECT id, name, state_id, created_at, updated_at
            FROM cities
            WHERE state_id = $1
            ORDER BY name
            "#,
        )
        .bind(state_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(cities)
    }

    async fn list_all(&self) -> Result<Vec<City>, RepositoryError> {
        let cities = sqlx::query_as::<_, City>(
            r#"
            SELECT id, name, state_id, created_at, updated_at
            FROM cities
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(cities)
    }

    async fn update(&self, id: Uuid, dto: &UpdateCityDto) -> Result<City, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::Database("City not found".to_string()))?;

        let city = sqlx::query_as::<_, City>(
            r#"
            UPDATE cities
            SET name = $1, state_id = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, state_id, created_at, updated_at
            "#,
        )
        .bind(dto.name.as_ref().unwrap_or(&existing.name))
        .bind(dto.state_id.unwrap_or(existing.state_id))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(city)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM cities WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }
}
