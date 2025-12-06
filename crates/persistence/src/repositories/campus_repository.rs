use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::campus::{Campus, CreateCampusDto, UpdateCampusDto};
use domain::ports::CampusRepositoryPort;
use domain::value_objects::Coordinates;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct CampusRepository {
    pool: PgPool,
}

impl CampusRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        // Detect duplication (Postgres code 23505)
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
impl CampusRepositoryPort for CampusRepository {
    async fn create(&self, dto: &CreateCampusDto) -> Result<Campus, RepositoryError> {
        // Validate and create coordinates
        let coordinates = Coordinates::new(dto.coordinates.latitude, dto.coordinates.longitude)
            .map_err(|e| RepositoryError::Database(e))?;

        let campus = sqlx::query_as::<_, Campus>(
            r#"
            INSERT INTO campuses (id, name, acronym, city_id, coordinates, address, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING id, name, acronym, city_id, coordinates, address, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&dto.name)
        .bind(&dto.acronym)
        .bind(&dto.city_id)
        .bind(&coordinates)
        .bind(&dto.address)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Campus>, RepositoryError> {
        let campus = sqlx::query_as::<_, Campus>(
            r#"
            SELECT id, name, acronym, city_id, coordinates, address, created_at, updated_at
            FROM campuses
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus)
    }

    async fn find_by_acronym(&self, acronym: &str) -> Result<Option<Campus>, RepositoryError> {
        let campus = sqlx::query_as::<_, Campus>(
            r#"
            SELECT id, name, acronym, city_id, coordinates, address, created_at, updated_at
            FROM campuses
            WHERE acronym = $1
            "#,
        )
        .bind(acronym)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Campus>, RepositoryError> {
        let campus = sqlx::query_as::<_, Campus>(
            r#"
            SELECT id, name, acronym, city_id, coordinates, address, created_at, updated_at
            FROM campuses
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus)
    }

    async fn list_all(&self) -> Result<Vec<Campus>, RepositoryError> {
        let campus_list = sqlx::query_as::<_, Campus>(
            r#"
            SELECT id, name, acronym, city_id, coordinates, address, created_at, updated_at
            FROM campuses
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus_list)
    }

    async fn list_paginated(&self, limit: i64, offset: i64) -> Result<Vec<Campus>, RepositoryError> {
        let campus_list = sqlx::query_as::<_, Campus>(
            r#"
            SELECT id, name, acronym, city_id, coordinates, address, created_at, updated_at
            FROM campuses
            ORDER BY name ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus_list)
    }

    async fn find_by_city(&self, city_id: Uuid) -> Result<Vec<Campus>, RepositoryError> {
        let campus_list = sqlx::query_as::<_, Campus>(
            r#"
            SELECT id, name, acronym, city_id, coordinates, address, created_at, updated_at
            FROM campuses
            WHERE city_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(city_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(campus_list)
    }

    async fn update(&self, id: Uuid, dto: &UpdateCampusDto) -> Result<Campus, RepositoryError> {
        // Find current campus
        let current = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::Database("Campus not found".to_string()))?;

        // Apply updates (use current values if not provided)
        let name = dto.name.as_ref().unwrap_or(&current.name);
        let acronym = dto.acronym.as_ref().unwrap_or(&current.acronym);
        let city_id = dto.city_id.unwrap_or(current.city_id);
        let address = dto.address.as_ref().unwrap_or(&current.address);

        let coordinates = if let Some(ref coord_dto) = dto.coordinates {
            Coordinates::new(coord_dto.latitude, coord_dto.longitude)
                .map_err(|e| RepositoryError::Database(e))?
        } else {
            current.coordinates
        };

        let updated_campus = sqlx::query_as::<_, Campus>(
            r#"
            UPDATE campuses
            SET name = $2, acronym = $3, city_id = $4, coordinates = $5, address = $6, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, acronym, city_id, coordinates, address, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(acronym)
        .bind(city_id)
        .bind(&coordinates)
        .bind(address)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(updated_campus)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM campuses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_acronym(&self, acronym: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM campuses WHERE acronym = $1)")
            .bind(acronym)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(exists)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM campuses WHERE name = $1)")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campuses")
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::campus::CoordinatesDto;

    // Integration tests should be run with a real database
    // They are ignored by default
    #[ignore]
    #[tokio::test]
    async fn test_create_and_find_campus() {
        // This test requires a running PostgreSQL database
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let repo = CampusRepository::new(pool);
        let city_id = Uuid::new_v4(); // In real test, this should be a valid city ID

        let dto = CreateCampusDto {
            name: "Test Campus".to_string(),
            acronym: "TC".to_string(),
            city_id,
            coordinates: CoordinatesDto {
                latitude: -23.5505,
                longitude: -46.6333,
            },
            address: "Test Street, 123".to_string(),
        };

        let created = repo.create(&dto).await.expect("Failed to create campus");
        assert_eq!(created.name, "Test Campus");

        let found = repo
            .find_by_id(created.id)
            .await
            .expect("Failed to find campus")
            .expect("Campus not found");

        assert_eq!(found.id, created.id);
        assert_eq!(found.acronym, "TC");

        // Cleanup
        repo.delete(created.id).await.expect("Failed to delete");
    }
}
