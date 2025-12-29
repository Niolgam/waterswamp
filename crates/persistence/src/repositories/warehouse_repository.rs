use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{
    MaterialDto, MaterialGroupDto, MaterialWithGroupDto, MovementType, RequisitionDto,
    RequisitionItemDto, RequisitionStatus, RequisitionWithDetailsDto, StockMovementDto,
    StockMovementWithDetailsDto, WarehouseDto, WarehouseStockDto, WarehouseStockWithDetailsDto,
    WarehouseWithCityDto,
};
use domain::ports::{
    MaterialGroupRepositoryPort, MaterialRepositoryPort, RequisitionItemRepositoryPort,
    RequisitionRepositoryPort, StockMovementRepositoryPort, WarehouseRepositoryPort,
    WarehouseStockRepositoryPort,
};
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
        let mut where_clauses: Vec<String> = vec![];
        if search_pattern.is_some() {
            where_clauses.push("(name ILIKE $1 OR code ILIKE $1)".to_string());
        }
        if is_personnel_exclusive.is_some() {
            let idx = if search_pattern.is_some() { 2 } else { 1 };
            where_clauses.push(format!("is_personnel_exclusive = ${}", idx));
        }
        if is_active.is_some() {
            let idx = match (search_pattern.is_some(), is_personnel_exclusive.is_some()) {
                (true, true) => 3,
                (true, false) | (false, true) => 2,
                (false, false) => 1,
            };
            where_clauses.push(format!("is_active = ${}", idx));
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
                    return RepositoryError::Database(db_err.message().to_string());
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
        let mut where_clauses: Vec<String> = vec![];
        if search_pattern.is_some() {
            where_clauses.push("m.name ILIKE $1".to_string());
        }
        if material_group_id.is_some() {
            let idx = if search_pattern.is_some() { 2 } else { 1 };
            where_clauses.push(format!("m.material_group_id = ${}", idx));
        }
        if is_active.is_some() {
            let idx = match (search_pattern.is_some(), material_group_id.is_some()) {
                (true, true) => 3,
                (true, false) | (false, true) => 2,
                (false, false) => 1,
            };
            where_clauses.push(format!("m.is_active = ${}", idx));
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

// ============================
// Warehouse Repository
// ============================

#[derive(Clone)]
pub struct WarehouseRepository {
    pool: PgPool,
}

impl WarehouseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                } else if code == "23503" {
                    return RepositoryError::Database(db_err.message().to_string());
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl WarehouseRepositoryPort for WarehouseRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseDto>(
            "SELECT id, name, code, city_id, responsible_user_id, address, phone, email, is_active, created_at, updated_at
             FROM warehouses WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_city_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseWithCityDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseWithCityDto>(
            "SELECT w.id, w.name, w.code, w.city_id, c.name as city_name, s.code as state_code,
                    w.responsible_user_id, u.username as responsible_username,
                    w.address, w.phone, w.email, w.is_active, w.created_at, w.updated_at
             FROM warehouses w
             INNER JOIN cities c ON w.city_id = c.id
             INNER JOIN states s ON c.state_id = s.id
             LEFT JOIN users u ON w.responsible_user_id = u.id
             WHERE w.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM warehouses WHERE code = $1")
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
            sqlx::query_scalar("SELECT COUNT(*) FROM warehouses WHERE code = $1 AND id != $2")
                .bind(code)
                .bind(exclude_id)
                .fetch_one(&self.pool)
                .await
                .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &str,
        code: &str,
        city_id: Uuid,
        responsible_user_id: Option<Uuid>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> Result<WarehouseDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseDto>(
            "INSERT INTO warehouses (name, code, city_id, responsible_user_id, address, phone, email)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, name, code, city_id, responsible_user_id, address, phone, email, is_active, created_at, updated_at",
        )
        .bind(name)
        .bind(code)
        .bind(city_id)
        .bind(responsible_user_id)
        .bind(address)
        .bind(phone)
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        code: Option<&str>,
        city_id: Option<Uuid>,
        responsible_user_id: Option<Uuid>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<WarehouseDto, RepositoryError> {
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
        if city_id.is_some() {
            query_parts.push(format!("city_id = ${}", bind_index));
            bind_index += 1;
        }
        if responsible_user_id.is_some() {
            query_parts.push(format!("responsible_user_id = ${}", bind_index));
            bind_index += 1;
        }
        if address.is_some() {
            query_parts.push(format!("address = ${}", bind_index));
            bind_index += 1;
        }
        if phone.is_some() {
            query_parts.push(format!("phone = ${}", bind_index));
            bind_index += 1;
        }
        if email.is_some() {
            query_parts.push(format!("email = ${}", bind_index));
            bind_index += 1;
        }
        if is_active.is_some() {
            query_parts.push(format!("is_active = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE warehouses SET {} WHERE id = ${}
             RETURNING id, name, code, city_id, responsible_user_id, address, phone, email, is_active, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, WarehouseDto>(&query_str);

        if let Some(name_val) = name {
            query = query.bind(name_val);
        }
        if let Some(code_val) = code {
            query = query.bind(code_val);
        }
        if let Some(city_val) = city_id {
            query = query.bind(city_val);
        }
        if let Some(resp_val) = responsible_user_id {
            query = query.bind(resp_val);
        }
        if let Some(addr_val) = address {
            query = query.bind(addr_val);
        }
        if let Some(phone_val) = phone {
            query = query.bind(phone_val);
        }
        if let Some(email_val) = email {
            query = query.bind(email_val);
        }
        if let Some(active_val) = is_active {
            query = query.bind(active_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM warehouses WHERE id = $1")
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
        is_active: Option<bool>,
    ) -> Result<(Vec<WarehouseWithCityDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let mut where_clauses: Vec<String> = vec![];
        if search_pattern.is_some() {
            where_clauses.push("(w.name ILIKE $1 OR w.code ILIKE $1)".to_string());
        }
        if city_id.is_some() {
            let idx = if search_pattern.is_some() { 2 } else { 1 };
            where_clauses.push(format!("w.city_id = ${}", idx));
        }
        if is_active.is_some() {
            let idx = match (search_pattern.is_some(), city_id.is_some()) {
                (true, true) => 3,
                (true, false) | (false, true) => 2,
                (false, false) => 1,
            };
            where_clauses.push(format!("w.is_active = ${}", idx));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let param_offset = match (search_pattern.is_some(), city_id.is_some(), is_active.is_some()) {
            (true, true, true) => 4,
            (true, true, false) | (true, false, true) | (false, true, true) => 3,
            (true, false, false) | (false, true, false) | (false, false, true) => 2,
            (false, false, false) => 1,
        };

        let query_str = format!(
            "SELECT w.id, w.name, w.code, w.city_id, c.name as city_name, s.code as state_code,
                    w.responsible_user_id, u.username as responsible_username,
                    w.address, w.phone, w.email, w.is_active, w.created_at, w.updated_at
             FROM warehouses w
             INNER JOIN cities c ON w.city_id = c.id
             INNER JOIN states s ON c.state_id = s.id
             LEFT JOIN users u ON w.responsible_user_id = u.id
             {} ORDER BY w.name LIMIT ${} OFFSET ${}",
            where_clause, param_offset, param_offset + 1
        );

        let mut query = sqlx::query_as::<_, WarehouseWithCityDto>(&query_str);

        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(city_val) = city_id {
            query = query.bind(city_val);
        }
        if let Some(active_val) = is_active {
            query = query.bind(active_val);
        }
        query = query.bind(limit).bind(offset);

        let warehouses = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        let count_query_str = format!("SELECT COUNT(*) FROM warehouses w {}", where_clause);
        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(city_val) = city_id {
            count_query = count_query.bind(city_val);
        }
        if let Some(active_val) = is_active {
            count_query = count_query.bind(active_val);
        }

        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(Self::map_err)?;

        Ok((warehouses, total))
    }
}

// ============================
// Warehouse Stock Repository
// ============================

#[derive(Clone)]
pub struct WarehouseStockRepository {
    pool: PgPool,
}

impl WarehouseStockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                } else if code == "23503" {
                    return RepositoryError::Database(db_err.message().to_string());
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl WarehouseStockRepositoryPort for WarehouseStockRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseStockDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            "SELECT id, warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location, created_at, updated_at
             FROM warehouse_stocks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseStockWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockWithDetailsDto>(
            "SELECT ws.id, ws.warehouse_id, w.name as warehouse_name,
                    ws.material_id, m.name as material_name, mg.name as material_group_name,
                    m.unit_of_measure, ws.quantity, ws.average_unit_value,
                    (ws.quantity * ws.average_unit_value) as total_value,
                    ws.min_stock, ws.max_stock, ws.location, ws.created_at, ws.updated_at
             FROM warehouse_stocks ws
             INNER JOIN warehouses w ON ws.warehouse_id = w.id
             INNER JOIN materials m ON ws.material_id = m.id
             INNER JOIN material_groups mg ON m.material_group_id = mg.id
             WHERE ws.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_warehouse_and_material(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
    ) -> Result<Option<WarehouseStockDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            "SELECT id, warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location, created_at, updated_at
             FROM warehouse_stocks WHERE warehouse_id = $1 AND material_id = $2",
        )
        .bind(warehouse_id)
        .bind(material_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn create(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
        quantity: rust_decimal::Decimal,
        average_unit_value: rust_decimal::Decimal,
        min_stock: Option<rust_decimal::Decimal>,
        max_stock: Option<rust_decimal::Decimal>,
        location: Option<&str>,
    ) -> Result<WarehouseStockDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            "INSERT INTO warehouse_stocks (warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location, created_at, updated_at",
        )
        .bind(warehouse_id)
        .bind(material_id)
        .bind(quantity)
        .bind(average_unit_value)
        .bind(min_stock)
        .bind(max_stock)
        .bind(location)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        min_stock: Option<rust_decimal::Decimal>,
        max_stock: Option<rust_decimal::Decimal>,
        location: Option<&str>,
    ) -> Result<WarehouseStockDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if min_stock.is_some() {
            query_parts.push(format!("min_stock = ${}", bind_index));
            bind_index += 1;
        }
        if max_stock.is_some() {
            query_parts.push(format!("max_stock = ${}", bind_index));
            bind_index += 1;
        }
        if location.is_some() {
            query_parts.push(format!("location = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            "UPDATE warehouse_stocks SET {} WHERE id = ${}
             RETURNING id, warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location, created_at, updated_at",
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, WarehouseStockDto>(&query_str);

        if let Some(min_val) = min_stock {
            query = query.bind(min_val);
        }
        if let Some(max_val) = max_stock {
            query = query.bind(max_val);
        }
        if let Some(loc_val) = location {
            query = query.bind(loc_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    // Método crítico para atualizar quantidade e média ponderada
    async fn update_stock_and_average(
        &self,
        id: Uuid,
        new_quantity: rust_decimal::Decimal,
        new_average: rust_decimal::Decimal,
    ) -> Result<WarehouseStockDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            "UPDATE warehouse_stocks SET quantity = $1, average_unit_value = $2
             WHERE id = $3
             RETURNING id, warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location, created_at, updated_at",
        )
        .bind(new_quantity)
        .bind(new_average)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM warehouse_stocks WHERE id = $1")
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
        warehouse_id: Option<Uuid>,
        material_id: Option<Uuid>,
        search: Option<String>,
        low_stock: Option<bool>,
    ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let mut where_clauses = vec![];
        let mut param_idx = 1;

        if search_pattern.is_some() {
            where_clauses.push(format!("m.name ILIKE ${}", param_idx));
            param_idx += 1;
        }
        if warehouse_id.is_some() {
            where_clauses.push(format!("ws.warehouse_id = ${}", param_idx));
            param_idx += 1;
        }
        if material_id.is_some() {
            where_clauses.push(format!("ws.material_id = ${}", param_idx));
            param_idx += 1;
        }
        if low_stock == Some(true) {
            where_clauses.push("ws.min_stock IS NOT NULL AND ws.quantity <= ws.min_stock".to_string());
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let query_str = format!(
            "SELECT ws.id, ws.warehouse_id, w.name as warehouse_name,
                    ws.material_id, m.name as material_name, mg.name as material_group_name,
                    m.unit_of_measure, ws.quantity, ws.average_unit_value,
                    (ws.quantity * ws.average_unit_value) as total_value,
                    ws.min_stock, ws.max_stock, ws.location, ws.created_at, ws.updated_at
             FROM warehouse_stocks ws
             INNER JOIN warehouses w ON ws.warehouse_id = w.id
             INNER JOIN materials m ON ws.material_id = m.id
             INNER JOIN material_groups mg ON m.material_group_id = mg.id
             {} ORDER BY w.name, m.name LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );

        let mut query = sqlx::query_as::<_, WarehouseStockWithDetailsDto>(&query_str);

        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(wh_id) = warehouse_id {
            query = query.bind(wh_id);
        }
        if let Some(mat_id) = material_id {
            query = query.bind(mat_id);
        }
        query = query.bind(limit).bind(offset);

        let stocks = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        let count_query_str = format!(
            "SELECT COUNT(*) FROM warehouse_stocks ws
             INNER JOIN materials m ON ws.material_id = m.id {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(wh_id) = warehouse_id {
            count_query = count_query.bind(wh_id);
        }
        if let Some(mat_id) = material_id {
            count_query = count_query.bind(mat_id);
        }

        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(Self::map_err)?;

        Ok((stocks, total))
    }
}

// ============================
// Stock Movement Repository Implementation
// ============================

#[derive(Clone)]
pub struct StockMovementRepository {
    pool: PgPool,
}

impl StockMovementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(err: sqlx::Error) -> RepositoryError {
        match err {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(constraint) = db_err.constraint() {
                    RepositoryError::Duplicate(constraint.to_string())
                } else {
                    RepositoryError::Database(db_err.to_string())
                }
            }
            _ => RepositoryError::Database(err.to_string()),
        }
    }
}

#[async_trait]
impl StockMovementRepositoryPort for StockMovementRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StockMovementDto>, RepositoryError> {
        sqlx::query_as::<_, StockMovementDto>(
            "SELECT id, warehouse_stock_id, movement_type, quantity, unit_value, total_value,
                    balance_before, balance_after, average_before, average_after,
                    movement_date, document_number, requisition_id, user_id, notes, created_at
             FROM stock_movements
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<StockMovementWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, StockMovementWithDetailsDto>(
            "SELECT sm.id, sm.warehouse_stock_id,
                    ws.warehouse_id, w.name as warehouse_name,
                    ws.material_id, m.name as material_name, mg.name as material_group_name,
                    m.unit_of_measure,
                    sm.movement_type, sm.quantity, sm.unit_value, sm.total_value,
                    sm.balance_before, sm.balance_after, sm.average_before, sm.average_after,
                    sm.movement_date, sm.document_number, sm.requisition_id,
                    sm.user_id, u.username as user_username,
                    sm.notes, sm.created_at
             FROM stock_movements sm
             INNER JOIN warehouse_stocks ws ON sm.warehouse_stock_id = ws.id
             INNER JOIN warehouses w ON ws.warehouse_id = w.id
             INNER JOIN materials m ON ws.material_id = m.id
             INNER JOIN material_groups mg ON m.material_group_id = mg.id
             INNER JOIN users u ON sm.user_id = u.id
             WHERE sm.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn create(
        &self,
        warehouse_stock_id: Uuid,
        movement_type: MovementType,
        quantity: rust_decimal::Decimal,
        unit_value: rust_decimal::Decimal,
        total_value: rust_decimal::Decimal,
        balance_before: rust_decimal::Decimal,
        balance_after: rust_decimal::Decimal,
        average_before: rust_decimal::Decimal,
        average_after: rust_decimal::Decimal,
        movement_date: chrono::DateTime<chrono::Utc>,
        document_number: Option<&str>,
        requisition_id: Option<Uuid>,
        user_id: Uuid,
        notes: Option<&str>,
    ) -> Result<StockMovementDto, RepositoryError> {
        sqlx::query_as::<_, StockMovementDto>(
            "INSERT INTO stock_movements (
                warehouse_stock_id, movement_type, quantity, unit_value, total_value,
                balance_before, balance_after, average_before, average_after,
                movement_date, document_number, requisition_id, user_id, notes
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             RETURNING id, warehouse_stock_id, movement_type, quantity, unit_value, total_value,
                       balance_before, balance_after, average_before, average_after,
                       movement_date, document_number, requisition_id, user_id, notes, created_at",
        )
        .bind(warehouse_stock_id)
        .bind(movement_type)
        .bind(quantity)
        .bind(unit_value)
        .bind(total_value)
        .bind(balance_before)
        .bind(balance_after)
        .bind(average_before)
        .bind(average_after)
        .bind(movement_date)
        .bind(document_number)
        .bind(requisition_id)
        .bind(user_id)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        warehouse_id: Option<Uuid>,
        material_id: Option<Uuid>,
        movement_type: Option<MovementType>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<StockMovementWithDetailsDto>, i64), RepositoryError> {
        // Build dynamic WHERE clause
        let mut where_clauses: Vec<String> = vec![];
        let mut param_idx = 1;

        if warehouse_id.is_some() {
            where_clauses.push(format!("ws.warehouse_id = ${}", param_idx));
            param_idx += 1;
        }
        if material_id.is_some() {
            where_clauses.push(format!("ws.material_id = ${}", param_idx));
            param_idx += 1;
        }
        if movement_type.is_some() {
            where_clauses.push(format!("sm.movement_type = ${}", param_idx));
            param_idx += 1;
        }
        if start_date.is_some() {
            where_clauses.push(format!("sm.movement_date >= ${}", param_idx));
            param_idx += 1;
        }
        if end_date.is_some() {
            where_clauses.push(format!("sm.movement_date <= ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Main query
        let query_str = format!(
            "SELECT sm.id, sm.warehouse_stock_id,
                    ws.warehouse_id, w.name as warehouse_name,
                    ws.material_id, m.name as material_name, mg.name as material_group_name,
                    m.unit_of_measure,
                    sm.movement_type, sm.quantity, sm.unit_value, sm.total_value,
                    sm.balance_before, sm.balance_after, sm.average_before, sm.average_after,
                    sm.movement_date, sm.document_number, sm.requisition_id,
                    sm.user_id, u.username as user_username,
                    sm.notes, sm.created_at
             FROM stock_movements sm
             INNER JOIN warehouse_stocks ws ON sm.warehouse_stock_id = ws.id
             INNER JOIN warehouses w ON ws.warehouse_id = w.id
             INNER JOIN materials m ON ws.material_id = m.id
             INNER JOIN material_groups mg ON m.material_group_id = mg.id
             INNER JOIN users u ON sm.user_id = u.id
             {} ORDER BY sm.movement_date DESC, sm.created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );

        let mut query = sqlx::query_as::<_, StockMovementWithDetailsDto>(&query_str);

        if let Some(ref wh_id) = warehouse_id {
            query = query.bind(wh_id);
        }
        if let Some(ref mat_id) = material_id {
            query = query.bind(mat_id);
        }
        if let Some(ref mv_type) = movement_type {
            query = query.bind(mv_type);
        }
        if let Some(ref start) = start_date {
            query = query.bind(start);
        }
        if let Some(ref end) = end_date {
            query = query.bind(end);
        }
        query = query.bind(limit).bind(offset);

        let movements = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count query
        let count_query_str = format!(
            "SELECT COUNT(*) FROM stock_movements sm
             INNER JOIN warehouse_stocks ws ON sm.warehouse_stock_id = ws.id
             {}",
            where_clause
        );

        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref wh_id) = warehouse_id {
            count_query = count_query.bind(wh_id);
        }
        if let Some(ref mat_id) = material_id {
            count_query = count_query.bind(mat_id);
        }
        if let Some(ref mv_type) = movement_type {
            count_query = count_query.bind(mv_type);
        }
        if let Some(ref start) = start_date {
            count_query = count_query.bind(start);
        }
        if let Some(ref end) = end_date {
            count_query = count_query.bind(end);
        }

        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(Self::map_err)?;

        Ok((movements, total))
    }
}

// ============================
// Requisition Repository Implementation
// ============================

#[derive(Clone)]
pub struct RequisitionRepository {
    pool: PgPool,
}

impl RequisitionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(err: sqlx::Error) -> RepositoryError {
        match err {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(constraint) = db_err.constraint() {
                    RepositoryError::Duplicate(constraint.to_string())
                } else {
                    RepositoryError::Database(db_err.to_string())
                }
            }
            _ => RepositoryError::Database(err.to_string()),
        }
    }
}

#[async_trait]
impl RequisitionRepositoryPort for RequisitionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RequisitionDto>, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "SELECT id, warehouse_id, requester_id, status, total_value, request_date,
                    approved_by, approved_at, fulfilled_by, fulfilled_at, rejection_reason,
                    notes, created_at, updated_at
             FROM requisitions
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<RequisitionWithDetailsDto>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct RequisitionRow {
            id: Uuid,
            warehouse_id: Uuid,
            warehouse_name: String,
            requester_id: Uuid,
            requester_username: String,
            status: RequisitionStatus,
            total_value: rust_decimal::Decimal,
            request_date: chrono::DateTime<chrono::Utc>,
            approved_by: Option<Uuid>,
            approved_by_username: Option<String>,
            approved_at: Option<chrono::DateTime<chrono::Utc>>,
            fulfilled_by: Option<Uuid>,
            fulfilled_by_username: Option<String>,
            fulfilled_at: Option<chrono::DateTime<chrono::Utc>>,
            rejection_reason: Option<String>,
            notes: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, RequisitionRow>(
            "SELECT r.id, r.warehouse_id, w.name as warehouse_name,
                    r.requester_id, u1.username as requester_username,
                    r.status, r.total_value, r.request_date,
                    r.approved_by, u2.username as approved_by_username, r.approved_at,
                    r.fulfilled_by, u3.username as fulfilled_by_username, r.fulfilled_at,
                    r.rejection_reason, r.notes, r.created_at, r.updated_at
             FROM requisitions r
             INNER JOIN warehouses w ON r.warehouse_id = w.id
             INNER JOIN users u1 ON r.requester_id = u1.id
             LEFT JOIN users u2 ON r.approved_by = u2.id
             LEFT JOIN users u3 ON r.fulfilled_by = u3.id
             WHERE r.id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(row.map(|r| RequisitionWithDetailsDto {
            id: r.id,
            warehouse_id: r.warehouse_id,
            warehouse_name: r.warehouse_name,
            requester_id: r.requester_id,
            requester_username: r.requester_username,
            status: r.status,
            total_value: r.total_value,
            request_date: r.request_date,
            approved_by: r.approved_by,
            approved_by_username: r.approved_by_username,
            approved_at: r.approved_at,
            fulfilled_by: r.fulfilled_by,
            fulfilled_by_username: r.fulfilled_by_username,
            fulfilled_at: r.fulfilled_at,
            rejection_reason: r.rejection_reason,
            notes: r.notes,
            items: vec![], // Items will be fetched separately by service layer
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn create(
        &self,
        warehouse_id: Uuid,
        requester_id: Uuid,
        total_value: rust_decimal::Decimal,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "INSERT INTO requisitions (warehouse_id, requester_id, total_value, notes)
             VALUES ($1, $2, $3, $4)
             RETURNING id, warehouse_id, requester_id, status, total_value, request_date,
                       approved_by, approved_at, fulfilled_by, fulfilled_at, rejection_reason,
                       notes, created_at, updated_at",
        )
        .bind(warehouse_id)
        .bind(requester_id)
        .bind(total_value)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: RequisitionStatus,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "UPDATE requisitions SET status = $1
             WHERE id = $2
             RETURNING id, warehouse_id, requester_id, status, total_value, request_date,
                       approved_by, approved_at, fulfilled_by, fulfilled_at, rejection_reason,
                       notes, created_at, updated_at",
        )
        .bind(status)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn approve(
        &self,
        id: Uuid,
        approved_by: Uuid,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "UPDATE requisitions
             SET status = 'APROVADA', approved_by = $1, approved_at = NOW(), notes = COALESCE($2, notes)
             WHERE id = $3
             RETURNING id, warehouse_id, requester_id, status, total_value, request_date,
                       approved_by, approved_at, fulfilled_by, fulfilled_at, rejection_reason,
                       notes, created_at, updated_at",
        )
        .bind(approved_by)
        .bind(notes)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn reject(
        &self,
        id: Uuid,
        rejection_reason: &str,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "UPDATE requisitions
             SET status = 'REJEITADA', rejection_reason = $1
             WHERE id = $2
             RETURNING id, warehouse_id, requester_id, status, total_value, request_date,
                       approved_by, approved_at, fulfilled_by, fulfilled_at, rejection_reason,
                       notes, created_at, updated_at",
        )
        .bind(rejection_reason)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn fulfill(
        &self,
        id: Uuid,
        fulfilled_by: Uuid,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "UPDATE requisitions
             SET status = 'ATENDIDA', fulfilled_by = $1, fulfilled_at = NOW(), notes = COALESCE($2, notes)
             WHERE id = $3
             RETURNING id, warehouse_id, requester_id, status, total_value, request_date,
                       approved_by, approved_at, fulfilled_by, fulfilled_at, rejection_reason,
                       notes, created_at, updated_at",
        )
        .bind(fulfilled_by)
        .bind(notes)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        warehouse_id: Option<Uuid>,
        requester_id: Option<Uuid>,
        status: Option<RequisitionStatus>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<RequisitionWithDetailsDto>, i64), RepositoryError> {
        // Build dynamic WHERE clause
        let mut where_clauses: Vec<String> = vec![];
        let mut param_idx = 1;

        if warehouse_id.is_some() {
            where_clauses.push(format!("r.warehouse_id = ${}", param_idx));
            param_idx += 1;
        }
        if requester_id.is_some() {
            where_clauses.push(format!("r.requester_id = ${}", param_idx));
            param_idx += 1;
        }
        if status.is_some() {
            where_clauses.push(format!("r.status = ${}", param_idx));
            param_idx += 1;
        }
        if start_date.is_some() {
            where_clauses.push(format!("r.request_date >= ${}", param_idx));
            param_idx += 1;
        }
        if end_date.is_some() {
            where_clauses.push(format!("r.request_date <= ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Reuse the same row struct from find_with_details_by_id
        #[derive(sqlx::FromRow)]
        struct RequisitionRow {
            id: Uuid,
            warehouse_id: Uuid,
            warehouse_name: String,
            requester_id: Uuid,
            requester_username: String,
            status: RequisitionStatus,
            total_value: rust_decimal::Decimal,
            request_date: chrono::DateTime<chrono::Utc>,
            approved_by: Option<Uuid>,
            approved_by_username: Option<String>,
            approved_at: Option<chrono::DateTime<chrono::Utc>>,
            fulfilled_by: Option<Uuid>,
            fulfilled_by_username: Option<String>,
            fulfilled_at: Option<chrono::DateTime<chrono::Utc>>,
            rejection_reason: Option<String>,
            notes: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        // Main query
        let query_str = format!(
            "SELECT r.id, r.warehouse_id, w.name as warehouse_name,
                    r.requester_id, u1.username as requester_username,
                    r.status, r.total_value, r.request_date,
                    r.approved_by, u2.username as approved_by_username, r.approved_at,
                    r.fulfilled_by, u3.username as fulfilled_by_username, r.fulfilled_at,
                    r.rejection_reason, r.notes, r.created_at, r.updated_at
             FROM requisitions r
             INNER JOIN warehouses w ON r.warehouse_id = w.id
             INNER JOIN users u1 ON r.requester_id = u1.id
             LEFT JOIN users u2 ON r.approved_by = u2.id
             LEFT JOIN users u3 ON r.fulfilled_by = u3.id
             {} ORDER BY r.request_date DESC, r.created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );

        let mut query = sqlx::query_as::<_, RequisitionRow>(&query_str);

        if let Some(ref wh_id) = warehouse_id {
            query = query.bind(wh_id);
        }
        if let Some(ref req_id) = requester_id {
            query = query.bind(req_id);
        }
        if let Some(ref st) = status {
            query = query.bind(st);
        }
        if let Some(ref start) = start_date {
            query = query.bind(start);
        }
        if let Some(ref end) = end_date {
            query = query.bind(end);
        }
        query = query.bind(limit).bind(offset);

        let rows = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        let requisitions: Vec<RequisitionWithDetailsDto> = rows.into_iter().map(|r| RequisitionWithDetailsDto {
            id: r.id,
            warehouse_id: r.warehouse_id,
            warehouse_name: r.warehouse_name,
            requester_id: r.requester_id,
            requester_username: r.requester_username,
            status: r.status,
            total_value: r.total_value,
            request_date: r.request_date,
            approved_by: r.approved_by,
            approved_by_username: r.approved_by_username,
            approved_at: r.approved_at,
            fulfilled_by: r.fulfilled_by,
            fulfilled_by_username: r.fulfilled_by_username,
            fulfilled_at: r.fulfilled_at,
            rejection_reason: r.rejection_reason,
            notes: r.notes,
            items: vec![], // Items will be fetched separately by service layer
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        // Count query
        let count_query_str = format!(
            "SELECT COUNT(*) FROM requisitions r {}",
            where_clause
        );

        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref wh_id) = warehouse_id {
            count_query = count_query.bind(wh_id);
        }
        if let Some(ref req_id) = requester_id {
            count_query = count_query.bind(req_id);
        }
        if let Some(ref st) = status {
            count_query = count_query.bind(st);
        }
        if let Some(ref start) = start_date {
            count_query = count_query.bind(start);
        }
        if let Some(ref end) = end_date {
            count_query = count_query.bind(end);
        }

        let total: i64 = count_query.fetch_one(&self.pool).await.map_err(Self::map_err)?;

        Ok((requisitions, total))
    }
}

// ============================
// Requisition Item Repository Implementation
// ============================

#[derive(Clone)]
pub struct RequisitionItemRepository {
    pool: PgPool,
}

impl RequisitionItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(err: sqlx::Error) -> RepositoryError {
        match err {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(constraint) = db_err.constraint() {
                    RepositoryError::Duplicate(constraint.to_string())
                } else {
                    RepositoryError::Database(db_err.to_string())
                }
            }
            _ => RepositoryError::Database(err.to_string()),
        }
    }
}

#[async_trait]
impl RequisitionItemRepositoryPort for RequisitionItemRepository {
    async fn create(
        &self,
        requisition_id: Uuid,
        material_id: Uuid,
        requested_quantity: rust_decimal::Decimal,
        unit_value: rust_decimal::Decimal,
        total_value: rust_decimal::Decimal,
    ) -> Result<RequisitionItemDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionItemDto>(
            "INSERT INTO requisition_items (requisition_id, material_id, requested_quantity, unit_value, total_value)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, requisition_id, material_id, requested_quantity, fulfilled_quantity,
                       unit_value, total_value, created_at",
        )
        .bind(requisition_id)
        .bind(material_id)
        .bind(requested_quantity)
        .bind(unit_value)
        .bind(total_value)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update_fulfilled_quantity(
        &self,
        id: Uuid,
        fulfilled_quantity: rust_decimal::Decimal,
    ) -> Result<RequisitionItemDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionItemDto>(
            "UPDATE requisition_items SET fulfilled_quantity = $1
             WHERE id = $2
             RETURNING id, requisition_id, material_id, requested_quantity, fulfilled_quantity,
                       unit_value, total_value, created_at",
        )
        .bind(fulfilled_quantity)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_requisition_id(
        &self,
        requisition_id: Uuid,
    ) -> Result<Vec<RequisitionItemDto>, RepositoryError> {
        sqlx::query_as::<_, RequisitionItemDto>(
            "SELECT id, requisition_id, material_id, requested_quantity, fulfilled_quantity,
                    unit_value, total_value, created_at
             FROM requisition_items
             WHERE requisition_id = $1
             ORDER BY created_at",
        )
        .bind(requisition_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)
    }
}
