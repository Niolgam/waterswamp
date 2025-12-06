use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::RepositoryError;
use domain::models::{CreateOrganizationalUnitDto, OrganizationalUnit, UpdateOrganizationalUnitDto};
use domain::ports::OrganizationalUnitRepositoryPort;

pub struct OrganizationalUnitRepository {
    pool: PgPool,
}

impl OrganizationalUnitRepository {
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
impl OrganizationalUnitRepositoryPort for OrganizationalUnitRepository {
    async fn create(&self, dto: &CreateOrganizationalUnitDto) -> Result<OrganizationalUnit, RepositoryError> {
        let unit = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            INSERT INTO organizational_units
                (id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&dto.name)
        .bind(&dto.acronym)
        .bind(&dto.category_id)
        .bind(&dto.parent_id)
        .bind(&dto.description)
        .bind(dto.is_uorg.unwrap_or(false))
        .bind(&dto.campus_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(unit)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnit>, RepositoryError> {
        let unit = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(unit)
    }

    async fn find_by_acronym(&self, acronym: &str) -> Result<Option<OrganizationalUnit>, RepositoryError> {
        let unit = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            WHERE acronym = $1
            "#,
        )
        .bind(acronym)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(unit)
    }

    async fn list_all(&self) -> Result<Vec<OrganizationalUnit>, RepositoryError> {
        let units = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(units)
    }

    async fn list_by_parent(&self, parent_id: Option<Uuid>) -> Result<Vec<OrganizationalUnit>, RepositoryError> {
        let units = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            WHERE ($1::UUID IS NULL AND parent_id IS NULL) OR parent_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(units)
    }

    async fn list_by_category(&self, category_id: Uuid) -> Result<Vec<OrganizationalUnit>, RepositoryError> {
        let units = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            WHERE category_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(category_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(units)
    }

    async fn list_by_campus(&self, campus_id: Uuid) -> Result<Vec<OrganizationalUnit>, RepositoryError> {
        let units = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            WHERE campus_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(campus_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(units)
    }

    async fn list_root_units(&self) -> Result<Vec<OrganizationalUnit>, RepositoryError> {
        let units = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            SELECT id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            FROM organizational_units
            WHERE parent_id IS NULL
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(units)
    }

    async fn update(&self, id: Uuid, dto: &UpdateOrganizationalUnitDto) -> Result<OrganizationalUnit, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::Database("Organizational unit not found".to_string()))?;

        let unit = sqlx::query_as::<_, OrganizationalUnit>(
            r#"
            UPDATE organizational_units
            SET name = $1, acronym = $2, category_id = $3, parent_id = $4,
                description = $5, is_uorg = $6, campus_id = $7, updated_at = NOW()
            WHERE id = $8
            RETURNING id, name, acronym, category_id, parent_id, description, is_uorg, campus_id, created_at, updated_at
            "#,
        )
        .bind(dto.name.as_ref().unwrap_or(&existing.name))
        .bind(dto.acronym.as_ref().or(existing.acronym.as_ref()))
        .bind(dto.category_id.unwrap_or(existing.category_id))
        .bind(dto.parent_id.or(existing.parent_id))
        .bind(dto.description.as_ref().or(existing.description.as_ref()))
        .bind(dto.is_uorg.unwrap_or(existing.is_uorg))
        .bind(dto.campus_id.or(existing.campus_id))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(unit)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM organizational_units WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizational_units")
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(count)
    }
}
