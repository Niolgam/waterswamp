use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::DepartmentCategoryDto;
use domain::ports::DepartmentCategoryRepositoryPort;
use domain::value_objects::LocationName;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================
// Department Category Repository
// ============================

#[derive(Clone)]
pub struct DepartmentCategoryRepository {
    pool: PgPool,
}

impl DepartmentCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DepartmentCategoryRepositoryPort for DepartmentCategoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DepartmentCategoryDto>, RepositoryError> {
        sqlx::query_as::<_, DepartmentCategoryDto>(
            "SELECT id, name, description, created_at, updated_at FROM department_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<DepartmentCategoryDto>, RepositoryError> {
        sqlx::query_as::<_, DepartmentCategoryDto>(
            "SELECT id, name, description, created_at, updated_at FROM department_categories WHERE name = $1",
        )
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_name(&self, name: &LocationName) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM department_categories WHERE name = $1")
            .bind(name.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(
        &self,
        name: &LocationName,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM department_categories WHERE name = $1 AND id != $2")
                .bind(name.as_str())
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &LocationName,
        description: Option<&str>,
    ) -> Result<DepartmentCategoryDto, RepositoryError> {
        sqlx::query_as::<_, DepartmentCategoryDto>(
            "INSERT INTO department_categories (name, description) VALUES ($1, $2)
             RETURNING id, name, description, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        description: Option<&str>,
    ) -> Result<DepartmentCategoryDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if description.is_some() {
            query_parts.push(format!("description = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE department_categories SET {} WHERE id = ${} RETURNING id, name, description, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, DepartmentCategoryDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val.as_str());
        }
        if let Some(desc_val) = description {
            query = query.bind(desc_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM department_categories WHERE id = $1")
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
    ) -> Result<(Vec<DepartmentCategoryDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let department_categories = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, DepartmentCategoryDto>(
                "SELECT id, name, description, created_at, updated_at FROM department_categories
                 WHERE name ILIKE $1
                 ORDER BY name LIMIT $2 OFFSET $3",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
        } else {
            sqlx::query_as::<_, DepartmentCategoryDto>(
                "SELECT id, name, description, created_at, updated_at FROM department_categories
                 ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar("SELECT COUNT(*) FROM department_categories WHERE name ILIKE $1")
                .bind(pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM department_categories")
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?
        };

        Ok((department_categories, total))
    }
}
