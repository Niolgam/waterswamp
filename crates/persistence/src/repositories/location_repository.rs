use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{CityDto, CityWithStateDto, SiteTypeDto, StateDto};
use domain::ports::{CityRepositoryPort, SiteTypeRepositoryPort, StateRepositoryPort};
use domain::value_objects::{LocationName, StateCode};
use sqlx::PgPool;
use uuid::Uuid;

// ============================
// State Repository
// ============================

#[derive(Clone)]
pub struct StateRepository {
    pool: PgPool,
}

impl StateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                }
                if code == "23503" {
                    return RepositoryError::Database(
                        "Foreign key constraint violation".to_string(),
                    );
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl StateRepositoryPort for StateRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StateDto>, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "SELECT id, name, code, created_at, updated_at FROM states WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_code(&self, code: &StateCode) -> Result<Option<StateDto>, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "SELECT id, name, code, created_at, updated_at FROM states WHERE code = $1",
        )
        .bind(code.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_code(&self, code: &StateCode) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM states WHERE code = $1")
            .bind(code.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(
        &self,
        code: &StateCode,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM states WHERE code = $1 AND id != $2")
                .bind(code.as_str())
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &LocationName,
        code: &StateCode,
    ) -> Result<StateDto, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "INSERT INTO states (name, code) VALUES ($1, $2) RETURNING id, name, code, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(code.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        code: Option<&StateCode>,
    ) -> Result<StateDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if code.is_some() {
            query_parts.push(format!("code = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            // If no fields to update, just return the existing state
            return self
                .find_by_id(id)
                .await?
                .ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE states SET {} WHERE id = ${} RETURNING id, name, code, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, StateDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val.as_str());
        }
        if let Some(code_val) = code {
            query = query.bind(code_val.as_str());
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM states WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<StateDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let states = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, StateDto>(
                "SELECT id, name, code, created_at, updated_at FROM states
                 WHERE name ILIKE $1 OR code ILIKE $1
                 ORDER BY name LIMIT $2 OFFSET $3",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        } else {
            sqlx::query_as::<_, StateDto>(
                "SELECT id, name, code, created_at, updated_at FROM states
                 ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM states WHERE name ILIKE $1 OR code ILIKE $1",
            )
            .bind(pattern)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM states")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        };

        Ok((states, total))
    }
}

// ============================
// City Repository
// ============================

#[derive(Clone)]
pub struct CityRepository {
    pool: PgPool,
}

impl CityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                }
                if code == "23503" {
                    return RepositoryError::Database(
                        "Foreign key constraint violation".to_string(),
                    );
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl CityRepositoryPort for CityRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CityDto>, RepositoryError> {
        sqlx::query_as::<_, CityDto>(
            "SELECT id, name, state_id, created_at, updated_at FROM cities WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_state_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CityWithStateDto>, RepositoryError> {
        let result = sqlx::query_as::<_, CityWithStateDto>(
            r#"
            SELECT
                c.id, c.name, c.state_id,
                s.name as state_name, s.code as state_code,
                c.created_at, c.updated_at
            FROM cities c
            INNER JOIN states s ON c.state_id = s.id
            WHERE c.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(result)
    }

    async fn create(
        &self,
        name: &LocationName,
        state_id: Uuid,
    ) -> Result<CityDto, RepositoryError> {
        sqlx::query_as::<_, CityDto>(
            "INSERT INTO cities (name, state_id) VALUES ($1, $2)
             RETURNING id, name, state_id, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(state_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        state_id: Option<Uuid>,
    ) -> Result<CityDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if state_id.is_some() {
            query_parts.push(format!("state_id = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE cities SET {} WHERE id = ${} RETURNING id, name, state_id, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, CityDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val.as_str());
        }
        if let Some(state_id_val) = state_id {
            query = query.bind(state_id_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM cities WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        state_id: Option<Uuid>,
    ) -> Result<(Vec<CityWithStateDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let cities = match (search_pattern.as_ref(), state_id) {
            (Some(pattern), Some(state)) => {
                sqlx::query_as::<_, CityWithStateDto>(
                    r#"
                    SELECT
                        c.id, c.name, c.state_id,
                        s.name as state_name, s.code as state_code,
                        c.created_at, c.updated_at
                    FROM cities c
                    INNER JOIN states s ON c.state_id = s.id
                    WHERE c.name ILIKE $1 AND c.state_id = $2
                    ORDER BY c.name LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(pattern)
                .bind(state)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(Self::map_err)?
            }
            (Some(pattern), None) => {
                sqlx::query_as::<_, CityWithStateDto>(
                    r#"
                    SELECT
                        c.id, c.name, c.state_id,
                        s.name as state_name, s.code as state_code,
                        c.created_at, c.updated_at
                    FROM cities c
                    INNER JOIN states s ON c.state_id = s.id
                    WHERE c.name ILIKE $1
                    ORDER BY c.name LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(Self::map_err)?
            }
            (None, Some(state)) => {
                sqlx::query_as::<_, CityWithStateDto>(
                    r#"
                    SELECT
                        c.id, c.name, c.state_id,
                        s.name as state_name, s.code as state_code,
                        c.created_at, c.updated_at
                    FROM cities c
                    INNER JOIN states s ON c.state_id = s.id
                    WHERE c.state_id = $1
                    ORDER BY c.name LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(state)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(Self::map_err)?
            }
            (None, None) => {
                sqlx::query_as::<_, CityWithStateDto>(
                    r#"
                    SELECT
                        c.id, c.name, c.state_id,
                        s.name as state_name, s.code as state_code,
                        c.created_at, c.updated_at
                    FROM cities c
                    INNER JOIN states s ON c.state_id = s.id
                    ORDER BY c.name LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(Self::map_err)?
            }
        };

        let total: i64 = match (search_pattern.as_ref(), state_id) {
            (Some(pattern), Some(state)) => {
                sqlx::query_scalar(
                    "SELECT COUNT(*) FROM cities WHERE name ILIKE $1 AND state_id = $2",
                )
                .bind(pattern)
                .bind(state)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
            }
            (Some(pattern), None) => {
                sqlx::query_scalar("SELECT COUNT(*) FROM cities WHERE name ILIKE $1")
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (None, Some(state)) => {
                sqlx::query_scalar("SELECT COUNT(*) FROM cities WHERE state_id = $1")
                    .bind(state)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (None, None) => sqlx::query_scalar("SELECT COUNT(*) FROM cities")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?,
        };

        Ok((cities, total))
    }
}

// ============================
// Site Type Repository
// ============================

#[derive(Clone)]
pub struct SiteTypeRepository {
    pool: PgPool,
}

impl SiteTypeRepository {
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
impl SiteTypeRepositoryPort for SiteTypeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiteTypeDto>, RepositoryError> {
        sqlx::query_as::<_, SiteTypeDto>(
            "SELECT id, name, description, created_at, updated_at FROM site_types WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<SiteTypeDto>, RepositoryError> {
        sqlx::query_as::<_, SiteTypeDto>(
            "SELECT id, name, description, created_at, updated_at FROM site_types WHERE name = $1",
        )
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_name(&self, name: &LocationName) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM site_types WHERE name = $1")
            .bind(name.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(
        &self,
        name: &LocationName,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM site_types WHERE name = $1 AND id != $2")
                .bind(name.as_str())
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &LocationName,
        description: Option<&str>,
    ) -> Result<SiteTypeDto, RepositoryError> {
        sqlx::query_as::<_, SiteTypeDto>(
            "INSERT INTO site_types (name, description) VALUES ($1, $2)
             RETURNING id, name, description, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        description: Option<&str>,
    ) -> Result<SiteTypeDto, RepositoryError> {
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
            "UPDATE site_types SET {} WHERE id = ${} RETURNING id, name, description, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, SiteTypeDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val.as_str());
        }
        if let Some(desc_val) = description {
            query = query.bind(desc_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM site_types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<SiteTypeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let site_types = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, SiteTypeDto>(
                "SELECT id, name, description, created_at, updated_at FROM site_types
                 WHERE name ILIKE $1
                 ORDER BY name LIMIT $2 OFFSET $3",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        } else {
            sqlx::query_as::<_, SiteTypeDto>(
                "SELECT id, name, description, created_at, updated_at FROM site_types
                 ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar("SELECT COUNT(*) FROM site_types WHERE name ILIKE $1")
                .bind(pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM site_types")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        };

        Ok((site_types, total))
    }
}
