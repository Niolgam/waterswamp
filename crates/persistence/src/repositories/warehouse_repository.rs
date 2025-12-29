use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{MaterialDto, MaterialGroupDto, MaterialWithGroupDto};
use domain::ports::{MaterialGroupRepositoryPort, MaterialRepositoryPort};
use domain::value_objects::{CatmatCode, MaterialCode, UnitOfMeasure};
use sqlx::PgPool;
use uuid::Uuid;

// ============================
// Material Group Repository
// ============================

#[derive(Clone)]
pub struct MaterialGroupRepository {
    pool: PgPool,
}

impl MaterialGroupRepository {
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
impl MaterialGroupRepositoryPort for MaterialGroupRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaterialGroupDto>, RepositoryError> {
        sqlx::query_as::<_, MaterialGroupDto>(
            "SELECT id, code, name, description, expense_element, is_personnel_exclusive, is_active, created_at, updated_at
             FROM material_groups WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_code(
        &self,
        code: &MaterialCode,
    ) -> Result<Option<MaterialGroupDto>, RepositoryError> {
        sqlx::query_as::<_, MaterialGroupDto>(
            "SELECT id, code, name, description, expense_element, is_personnel_exclusive, is_active, created_at, updated_at
             FROM material_groups WHERE code = $1",
        )
        .bind(code.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_code(&self, code: &MaterialCode) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM material_groups WHERE code = $1")
            .bind(code.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(
        &self,
        code: &MaterialCode,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM material_groups WHERE code = $1 AND id != $2")
                .bind(code.as_str())
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        code: &MaterialCode,
        name: &str,
        description: Option<&str>,
        expense_element: Option<&str>,
        is_personnel_exclusive: bool,
    ) -> Result<MaterialGroupDto, RepositoryError> {
        sqlx::query_as::<_, MaterialGroupDto>(
            "INSERT INTO material_groups (code, name, description, expense_element, is_personnel_exclusive)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, code, name, description, expense_element, is_personnel_exclusive, is_active, created_at, updated_at",
        )
        .bind(code.as_str())
        .bind(name)
        .bind(description)
        .bind(expense_element)
        .bind(is_personnel_exclusive)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        code: Option<&MaterialCode>,
        name: Option<&str>,
        description: Option<&str>,
        expense_element: Option<&str>,
        is_personnel_exclusive: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<MaterialGroupDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if code.is_some() {
            query_parts.push(format!("code = ${}", bind_index));
            bind_index += 1;
        }
        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if description.is_some() {
            query_parts.push(format!("description = ${}", bind_index));
            bind_index += 1;
        }
        if expense_element.is_some() {
            query_parts.push(format!("expense_element = ${}", bind_index));
            bind_index += 1;
        }
        if is_personnel_exclusive.is_some() {
            query_parts.push(format!("is_personnel_exclusive = ${}", bind_index));
            bind_index += 1;
        }
        if is_active.is_some() {
            query_parts.push(format!("is_active = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE material_groups SET {} WHERE id = ${}
             RETURNING id, code, name, description, expense_element, is_personnel_exclusive, is_active, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, MaterialGroupDto>(&query_str);

        if let Some(code_val) = code {
            query = query.bind(code_val.as_str());
        }
        if let Some(name_val) = name {
            query = query.bind(name_val);
        }
        if let Some(desc_val) = description {
            query = query.bind(desc_val);
        }
        if let Some(expense_val) = expense_element {
            query = query.bind(expense_val);
        }
        if let Some(personnel_val) = is_personnel_exclusive {
            query = query.bind(personnel_val);
        }
        if let Some(active_val) = is_active {
            query = query.bind(active_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM material_groups WHERE id = $1")
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
        is_personnel_exclusive: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(Vec<MaterialGroupDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        // Build the WHERE clause dynamically
        let mut where_clauses = vec![];
        if search_pattern.is_some() {
            where_clauses.push("(name ILIKE $1 OR code ILIKE $1)");
        }
        if is_personnel_exclusive.is_some() {
            let idx = if search_pattern.is_some() { 2 } else { 1 };
            where_clauses.push(&format!("is_personnel_exclusive = ${}", idx));
        }
        if is_active.is_some() {
            let idx = match (search_pattern.is_some(), is_personnel_exclusive.is_some()) {
                (true, true) => 3,
                (true, false) | (false, true) => 2,
                (false, false) => 1,
            };
            where_clauses.push(&format!("is_active = ${}", idx));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Construct the main query
        let param_offset = match (
            search_pattern.is_some(),
            is_personnel_exclusive.is_some(),
            is_active.is_some(),
        ) {
            (true, true, true) => 4,
            (true, true, false) | (true, false, true) | (false, true, true) => 3,
            (true, false, false) | (false, true, false) | (false, false, true) => 2,
            (false, false, false) => 1,
        };

        let query_str = format!(
            "SELECT id, code, name, description, expense_element, is_personnel_exclusive, is_active, created_at, updated_at
             FROM material_groups {} ORDER BY code, name LIMIT ${} OFFSET ${}",
            where_clause, param_offset, param_offset + 1
        );

        let mut query = sqlx::query_as::<_, MaterialGroupDto>(&query_str);

        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(personnel_val) = is_personnel_exclusive {
            query = query.bind(personnel_val);
        }
        if let Some(active_val) = is_active {
            query = query.bind(active_val);
        }
        query = query.bind(limit).bind(offset);

        let material_groups = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count total
        let count_query_str = format!("SELECT COUNT(*) FROM material_groups {}", where_clause);
        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(personnel_val) = is_personnel_exclusive {
            count_query = count_query.bind(personnel_val);
        }
        if let Some(active_val) = is_active {
            count_query = count_query.bind(active_val);
        }

        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(Self::map_err)?;

        Ok((material_groups, total))
    }
}

// ============================
// Material Repository
// ============================

#[derive(Clone)]
pub struct MaterialRepository {
    pool: PgPool,
}

impl MaterialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                } else if code == "23503" {
                    return RepositoryError::ForeignKeyViolation(db_err.message().to_string());
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl MaterialRepositoryPort for MaterialRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaterialDto>, RepositoryError> {
        sqlx::query_as::<_, MaterialDto>(
            "SELECT id, material_group_id, name, estimated_value, unit_of_measure, specification,
                    search_links, catmat_code, photo_url, is_active, created_at, updated_at
             FROM materials WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_group_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<MaterialWithGroupDto>, RepositoryError> {
        sqlx::query_as::<_, MaterialWithGroupDto>(
            "SELECT m.id, m.material_group_id, mg.code as material_group_code, mg.name as material_group_name,
                    m.name, m.estimated_value, m.unit_of_measure, m.specification,
                    m.search_links, m.catmat_code, m.photo_url, m.is_active, m.created_at, m.updated_at
             FROM materials m
             INNER JOIN material_groups mg ON m.material_group_id = mg.id
             WHERE m.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_name_in_group(
        &self,
        name: &str,
        material_group_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM materials WHERE name = $1 AND material_group_id = $2",
        )
        .bind(name)
        .bind(material_group_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_name_in_group_excluding(
        &self,
        name: &str,
        material_group_id: Uuid,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM materials WHERE name = $1 AND material_group_id = $2 AND id != $3",
        )
        .bind(name)
        .bind(material_group_id)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        material_group_id: Uuid,
        name: &str,
        estimated_value: rust_decimal::Decimal,
        unit_of_measure: &UnitOfMeasure,
        specification: &str,
        search_links: Option<&str>,
        catmat_code: Option<&CatmatCode>,
        photo_url: Option<&str>,
    ) -> Result<MaterialDto, RepositoryError> {
        sqlx::query_as::<_, MaterialDto>(
            "INSERT INTO materials (material_group_id, name, estimated_value, unit_of_measure, specification,
                                   search_links, catmat_code, photo_url)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, material_group_id, name, estimated_value, unit_of_measure, specification,
                       search_links, catmat_code, photo_url, is_active, created_at, updated_at",
        )
        .bind(material_group_id)
        .bind(name)
        .bind(estimated_value)
        .bind(unit_of_measure.as_str())
        .bind(specification)
        .bind(search_links)
        .bind(catmat_code.map(|c| c.as_str()))
        .bind(photo_url)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        material_group_id: Option<Uuid>,
        name: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>,
        unit_of_measure: Option<&UnitOfMeasure>,
        specification: Option<&str>,
        search_links: Option<&str>,
        catmat_code: Option<&CatmatCode>,
        photo_url: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<MaterialDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if material_group_id.is_some() {
            query_parts.push(format!("material_group_id = ${}", bind_index));
            bind_index += 1;
        }
        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if estimated_value.is_some() {
            query_parts.push(format!("estimated_value = ${}", bind_index));
            bind_index += 1;
        }
        if unit_of_measure.is_some() {
            query_parts.push(format!("unit_of_measure = ${}", bind_index));
            bind_index += 1;
        }
        if specification.is_some() {
            query_parts.push(format!("specification = ${}", bind_index));
            bind_index += 1;
        }
        if search_links.is_some() {
            query_parts.push(format!("search_links = ${}", bind_index));
            bind_index += 1;
        }
        if catmat_code.is_some() {
            query_parts.push(format!("catmat_code = ${}", bind_index));
            bind_index += 1;
        }
        if photo_url.is_some() {
            query_parts.push(format!("photo_url = ${}", bind_index));
            bind_index += 1;
        }
        if is_active.is_some() {
            query_parts.push(format!("is_active = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE materials SET {} WHERE id = ${}
             RETURNING id, material_group_id, name, estimated_value, unit_of_measure, specification,
                       search_links, catmat_code, photo_url, is_active, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, MaterialDto>(&query_str);

        if let Some(group_id) = material_group_id {
            query = query.bind(group_id);
        }
        if let Some(name_val) = name {
            query = query.bind(name_val);
        }
        if let Some(value) = estimated_value {
            query = query.bind(value);
        }
        if let Some(unit) = unit_of_measure {
            query = query.bind(unit.as_str());
        }
        if let Some(spec) = specification {
            query = query.bind(spec);
        }
        if let Some(links) = search_links {
            query = query.bind(links);
        }
        if let Some(catmat) = catmat_code {
            query = query.bind(catmat.as_str());
        }
        if let Some(photo) = photo_url {
            query = query.bind(photo);
        }
        if let Some(active) = is_active {
            query = query.bind(active);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM materials WHERE id = $1")
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
        material_group_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<MaterialWithGroupDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        // Build the WHERE clause dynamically
        let mut where_clauses = vec![];
        if search_pattern.is_some() {
            where_clauses.push("m.name ILIKE $1");
        }
        if material_group_id.is_some() {
            let idx = if search_pattern.is_some() { 2 } else { 1 };
            where_clauses.push(&format!("m.material_group_id = ${}", idx));
        }
        if is_active.is_some() {
            let idx = match (search_pattern.is_some(), material_group_id.is_some()) {
                (true, true) => 3,
                (true, false) | (false, true) => 2,
                (false, false) => 1,
            };
            where_clauses.push(&format!("m.is_active = ${}", idx));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Construct the main query
        let param_offset = match (
            search_pattern.is_some(),
            material_group_id.is_some(),
            is_active.is_some(),
        ) {
            (true, true, true) => 4,
            (true, true, false) | (true, false, true) | (false, true, true) => 3,
            (true, false, false) | (false, true, false) | (false, false, true) => 2,
            (false, false, false) => 1,
        };

        let query_str = format!(
            "SELECT m.id, m.material_group_id, mg.code as material_group_code, mg.name as material_group_name,
                    m.name, m.estimated_value, m.unit_of_measure, m.specification,
                    m.search_links, m.catmat_code, m.photo_url, m.is_active, m.created_at, m.updated_at
             FROM materials m
             INNER JOIN material_groups mg ON m.material_group_id = mg.id
             {} ORDER BY mg.code, m.name LIMIT ${} OFFSET ${}",
            where_clause, param_offset, param_offset + 1
        );

        let mut query = sqlx::query_as::<_, MaterialWithGroupDto>(&query_str);

        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(group_id) = material_group_id {
            query = query.bind(group_id);
        }
        if let Some(active_val) = is_active {
            query = query.bind(active_val);
        }
        query = query.bind(limit).bind(offset);

        let materials = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count total
        let count_query_str = format!(
            "SELECT COUNT(*) FROM materials m
             INNER JOIN material_groups mg ON m.material_group_id = mg.id {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(group_id) = material_group_id {
            count_query = count_query.bind(group_id);
        }
        if let Some(active_val) = is_active {
            count_query = count_query.bind(active_val);
        }

        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(Self::map_err)?;

        Ok((materials, total))
    }
}
