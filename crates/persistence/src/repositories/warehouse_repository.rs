use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::warehouse::*,
    ports::warehouse::*,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================
// Warehouse Repository
// ============================

pub struct WarehouseRepository {
    pool: PgPool,
}

impl WarehouseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WarehouseRepositoryPort for WarehouseRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseDto>("SELECT * FROM warehouses WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseWithDetailsDto>(
            r#"SELECT w.id, w.name, w.code, w.warehouse_type, w.city_id,
                      c.name AS city_name, s.abbreviation AS state_abbreviation,
                      w.responsible_user_id, w.responsible_unit_id,
                      w.allows_transfers, w.is_budgetary,
                      w.address, w.phone, w.email, w.is_active,
                      w.created_at, w.updated_at
               FROM warehouses w
               LEFT JOIN cities c ON c.id = w.city_id
               LEFT JOIN states s ON s.id = c.state_id
               WHERE w.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM warehouses WHERE code = $1) AS exists",
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn exists_by_code_excluding(
        &self,
        code: &str,
        id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM warehouses WHERE code = $1 AND id != $2) AS exists",
        )
        .bind(code)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn create(
        &self,
        name: &str,
        code: &str,
        warehouse_type: WarehouseType,
        city_id: Uuid,
        responsible_user_id: Option<Uuid>,
        responsible_unit_id: Option<Uuid>,
        allows_transfers: bool,
        is_budgetary: bool,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> Result<WarehouseDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseDto>(
            r#"INSERT INTO warehouses (
                name, code, warehouse_type, city_id,
                responsible_user_id, responsible_unit_id,
                allows_transfers, is_budgetary,
                address, phone, email
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *"#,
        )
        .bind(name)
        .bind(code)
        .bind(warehouse_type)
        .bind(city_id)
        .bind(responsible_user_id)
        .bind(responsible_unit_id)
        .bind(allows_transfers)
        .bind(is_budgetary)
        .bind(address)
        .bind(phone)
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        code: Option<&str>,
        warehouse_type: Option<WarehouseType>,
        city_id: Option<Uuid>,
        responsible_user_id: Option<Uuid>,
        responsible_unit_id: Option<Uuid>,
        allows_transfers: Option<bool>,
        is_budgetary: Option<bool>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<WarehouseDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseDto>(
            r#"UPDATE warehouses SET
                name = COALESCE($2, name),
                code = COALESCE($3, code),
                warehouse_type = COALESCE($4, warehouse_type),
                city_id = COALESCE($5, city_id),
                responsible_user_id = COALESCE($6, responsible_user_id),
                responsible_unit_id = COALESCE($7, responsible_unit_id),
                allows_transfers = COALESCE($8, allows_transfers),
                is_budgetary = COALESCE($9, is_budgetary),
                address = COALESCE($10, address),
                phone = COALESCE($11, phone),
                email = COALESCE($12, email),
                is_active = COALESCE($13, is_active)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(code)
        .bind(warehouse_type)
        .bind(city_id)
        .bind(responsible_user_id)
        .bind(responsible_unit_id)
        .bind(allows_transfers)
        .bind(is_budgetary)
        .bind(address)
        .bind(phone)
        .bind(email)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM warehouses WHERE id = $1")
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
        warehouse_type: Option<WarehouseType>,
        city_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<WarehouseWithDetailsDto>, i64), RepositoryError> {
        let mut where_clauses = Vec::new();
        let mut param_index = 1u32;

        if search.is_some() {
            where_clauses.push(format!(
                "(w.name ILIKE ${p} OR w.code ILIKE ${p} OR c.name ILIKE ${p})",
                p = param_index
            ));
            param_index += 1;
        }
        if warehouse_type.is_some() {
            where_clauses.push(format!("w.warehouse_type = ${}", param_index));
            param_index += 1;
        }
        if city_id.is_some() {
            where_clauses.push(format!("w.city_id = ${}", param_index));
            param_index += 1;
        }
        if is_active.is_some() {
            where_clauses.push(format!("w.is_active = ${}", param_index));
            param_index += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!(
            r#"SELECT COUNT(*) AS total FROM warehouses w
               LEFT JOIN cities c ON c.id = w.city_id
               {}"#,
            where_sql
        );
        let list_sql = format!(
            r#"SELECT w.id, w.name, w.code, w.warehouse_type, w.city_id,
                      c.name AS city_name, s.abbreviation AS state_abbreviation,
                      w.responsible_user_id, w.responsible_unit_id,
                      w.allows_transfers, w.is_budgetary,
                      w.address, w.phone, w.email, w.is_active,
                      w.created_at, w.updated_at
               FROM warehouses w
               LEFT JOIN cities c ON c.id = w.city_id
               LEFT JOIN states s ON s.id = c.state_id
               {}
               ORDER BY w.name ASC
               LIMIT ${} OFFSET ${}"#,
            where_sql, param_index, param_index + 1
        );

        let mut count_query = sqlx::query(&count_sql);
        let mut list_query = sqlx::query_as::<_, WarehouseWithDetailsDto>(&list_sql);

        if let Some(ref s) = search {
            let pattern = format!("%{}%", s);
            count_query = count_query.bind(pattern.clone());
            list_query = list_query.bind(pattern);
        }
        if let Some(ref wt) = warehouse_type {
            count_query = count_query.bind(wt);
            list_query = list_query.bind(wt);
        }
        if let Some(cid) = city_id {
            count_query = count_query.bind(cid);
            list_query = list_query.bind(cid);
        }
        if let Some(active) = is_active {
            count_query = count_query.bind(active);
            list_query = list_query.bind(active);
        }

        count_query = count_query.bind(limit);
        list_query = list_query.bind(limit).bind(offset);

        let total: i64 = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get("total");

        let items = list_query
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Warehouse Stock Repository
// ============================

pub struct WarehouseStockRepository {
    pool: PgPool,
}

impl WarehouseStockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WarehouseStockRepositoryPort for WarehouseStockRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseStockDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>("SELECT * FROM warehouse_stocks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseStockWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockWithDetailsDto>(
            r#"SELECT ws.id, ws.warehouse_id, w.name AS warehouse_name,
                      ws.catalog_item_id, ci.description AS catalog_item_name,
                      ci.code AS catalog_item_code,
                      u.symbol AS unit_symbol, u.name AS unit_name,
                      ws.quantity, ws.reserved_quantity,
                      CASE WHEN ws.is_blocked THEN 0
                           ELSE GREATEST(ws.quantity - ws.reserved_quantity, 0)
                      END AS available_quantity,
                      ws.average_unit_value,
                      ws.quantity * ws.average_unit_value AS total_value,
                      ws.min_stock, ws.max_stock, ws.reorder_point, ws.resupply_days,
                      ws.location, ws.secondary_location,
                      ws.is_blocked, ws.block_reason, ws.blocked_at, ws.blocked_by,
                      ws.last_entry_at, ws.last_exit_at, ws.last_inventory_at,
                      ws.created_at, ws.updated_at
               FROM warehouse_stocks ws
               JOIN warehouses w ON w.id = ws.warehouse_id
               JOIN catmat_items ci ON ci.id = ws.catalog_item_id
               LEFT JOIN units_of_measure u ON u.id = ci.unit_of_measure_id
               WHERE ws.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_warehouse_and_item(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
    ) -> Result<Option<WarehouseStockDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            "SELECT * FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_warehouse(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        search: Option<String>,
        is_blocked: Option<bool>,
    ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), RepositoryError> {
        let mut where_clauses = vec!["ws.warehouse_id = $1".to_string()];
        let mut param_index = 2u32;

        if search.is_some() {
            where_clauses.push(format!(
                "(ci.description ILIKE ${p} OR ci.code ILIKE ${p} OR ws.location ILIKE ${p})",
                p = param_index
            ));
            param_index += 1;
        }
        if is_blocked.is_some() {
            where_clauses.push(format!("ws.is_blocked = ${}", param_index));
            param_index += 1;
        }

        let where_sql = format!("WHERE {}", where_clauses.join(" AND "));

        let count_sql = format!(
            r#"SELECT COUNT(*) AS total FROM warehouse_stocks ws
               JOIN catmat_items ci ON ci.id = ws.catalog_item_id
               {}"#,
            where_sql
        );
        let list_sql = format!(
            r#"SELECT ws.id, ws.warehouse_id, w.name AS warehouse_name,
                      ws.catalog_item_id, ci.description AS catalog_item_name,
                      ci.code AS catalog_item_code,
                      u.symbol AS unit_symbol, u.name AS unit_name,
                      ws.quantity, ws.reserved_quantity,
                      CASE WHEN ws.is_blocked THEN 0
                           ELSE GREATEST(ws.quantity - ws.reserved_quantity, 0)
                      END AS available_quantity,
                      ws.average_unit_value,
                      ws.quantity * ws.average_unit_value AS total_value,
                      ws.min_stock, ws.max_stock, ws.reorder_point, ws.resupply_days,
                      ws.location, ws.secondary_location,
                      ws.is_blocked, ws.block_reason, ws.blocked_at, ws.blocked_by,
                      ws.last_entry_at, ws.last_exit_at, ws.last_inventory_at,
                      ws.created_at, ws.updated_at
               FROM warehouse_stocks ws
               JOIN warehouses w ON w.id = ws.warehouse_id
               JOIN catmat_items ci ON ci.id = ws.catalog_item_id
               LEFT JOIN units_of_measure u ON u.id = ci.unit_of_measure_id
               {}
               ORDER BY ci.description ASC
               LIMIT ${} OFFSET ${}"#,
            where_sql, param_index, param_index + 1
        );

        let mut count_query = sqlx::query(&count_sql).bind(warehouse_id);
        let mut list_query =
            sqlx::query_as::<_, WarehouseStockWithDetailsDto>(&list_sql).bind(warehouse_id);

        if let Some(ref s) = search {
            let pattern = format!("%{}%", s);
            count_query = count_query.bind(pattern.clone());
            list_query = list_query.bind(pattern);
        }
        if let Some(blocked) = is_blocked {
            count_query = count_query.bind(blocked);
            list_query = list_query.bind(blocked);
        }

        count_query = count_query.bind(limit);
        list_query = list_query.bind(limit).bind(offset);

        let total: i64 = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get("total");

        let items = list_query
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

        Ok((items, total))
    }

    async fn update_params(
        &self,
        id: Uuid,
        min_stock: Option<Decimal>,
        max_stock: Option<Decimal>,
        reorder_point: Option<Decimal>,
        resupply_days: Option<i32>,
        location: Option<&str>,
        secondary_location: Option<&str>,
    ) -> Result<WarehouseStockDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            r#"UPDATE warehouse_stocks SET
                min_stock = COALESCE($2, min_stock),
                max_stock = COALESCE($3, max_stock),
                reorder_point = COALESCE($4, reorder_point),
                resupply_days = COALESCE($5, resupply_days),
                location = COALESCE($6, location),
                secondary_location = COALESCE($7, secondary_location)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(min_stock)
        .bind(max_stock)
        .bind(reorder_point)
        .bind(resupply_days)
        .bind(location)
        .bind(secondary_location)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn block(
        &self,
        id: Uuid,
        block_reason: &str,
        blocked_by: Uuid,
    ) -> Result<WarehouseStockDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            r#"UPDATE warehouse_stocks SET
                is_blocked = TRUE,
                block_reason = $2,
                blocked_at = NOW(),
                blocked_by = $3
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(block_reason)
        .bind(blocked_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn unblock(&self, id: Uuid) -> Result<WarehouseStockDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseStockDto>(
            r#"UPDATE warehouse_stocks SET
                is_blocked = FALSE,
                block_reason = NULL,
                blocked_at = NULL,
                blocked_by = NULL
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
