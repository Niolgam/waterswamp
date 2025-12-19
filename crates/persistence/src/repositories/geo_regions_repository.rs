use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{CityDto, CityWithStateDto, CountryDto, StateDto, StateWithCountryDto};
use domain::ports::{CityRepositoryPort, CountryRepositoryPort, StateRepositoryPort};
use domain::value_objects::{LocationName, StateCode};
use sqlx::PgPool;
use uuid::Uuid;

// ============================
// Country Repository
// ============================

#[derive(Clone)]
pub struct CountryRepository {
    pool: PgPool,
}

impl CountryRepository {
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
impl CountryRepositoryPort for CountryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CountryDto>, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "SELECT id, name, code, created_at, updated_at FROM countries WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CountryDto>, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "SELECT id, name, code, created_at, updated_at FROM countries WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(
        &self,
        code: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE code = $1 AND id != $2")
                .bind(code)
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &LocationName, code: &str) -> Result<CountryDto, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "INSERT INTO countries (name, code) VALUES ($1, $2) RETURNING id, name, code, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        code: Option<&str>,
    ) -> Result<CountryDto, RepositoryError> {
        let current = self
            .find_by_id(id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let new_name = name.unwrap_or(&current.name);
        let new_code = code.unwrap_or(&current.code);

        sqlx::query_as::<_, CountryDto>(
            "UPDATE countries SET name = $1, code = $2, updated_at = NOW() WHERE id = $3 RETURNING id, name, code, created_at, updated_at",
        )
        .bind(new_name.as_str())
        .bind(new_code)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM countries WHERE id = $1")
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
    ) -> Result<(Vec<CountryDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let countries = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, CountryDto>(
                "SELECT id, name, code, created_at, updated_at FROM countries WHERE name ILIKE $1 ORDER BY name LIMIT $2 OFFSET $3",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        } else {
            sqlx::query_as::<_, CountryDto>(
                "SELECT id, name, code, created_at, updated_at FROM countries ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE name ILIKE $1")
                .bind(pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM countries")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        };

        Ok((countries, total))
    }
}

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
            "SELECT id, name, code, country_id, created_at, updated_at FROM states WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_country_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<StateWithCountryDto>, RepositoryError> {
        sqlx::query_as::<_, StateWithCountryDto>(
            r#"
            SELECT
                s.id, s.name, s.code, s.country_id,
                c.name as country_name, c.code as country_code,
                s.created_at, s.updated_at
            FROM states s
            INNER JOIN countries c ON s.country_id = c.id
            WHERE s.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_code(&self, code: &StateCode) -> Result<Option<StateDto>, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "SELECT id, name, code, country_id, created_at, updated_at FROM states WHERE code = $1",
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
        country_id: Uuid,
    ) -> Result<StateDto, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "INSERT INTO states (name, code, country_id) VALUES ($1, $2, $3) RETURNING id, name, code, country_id, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(code.as_str())
        .bind(country_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        code: Option<&StateCode>,
        country_id: Option<Uuid>,
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
        if country_id.is_some() {
            query_parts.push(format!("country_id = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            // If no fields to update, just return the existing state
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE states SET {} WHERE id = ${} RETURNING id, name, code, country_id, created_at, updated_at",
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
        if let Some(country_id_val) = country_id {
            query = query.bind(country_id_val);
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
        country_id: Option<Uuid>,
    ) -> Result<(Vec<StateWithCountryDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!(
                "(s.name ILIKE ${} OR s.code ILIKE ${})",
                bind_index, bind_index
            ));
            bind_index += 1;
        }
        if country_id.is_some() {
            conditions.push(format!("s.country_id = ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            r#"
            SELECT
                s.id, s.name, s.code, s.country_id,
                c.name as country_name, c.code as country_code,
                s.created_at, s.updated_at
            FROM states s
            INNER JOIN countries c ON s.country_id = c.id
            {}
            ORDER BY s.name LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut query = sqlx::query_as::<_, StateWithCountryDto>(&query_str);

        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(country_id_val) = country_id {
            query = query.bind(country_id_val);
        }
        query = query.bind(limit).bind(offset);

        let states = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count query
        let count_query_str = format!(
            "SELECT COUNT(*) FROM states s INNER JOIN countries c ON s.country_id = c.id {}",
            where_clause
        );

        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(country_id_val) = country_id {
            count_query = count_query.bind(country_id_val);
        }

        let total: i64 = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

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
                co.id as country_id, co.name as country_name, co.code as country_code,
                c.created_at, c.updated_at
            FROM cities c
            INNER JOIN states s ON c.state_id = s.id
            INNER JOIN countries co ON s.country_id = co.id
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
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
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

        // Base query with joins
        let base_query = r#"
            SELECT
                c.id, c.name, c.state_id,
                s.name as state_name, s.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                c.created_at, c.updated_at
            FROM cities c
            INNER JOIN states s ON c.state_id = s.id
            INNER JOIN countries co ON s.country_id = co.id
        "#;

        let count_base = "SELECT COUNT(*) FROM cities c INNER JOIN states s ON c.state_id = s.id";

        let cities = match (search_pattern.as_ref(), state_id) {
            (Some(pattern), Some(state)) => {
                let q = format!("{} WHERE c.name ILIKE $1 AND c.state_id = $2 ORDER BY c.name LIMIT $3 OFFSET $4", base_query);
                sqlx::query_as::<_, CityWithStateDto>(&q)
                    .bind(pattern)
                    .bind(state)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (Some(pattern), None) => {
                let q = format!(
                    "{} WHERE c.name ILIKE $1 ORDER BY c.name LIMIT $2 OFFSET $3",
                    base_query
                );
                sqlx::query_as::<_, CityWithStateDto>(&q)
                    .bind(pattern)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (None, Some(state)) => {
                let q = format!(
                    "{} WHERE c.state_id = $1 ORDER BY c.name LIMIT $2 OFFSET $3",
                    base_query
                );
                sqlx::query_as::<_, CityWithStateDto>(&q)
                    .bind(state)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (None, None) => {
                let q = format!("{} ORDER BY c.name LIMIT $1 OFFSET $2", base_query);
                sqlx::query_as::<_, CityWithStateDto>(&q)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
        };

        let total: i64 = match (search_pattern.as_ref(), state_id) {
            (Some(pattern), Some(state)) => sqlx::query_scalar(&format!(
                "{} WHERE c.name ILIKE $1 AND c.state_id = $2",
                count_base
            ))
            .bind(pattern)
            .bind(state)
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?,
            (Some(pattern), None) => {
                sqlx::query_scalar(&format!("{} WHERE c.name ILIKE $1", count_base))
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (None, Some(state)) => {
                sqlx::query_scalar(&format!("{} WHERE c.state_id = $1", count_base))
                    .bind(state)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(Self::map_err)?
            }
            (None, None) => sqlx::query_scalar(count_base)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?,
        };

        Ok((cities, total))
    }
}
