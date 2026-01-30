use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{CityDto, CityWithStateDto, CountryDto, StateDto, StateWithCountryDto};
use domain::ports::{CityRepositoryPort, CountryRepositoryPort, StateRepositoryPort};
use domain::value_objects::{LocationName, StateCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

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
}

#[async_trait]
impl CountryRepositoryPort for CountryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CountryDto>, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "SELECT id, name, iso2, bacen_code, created_at, updated_at FROM countries WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_iso2(&self, iso2: &str) -> Result<Option<CountryDto>, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "SELECT id, name, iso2, bacen_code, created_at, updated_at FROM countries WHERE iso2 = $1",
        )
        .bind(iso2)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_bacen_code(&self, bacen_code: i32) -> Result<Option<CountryDto>, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "SELECT id, name, iso2, bacen_code, created_at, updated_at FROM countries WHERE bacen_code = $1",
        )
        .bind(bacen_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_iso2(&self, iso2: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE iso2 = $1")
            .bind(iso2)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_iso2_excluding(
        &self,
        iso2: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE iso2 = $1 AND id != $2")
                .bind(iso2)
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_bacen_code(&self, bacen_code: i32) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE bacen_code = $1")
            .bind(bacen_code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_bacen_code_excluding(
        &self,
        bacen_code: i32,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE bacen_code = $1 AND id != $2")
                .bind(bacen_code)
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &LocationName, iso2: &str, bacen_code: i32) -> Result<CountryDto, RepositoryError> {
        sqlx::query_as::<_, CountryDto>(
            "INSERT INTO countries (name, iso2, bacen_code) VALUES ($1, $2, $3) RETURNING id, name, iso2, bacen_code, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(iso2)
        .bind(bacen_code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        iso2: Option<&str>,
        bacen_code: Option<i32>,
    ) -> Result<CountryDto, RepositoryError> {
        let current = self
            .find_by_id(id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let new_name = name.unwrap_or(&current.name);
        let new_iso2 = iso2.unwrap_or(&current.iso2);
        let new_bacen_code = bacen_code.unwrap_or(current.bacen_code);

        sqlx::query_as::<_, CountryDto>(
            "UPDATE countries SET name = $1, iso2 = $2, bacen_code = $3, updated_at = NOW() WHERE id = $4 RETURNING id, name, iso2, bacen_code, created_at, updated_at",
        )
        .bind(new_name.as_str())
        .bind(new_iso2)
        .bind(new_bacen_code)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM countries WHERE id = $1")
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
    ) -> Result<(Vec<CountryDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let countries = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, CountryDto>(
                "SELECT id, name, iso2, bacen_code, created_at, updated_at FROM countries WHERE name ILIKE $1 ORDER BY name LIMIT $2 OFFSET $3",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
        } else {
            sqlx::query_as::<_, CountryDto>(
                "SELECT id, name, iso2, bacen_code, created_at, updated_at FROM countries ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar("SELECT COUNT(*) FROM countries WHERE name ILIKE $1")
                .bind(pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM countries")
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?
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
}

#[async_trait]
impl StateRepositoryPort for StateRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StateDto>, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "SELECT id, name, abbreviation, ibge_code, country_id, created_at, updated_at FROM states WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_with_country_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<StateWithCountryDto>, RepositoryError> {
        sqlx::query_as::<_, StateWithCountryDto>(
            r#"
            SELECT
                s.id, s.name, s.abbreviation, s.ibge_code, s.country_id,
                c.name as country_name, c.iso2 as country_iso2, c.bacen_code as country_bacen_code,
                s.created_at, s.updated_at
            FROM states s
            INNER JOIN countries c ON s.country_id = c.id
            WHERE s.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_abbreviation(&self, abbreviation: &StateCode) -> Result<Option<StateDto>, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "SELECT id, name, abbreviation, ibge_code, country_id, created_at, updated_at FROM states WHERE abbreviation = $1",
        )
        .bind(abbreviation.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_ibge_code(&self, ibge_code: i32) -> Result<Option<StateDto>, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "SELECT id, name, abbreviation, ibge_code, country_id, created_at, updated_at FROM states WHERE ibge_code = $1",
        )
        .bind(ibge_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_abbreviation(&self, abbreviation: &StateCode) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM states WHERE abbreviation = $1")
            .bind(abbreviation.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_abbreviation_in_country(
        &self,
        abbreviation: &StateCode,
        country_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM states WHERE abbreviation = $1 AND country_id = $2"
        )
            .bind(abbreviation.as_str())
            .bind(country_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_abbreviation_excluding(
        &self,
        abbreviation: &StateCode,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM states WHERE abbreviation = $1 AND id != $2")
                .bind(abbreviation.as_str())
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_ibge_code(&self, ibge_code: i32) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM states WHERE ibge_code = $1")
            .bind(ibge_code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_ibge_code_excluding(
        &self,
        ibge_code: i32,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM states WHERE ibge_code = $1 AND id != $2")
                .bind(ibge_code)
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &LocationName,
        abbreviation: &StateCode,
        ibge_code: i32,
        country_id: Uuid,
    ) -> Result<StateDto, RepositoryError> {
        sqlx::query_as::<_, StateDto>(
            "INSERT INTO states (name, abbreviation, ibge_code, country_id) VALUES ($1, $2, $3, $4) RETURNING id, name, abbreviation, ibge_code, country_id, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(abbreviation.as_str())
        .bind(ibge_code)
        .bind(country_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        abbreviation: Option<&StateCode>,
        ibge_code: Option<i32>,
        country_id: Option<Uuid>,
    ) -> Result<StateDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if abbreviation.is_some() {
            query_parts.push(format!("abbreviation = ${}", bind_index));
            bind_index += 1;
        }
        if ibge_code.is_some() {
            query_parts.push(format!("ibge_code = ${}", bind_index));
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
            "UPDATE states SET {} WHERE id = ${} RETURNING id, name, abbreviation, ibge_code, country_id, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, StateDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val.as_str());
        }
        if let Some(abbreviation_val) = abbreviation {
            query = query.bind(abbreviation_val.as_str());
        }
        if let Some(ibge_code_val) = ibge_code {
            query = query.bind(ibge_code_val);
        }
        if let Some(country_id_val) = country_id {
            query = query.bind(country_id_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM states WHERE id = $1")
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
        country_id: Option<Uuid>,
    ) -> Result<(Vec<StateWithCountryDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!(
                "(s.name ILIKE ${} OR s.abbreviation ILIKE ${})",
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
                s.id, s.name, s.abbreviation, s.ibge_code, s.country_id,
                c.name as country_name, c.iso2 as country_iso2, c.bacen_code as country_bacen_code,
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

        let states = query.fetch_all(&self.pool).await.map_err(map_db_error)?;

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
            .map_err(map_db_error)?;

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
}

#[async_trait]
impl CityRepositoryPort for CityRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CityDto>, RepositoryError> {
        sqlx::query_as::<_, CityDto>(
            "SELECT id, name, ibge_code, state_id, created_at, updated_at FROM cities WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_with_state_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CityWithStateDto>, RepositoryError> {
        let result = sqlx::query_as::<_, CityWithStateDto>(
            r#"
            SELECT
                c.id, c.name, c.ibge_code, c.state_id,
                s.name as state_name, s.abbreviation as state_abbreviation, s.ibge_code as state_ibge_code,
                co.id as country_id, co.name as country_name, co.iso2 as country_iso2, co.bacen_code as country_bacen_code,
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
        .map_err(map_db_error)?;

        Ok(result)
    }

    async fn find_by_ibge_code(&self, ibge_code: i32) -> Result<Option<CityDto>, RepositoryError> {
        sqlx::query_as::<_, CityDto>(
            "SELECT id, name, ibge_code, state_id, created_at, updated_at FROM cities WHERE ibge_code = $1",
        )
        .bind(ibge_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_ibge_code(&self, ibge_code: i32) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cities WHERE ibge_code = $1")
            .bind(ibge_code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_ibge_code_excluding(
        &self,
        ibge_code: i32,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM cities WHERE ibge_code = $1 AND id != $2")
                .bind(ibge_code)
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &LocationName,
        ibge_code: i32,
        state_id: Uuid,
    ) -> Result<CityDto, RepositoryError> {
        sqlx::query_as::<_, CityDto>(
            "INSERT INTO cities (name, ibge_code, state_id) VALUES ($1, $2, $3)
             RETURNING id, name, ibge_code, state_id, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(ibge_code)
        .bind(state_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        ibge_code: Option<i32>,
        state_id: Option<Uuid>,
    ) -> Result<CityDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if ibge_code.is_some() {
            query_parts.push(format!("ibge_code = ${}", bind_index));
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
            "UPDATE cities SET {} WHERE id = ${} RETURNING id, name, ibge_code, state_id, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, CityDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val.as_str());
        }
        if let Some(ibge_code_val) = ibge_code {
            query = query.bind(ibge_code_val);
        }
        if let Some(state_id_val) = state_id {
            query = query.bind(state_id_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM cities WHERE id = $1")
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
        state_id: Option<Uuid>,
    ) -> Result<(Vec<CityWithStateDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        // Base query with joins
        let base_query = r#"
            SELECT
                c.id, c.name, c.ibge_code, c.state_id,
                s.name as state_name, s.abbreviation as state_abbreviation, s.ibge_code as state_ibge_code,
                co.id as country_id, co.name as country_name, co.iso2 as country_iso2, co.bacen_code as country_bacen_code,
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
                    .map_err(map_db_error)?
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
                    .map_err(map_db_error)?
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
                    .map_err(map_db_error)?
            }
            (None, None) => {
                let q = format!("{} ORDER BY c.name LIMIT $1 OFFSET $2", base_query);
                sqlx::query_as::<_, CityWithStateDto>(&q)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(map_db_error)?
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
            .map_err(map_db_error)?,
            (Some(pattern), None) => {
                sqlx::query_scalar(&format!("{} WHERE c.name ILIKE $1", count_base))
                    .bind(pattern)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(map_db_error)?
            }
            (None, Some(state)) => {
                sqlx::query_scalar(&format!("{} WHERE c.state_id = $1", count_base))
                    .bind(state)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(map_db_error)?
            }
            (None, None) => sqlx::query_scalar(count_base)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?,
        };

        Ok((cities, total))
    }
}
