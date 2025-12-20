use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{
    BuildingDto, BuildingTypeDto, BuildingWithRelationsDto, FloorDto, FloorWithRelationsDto,
    SiteDto, SiteTypeDto, SiteWithRelationsDto, SpaceDto, SpaceTypeDto, SpaceWithRelationsDto,
};
use domain::ports::{
    BuildingRepositoryPort, BuildingTypeRepositoryPort, FloorRepositoryPort, SiteRepositoryPort,
    SiteTypeRepositoryPort, SpaceRepositoryPort, SpaceTypeRepositoryPort,
};
use domain::value_objects::LocationName;
use sqlx::PgPool;
use uuid::Uuid;

// ... (SiteType, BuildingType, SpaceType repositories unchanged) ...
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
            "SELECT id, name, description, icon, color, created_at, updated_at FROM site_types WHERE id = $1",
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
            "SELECT id, name, description, icon, color, created_at, updated_at FROM site_types WHERE name = $1",
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
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
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
                "SELECT id, name, description, icon, color, created_at, updated_at FROM site_types
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
                "SELECT id, name, description, icon, color, created_at, updated_at FROM site_types
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

// ============================
// Building Type Repository
// ============================

#[derive(Clone)]
pub struct BuildingTypeRepository {
    pool: PgPool,
}

