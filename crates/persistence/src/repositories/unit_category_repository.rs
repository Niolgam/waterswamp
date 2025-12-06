use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::RepositoryError;
use domain::models::{CreateUnitCategoryDto, UnitCategory, UpdateUnitCategoryDto};
use domain::ports::UnitCategoryRepositoryPort;

pub struct UnitCategoryRepository {
    pool: PgPool,
}

impl UnitCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl UnitCategoryRepositoryPort for UnitCategoryRepository {
    async fn create(&self, dto: &CreateUnitCategoryDto) -> Result<UnitCategory, RepositoryError> {
        let category = sqlx::query_as::<_, UnitCategory>(
            r#"
            INSERT INTO unit_categories (id, name, color_hex, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING id, name, color_hex, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&dto.name)
        .bind(&dto.color_hex)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(category)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitCategory>, RepositoryError> {
        let category = sqlx::query_as::<_, UnitCategory>(
            r#"
            SELECT id, name, color_hex, created_at
            FROM unit_categories
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(category)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<UnitCategory>, RepositoryError> {
        let category = sqlx::query_as::<_, UnitCategory>(
            r#"
            SELECT id, name, color_hex, created_at
            FROM unit_categories
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(category)
    }

    async fn list_all(&self) -> Result<Vec<UnitCategory>, RepositoryError> {
        let categories = sqlx::query_as::<_, UnitCategory>(
            r#"
            SELECT id, name, color_hex, created_at
            FROM unit_categories
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(categories)
    }

    async fn update(&self, id: Uuid, dto: &UpdateUnitCategoryDto) -> Result<UnitCategory, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::Database("Unit category not found".to_string()))?;

        let category = sqlx::query_as::<_, UnitCategory>(
            r#"
            UPDATE unit_categories
            SET name = $1, color_hex = $2
            WHERE id = $3
            RETURNING id, name, color_hex, created_at
            "#,
        )
        .bind(dto.name.as_ref().unwrap_or(&existing.name))
        .bind(dto.color_hex.as_ref().unwrap_or(&existing.color_hex))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(category)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM unit_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM unit_categories WHERE name = $1)")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(exists)
    }
}