impl BuildingTypeRepository {
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
impl BuildingTypeRepositoryPort for BuildingTypeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BuildingTypeDto>, RepositoryError> {
        sqlx::query_as::<_, BuildingTypeDto>(
            "SELECT id, name, description, icon, color, created_at, updated_at FROM building_types WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<BuildingTypeDto>, RepositoryError> {
        sqlx::query_as::<_, BuildingTypeDto>(
            "SELECT id, name, description, icon, color, created_at, updated_at FROM building_types WHERE name = $1",
        )
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_name(&self, name: &LocationName) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM building_types WHERE name = $1")
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
            sqlx::query_scalar("SELECT COUNT(*) FROM building_types WHERE name = $1 AND id != $2")
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
    ) -> Result<BuildingTypeDto, RepositoryError> {
        sqlx::query_as::<_, BuildingTypeDto>(
            "INSERT INTO building_types (name, description) VALUES ($1, $2)
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
    ) -> Result<BuildingTypeDto, RepositoryError> {
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
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE building_types SET {} WHERE id = ${} RETURNING id, name, description, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, BuildingTypeDto>(&query_str);

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
        let result = sqlx::query("DELETE FROM building_types WHERE id = $1")
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
    ) -> Result<(Vec<BuildingTypeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let building_types = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, BuildingTypeDto>(
                "SELECT id, name, description, icon, color, created_at, updated_at FROM building_types
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
            sqlx::query_as::<_, BuildingTypeDto>(
                "SELECT id, name, description, icon, color, created_at, updated_at FROM building_types
                 ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar("SELECT COUNT(*) FROM building_types WHERE name ILIKE $1")
                .bind(pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM building_types")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        };

        Ok((building_types, total))
    }
}

// ============================
// Space Type Repository
// ============================

#[derive(Clone)]
pub struct SpaceTypeRepository {
    pool: PgPool,
}

impl SpaceTypeRepository {
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
impl SpaceTypeRepositoryPort for SpaceTypeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpaceTypeDto>, RepositoryError> {
        sqlx::query_as::<_, SpaceTypeDto>(
            "SELECT id, name, description, icon, color, created_at, updated_at FROM space_types WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<SpaceTypeDto>, RepositoryError> {
        sqlx::query_as::<_, SpaceTypeDto>(
            "SELECT id, name, description, icon, color, created_at, updated_at FROM space_types WHERE name = $1",
        )
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_name(&self, name: &LocationName) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM space_types WHERE name = $1")
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
            sqlx::query_scalar("SELECT COUNT(*) FROM space_types WHERE name = $1 AND id != $2")
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
    ) -> Result<SpaceTypeDto, RepositoryError> {
        sqlx::query_as::<_, SpaceTypeDto>(
            "INSERT INTO space_types (name, description) VALUES ($1, $2)
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
    ) -> Result<SpaceTypeDto, RepositoryError> {
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
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE space_types SET {} WHERE id = ${} RETURNING id, name, description, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, SpaceTypeDto>(&query_str);

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
        let result = sqlx::query("DELETE FROM space_types WHERE id = $1")
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
    ) -> Result<(Vec<SpaceTypeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let space_types = if let Some(ref pattern) = search_pattern {
            sqlx::query_as::<_, SpaceTypeDto>(
                "SELECT id, name, description, icon, color, created_at, updated_at FROM space_types
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
            sqlx::query_as::<_, SpaceTypeDto>(
                "SELECT id, name, description, icon, color, created_at, updated_at FROM space_types
                 ORDER BY name LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?
        };

        let total: i64 = if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar("SELECT COUNT(*) FROM space_types WHERE name ILIKE $1")
                .bind(pattern)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM space_types")
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?
        };

        Ok((space_types, total))
    }
}

// =============================================================================
// Site Repository (Phase 3A)
// =============================================================================

#[derive(Clone)]
pub struct SiteRepository {
    pool: PgPool,
}

impl SiteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(err: sqlx::Error) -> RepositoryError {
        match &err {
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        return RepositoryError::Duplicate(
                            db_err
                                .message()
                                .to_string()
                                .split("DETAIL:")
                                .nth(1)
                                .unwrap_or("Duplicate key violation")
                                .trim()
                                .to_string(),
                        );
                    } else if code == "23503" {
                        return RepositoryError::Database(
                            "Foreign key constraint violated".to_string(),
                        );
                    }
                }
            }
            sqlx::Error::RowNotFound => return RepositoryError::NotFound,
            _ => {}
        }
        RepositoryError::Database(err.to_string())
    }
}

#[async_trait]
impl SiteRepositoryPort for SiteRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiteDto>, RepositoryError> {
        let site = sqlx::query_as::<_, SiteDto>(
            "SELECT
                id, name, city_id, site_type_id, address, code,
                ST_AsGeoJSON(bounds)::json as bounds,
                ST_AsGeoJSON(center)::json as center,
                default_zoom, created_at, updated_at
             FROM sites WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(site)
    }

    async fn find_with_relations_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SiteWithRelationsDto>, RepositoryError> {
        let site = sqlx::query_as::<_, SiteWithRelationsDto>(
            "SELECT
                s.id, s.name,
                s.city_id, c.name as city_name,
                st.id as state_id, st.name as state_name, st.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                s.site_type_id, stype.name as site_type_name,
                s.address, s.code,
                ST_AsGeoJSON(s.bounds)::json as bounds,
                ST_AsGeoJSON(s.center)::json as center,
                s.default_zoom,
                s.created_at, s.updated_at
             FROM sites s
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             INNER JOIN site_types stype ON s.site_type_id = stype.id
             WHERE s.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(site)
    }

    async fn create(
        &self,
        name: &LocationName,
        city_id: Uuid,
        site_type_id: Uuid,
        address: Option<&str>,
    ) -> Result<SiteDto, RepositoryError> {
        let site = sqlx::query_as::<_, SiteDto>(
            "INSERT INTO sites (name, city_id, site_type_id, address)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, city_id, site_type_id, address, code, bounds, center, default_zoom, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(city_id)
        .bind(site_type_id)
        .bind(address)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(site)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        city_id: Option<Uuid>,
        site_type_id: Option<Uuid>,
        address: Option<&str>,
    ) -> Result<SiteDto, RepositoryError> {
        // First, fetch current values
        let current = self
            .find_by_id(id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        // Use provided values or keep current ones
        let final_name = name.unwrap_or(&current.name);
        let final_city_id = city_id.unwrap_or(current.city_id);
        let final_site_type_id = site_type_id.unwrap_or(current.site_type_id);
        let final_address = address.or(current.address.as_deref());

        let site = sqlx::query_as::<_, SiteDto>(
            "UPDATE sites
             SET name = $1, city_id = $2, site_type_id = $3, address = $4, updated_at = NOW()
             WHERE id = $5
             RETURNING id, name, city_id, site_type_id, address, code, bounds, center, default_zoom, created_at, updated_at",
        )
        .bind(final_name.as_str())
        .bind(final_city_id)
        .bind(final_site_type_id)
        .bind(final_address)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(site)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM sites WHERE id = $1")
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
        city_id: Option<Uuid>,
        site_type_id: Option<Uuid>,
    ) -> Result<(Vec<SiteWithRelationsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        // Build query dynamically based on filters
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!("s.name ILIKE ${}", bind_index));
            bind_index += 1;
        }
        if city_id.is_some() {
            conditions.push(format!("s.city_id = ${}", bind_index));
            bind_index += 1;
        }
        if site_type_id.is_some() {
            conditions.push(format!("s.site_type_id = ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "SELECT
                s.id, s.name,
                s.city_id, c.name as city_name,
                st.id as state_id, st.name as state_name, st.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                s.site_type_id, stype.name as site_type_name,
                s.address, s.code,
                ST_AsGeoJSON(s.bounds)::json as bounds,
                ST_AsGeoJSON(s.center)::json as center,
                s.default_zoom,
                s.created_at, s.updated_at
             FROM sites s
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             INNER JOIN site_types stype ON s.site_type_id = stype.id
             {}
             ORDER BY s.name LIMIT ${} OFFSET ${}",
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut query = sqlx::query_as::<_, SiteWithRelationsDto>(&query_str);

        // Bind parameters in order
        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(cid) = city_id {
            query = query.bind(cid);
        }
        if let Some(stid) = site_type_id {
            query = query.bind(stid);
        }
        query = query.bind(limit).bind(offset);

        let sites = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count total
        let count_query_str = format!("SELECT COUNT(*) FROM sites s {}", where_clause);

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(cid) = city_id {
            count_query = count_query.bind(cid);
        }
        if let Some(stid) = site_type_id {
            count_query = count_query.bind(stid);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok((sites, total))
    }
}

// ============================
// Building Repository (Phase 3B)
// ============================

#[derive(Clone)]
pub struct BuildingRepository {
    pool: PgPool,
}

impl BuildingRepository {
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
impl BuildingRepositoryPort for BuildingRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BuildingDto>, RepositoryError> {
        sqlx::query_as::<_, BuildingDto>(
            "SELECT id, name, site_id, building_type_id, description, created_at, updated_at
             FROM buildings
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_relations_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<BuildingWithRelationsDto>, RepositoryError> {
        let building = sqlx::query_as::<_, BuildingWithRelationsDto>(
            "SELECT
                b.id, b.name,
                b.site_id, s.name as site_name,
                s.city_id, c.name as city_name,
                st.id as state_id, st.name as state_name, st.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                b.building_type_id, bt.name as building_type_name,
                b.description, b.code, b.total_floors,
                ST_AsGeoJSON(b.coordinates)::json as coordinates,
                b.created_at, b.updated_at
             FROM buildings b
             INNER JOIN sites s ON b.site_id = s.id
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             INNER JOIN building_types bt ON b.building_type_id = bt.id
             WHERE b.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(building)
    }

    async fn create(
        &self,
        name: &LocationName,
        site_id: Uuid,
        building_type_id: Uuid,
        description: Option<&str>,
    ) -> Result<BuildingDto, RepositoryError> {
        sqlx::query_as::<_, BuildingDto>(
            "INSERT INTO buildings (name, site_id, building_type_id, description)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, site_id, building_type_id, description, created_at, updated_at",
        )
        .bind(name.as_str())
        .bind(site_id)
        .bind(building_type_id)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        site_id: Option<Uuid>,
        building_type_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<BuildingDto, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let final_name = name.unwrap_or(&existing.name);
        let final_site_id = site_id.unwrap_or(existing.site_id);
        let final_building_type_id = building_type_id.unwrap_or(existing.building_type_id);
        let final_description = description.or(existing.description.as_deref());

        sqlx::query_as::<_, BuildingDto>(
            "UPDATE buildings
             SET name = $1, site_id = $2, building_type_id = $3, description = $4
             WHERE id = $5
             RETURNING id, name, site_id, building_type_id, description, created_at, updated_at",
        )
        .bind(final_name.as_str())
        .bind(final_site_id)
        .bind(final_building_type_id)
        .bind(final_description)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM buildings WHERE id = $1")
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
        site_id: Option<Uuid>,
        building_type_id: Option<Uuid>,
    ) -> Result<(Vec<BuildingWithRelationsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!("b.name ILIKE ${}", bind_index));
            bind_index += 1;
        }
        if site_id.is_some() {
            conditions.push(format!("b.site_id = ${}", bind_index));
            bind_index += 1;
        }
        if building_type_id.is_some() {
            conditions.push(format!("b.building_type_id = ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "SELECT
                b.id, b.name,
                b.site_id, s.name as site_name,
                s.city_id, c.name as city_name,
                st.id as state_id, st.name as state_name, st.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                b.building_type_id, bt.name as building_type_name,
                b.description, b.code, b.total_floors,
                ST_AsGeoJSON(b.coordinates)::json as coordinates,
                b.created_at, b.updated_at
             FROM buildings b
             INNER JOIN sites s ON b.site_id = s.id
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             INNER JOIN building_types bt ON b.building_type_id = bt.id
             {}
             ORDER BY b.name LIMIT ${} OFFSET ${}",
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut query = sqlx::query_as::<_, BuildingWithRelationsDto>(&query_str);

        // Bind parameters in order
        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(sid) = site_id {
            query = query.bind(sid);
        }
        if let Some(btid) = building_type_id {
            query = query.bind(btid);
        }
        query = query.bind(limit).bind(offset);

        let buildings = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count total
        let count_query_str = format!("SELECT COUNT(*) FROM buildings b {}", where_clause);

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(sid) = site_id {
            count_query = count_query.bind(sid);
        }
        if let Some(btid) = building_type_id {
            count_query = count_query.bind(btid);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok((buildings, total))
    }
}

// ============================
// Floor Repository (Phase 3C)
// ============================

#[derive(Clone)]
pub struct FloorRepository {
    pool: PgPool,
}

impl FloorRepository {
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
impl FloorRepositoryPort for FloorRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FloorDto>, RepositoryError> {
        sqlx::query_as::<_, FloorDto>(
            "SELECT id, floor_number, building_id, description, created_at, updated_at
             FROM floors
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_relations_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<FloorWithRelationsDto>, RepositoryError> {
        let floor = sqlx::query_as::<_, FloorWithRelationsDto>(
            "SELECT
                f.id, f.floor_number,
                f.building_id, b.name as building_name,
                s.id as site_id, s.name as site_name,
                c.id as city_id, c.name as city_name,
                st.id as state_id, st.name as state_name, st.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                f.description,
                f.created_at, f.updated_at
             FROM floors f
             INNER JOIN buildings b ON f.building_id = b.id
             INNER JOIN sites s ON b.site_id = s.id
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             WHERE f.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(floor)
    }

    async fn create(
        &self,
        floor_number: i32,
        building_id: Uuid,
        description: Option<&str>,
    ) -> Result<FloorDto, RepositoryError> {
        sqlx::query_as::<_, FloorDto>(
            "INSERT INTO floors (floor_number, building_id, description)
             VALUES ($1, $2, $3)
             RETURNING id, floor_number, building_id, description, created_at, updated_at",
        )
        .bind(floor_number)
        .bind(building_id)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        floor_number: Option<i32>,
        building_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<FloorDto, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let final_floor_number = floor_number.unwrap_or(existing.floor_number);
        let final_building_id = building_id.unwrap_or(existing.building_id);
        let final_description = description.or(existing.description.as_deref());

        sqlx::query_as::<_, FloorDto>(
            "UPDATE floors
             SET floor_number = $1, building_id = $2, description = $3
             WHERE id = $4
             RETURNING id, floor_number, building_id, description, created_at, updated_at",
        )
        .bind(final_floor_number)
        .bind(final_building_id)
        .bind(final_description)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM floors WHERE id = $1")
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
        building_id: Option<Uuid>,
    ) -> Result<(Vec<FloorWithRelationsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!(
                "CAST(f.floor_number AS TEXT) ILIKE ${}",
                bind_index
            ));
            bind_index += 1;
        }
        if building_id.is_some() {
            conditions.push(format!("f.building_id = ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "SELECT
                f.id, f.floor_number,
                f.building_id, b.name as building_name,
                s.id as site_id, s.name as site_name,
                c.id as city_id, c.name as city_name,
                st.id as state_id, st.name as state_name, st.code as state_code,
                co.id as country_id, co.name as country_name, co.code as country_code,
                f.description,
                f.created_at, f.updated_at
             FROM floors f
             INNER JOIN buildings b ON f.building_id = b.id
             INNER JOIN sites s ON b.site_id = s.id
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             {}
             ORDER BY b.name, f.floor_number LIMIT ${} OFFSET ${}",
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut query = sqlx::query_as::<_, FloorWithRelationsDto>(&query_str);

        // Bind parameters in order
        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(bid) = building_id {
            query = query.bind(bid);
        }
        query = query.bind(limit).bind(offset);

        let floors = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count total
        let count_query_str = format!("SELECT COUNT(*) FROM floors f {}", where_clause);

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(bid) = building_id {
            count_query = count_query.bind(bid);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok((floors, total))
    }
}

// =============================================================================
// SPACE REPOSITORY (Phase 3D)
// =============================================================================

pub struct SpaceRepository {
    pool: PgPool,
}

impl SpaceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(err: sqlx::Error) -> RepositoryError {
        match err {
            sqlx::Error::Database(db_err) => {
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("unique_space_name_per_floor") {
                        return RepositoryError::Duplicate(
                            "Já existe um espaço com este nome neste andar".to_string(),
                        );
                    }
                    if constraint.contains("fk") || constraint.contains("foreign") {
                        return RepositoryError::Database(
                            "Andar ou tipo de espaço não encontrado".to_string(),
                        );
                    }
                }
                RepositoryError::Database(db_err.to_string())
            }
            _ => RepositoryError::Database(err.to_string()),
        }
    }
}

#[async_trait]
impl SpaceRepositoryPort for SpaceRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpaceDto>, RepositoryError> {
        sqlx::query_as::<_, SpaceDto>(
            r#"
            SELECT id, name, floor_id, space_type_id, description, created_at, updated_at
            FROM spaces
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_relations_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SpaceWithRelationsDto>, RepositoryError> {
        sqlx::query_as::<_, SpaceWithRelationsDto>(
            r#"
            SELECT
                sp.id,
                sp.name,
                sp.floor_id,
                f.floor_number,
                f.building_id,
                b.name AS building_name,
                b.site_id,
                s.name AS site_name,
                s.city_id,
                c.name AS city_name,
                c.state_id,
                st.name AS state_name,
                st.code AS state_code,
                co.id AS country_id,
                co.name AS country_name,
                co.code AS country_code,
                sp.space_type_id,
                spt.name AS space_type_name,
                sp.description,
                sp.code,
                sp.location_type,
                ST_AsGeoJSON(sp.coordinates)::json as coordinates,
                sp.capacity,
                sp.area,
                sp.created_at,
                sp.updated_at
            FROM spaces sp
            INNER JOIN floors f ON sp.floor_id = f.id
            INNER JOIN buildings b ON f.building_id = b.id
            INNER JOIN sites s ON b.site_id = s.id
            INNER JOIN cities c ON s.city_id = c.id
            INNER JOIN states st ON c.state_id = st.id
            INNER JOIN countries co ON st.country_id = co.id
            INNER JOIN space_types spt ON sp.space_type_id = spt.id
            WHERE sp.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn create(
        &self,
        name: &LocationName,
        floor_id: Uuid,
        space_type_id: Uuid,
        description: Option<&str>,
    ) -> Result<SpaceDto, RepositoryError> {
        sqlx::query_as::<_, SpaceDto>(
            r#"
            INSERT INTO spaces (name, floor_id, space_type_id, description)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, floor_id, space_type_id, description, created_at, updated_at
            "#,
        )
        .bind(name.as_ref())
        .bind(floor_id)
        .bind(space_type_id)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        floor_id: Option<Uuid>,
        space_type_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<SpaceDto, RepositoryError> {
        // Fetch current record
        let current = self
            .find_by_id(id)
            .await?
            .ok_or(RepositoryError::NotFound)?;

        // Use provided values or keep current ones
        let final_name = name.unwrap_or(&current.name);
        let final_floor_id = floor_id.unwrap_or(current.floor_id);
        let final_space_type_id = space_type_id.unwrap_or(current.space_type_id);
        let final_description = description.or(current.description.as_deref());

        sqlx::query_as::<_, SpaceDto>(
            r#"
            UPDATE spaces
            SET name = $1, floor_id = $2, space_type_id = $3, description = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING id, name, floor_id, space_type_id, description, created_at, updated_at
            "#,
        )
        .bind(final_name.as_ref())
        .bind(final_floor_id)
        .bind(final_space_type_id)
        .bind(final_description)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM spaces WHERE id = $1")
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
        floor_id: Option<Uuid>,
        space_type_id: Option<Uuid>,
    ) -> Result<(Vec<SpaceWithRelationsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!("sp.name ILIKE ${}", bind_index));
            bind_index += 1;
        }
        if floor_id.is_some() {
            conditions.push(format!("sp.floor_id = ${}", bind_index));
            bind_index += 1;
        }
        if space_type_id.is_some() {
            conditions.push(format!("sp.space_type_id = ${}", bind_index));
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
                sp.id,
                sp.name,
                sp.floor_id,
                f.floor_number,
                f.building_id,
                b.name AS building_name,
                b.site_id,
                s.name AS site_name,
                s.city_id,
                c.name AS city_name,
                c.state_id,
                st.name AS state_name,
                st.code AS state_code,
                co.id AS country_id,
                co.name AS country_name,
                co.code AS country_code,
                sp.space_type_id,
                spt.name AS space_type_name,
                sp.description,
                sp.code,
                sp.location_type,
                ST_AsGeoJSON(sp.coordinates)::json as coordinates,
                sp.capacity,
                sp.area,
                sp.created_at,
                sp.updated_at
             FROM spaces sp
             INNER JOIN floors f ON sp.floor_id = f.id
             INNER JOIN buildings b ON f.building_id = b.id
             INNER JOIN sites s ON b.site_id = s.id
             INNER JOIN cities c ON s.city_id = c.id
             INNER JOIN states st ON c.state_id = st.id
             INNER JOIN countries co ON st.country_id = co.id
             INNER JOIN space_types spt ON sp.space_type_id = spt.id
             {}
             ORDER BY b.name, f.floor_number, sp.name LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut query = sqlx::query_as::<_, SpaceWithRelationsDto>(&query_str);

        // Bind parameters in order
        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(fid) = floor_id {
            query = query.bind(fid);
        }
        if let Some(stid) = space_type_id {
            query = query.bind(stid);
        }
        query = query.bind(limit).bind(offset);

        let spaces = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count total
        let count_query_str = format!("SELECT COUNT(*) FROM spaces sp {}", where_clause);

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(fid) = floor_id {
            count_query = count_query.bind(fid);
        }
        if let Some(stid) = space_type_id {
            count_query = count_query.bind(stid);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok((spaces, total))
    }
}
