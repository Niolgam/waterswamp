use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::fueling::*,
    ports::fueling::*,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct FuelingRepository {
    pool: PgPool,
}

impl FuelingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FuelingRepositoryPort for FuelingRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FuelingDto>, RepositoryError> {
        sqlx::query_as::<_, FuelingDto>("SELECT * FROM fuelings WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<FuelingWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, FuelingWithDetailsDto>(
            r#"SELECT f.id, f.vehicle_id, v.license_plate AS vehicle_license_plate,
                      f.driver_id, d.full_name AS driver_name,
                      f.supplier_id, s.legal_name AS supplier_name,
                      f.fuel_type_id, ft.name AS fuel_type_name,
                      f.fueling_date, f.odometer_km, f.quantity_liters,
                      f.unit_price, f.total_cost, f.notes,
                      f.created_at, f.updated_at
               FROM fuelings f
               LEFT JOIN vehicles v ON v.id = f.vehicle_id
               LEFT JOIN drivers d ON d.id = f.driver_id
               LEFT JOIN suppliers s ON s.id = f.supplier_id
               LEFT JOIN vehicle_fuel_types ft ON ft.id = f.fuel_type_id
               WHERE f.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn create(
        &self,
        vehicle_id: Uuid,
        driver_id: Uuid,
        supplier_id: Option<Uuid>,
        fuel_type_id: Uuid,
        fueling_date: DateTime<Utc>,
        odometer_km: i32,
        quantity_liters: Decimal,
        unit_price: Decimal,
        total_cost: Decimal,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FuelingDto, RepositoryError> {
        sqlx::query_as::<_, FuelingDto>(
            r#"INSERT INTO fuelings (vehicle_id, driver_id, supplier_id, fuel_type_id,
                                     fueling_date, odometer_km, quantity_liters,
                                     unit_price, total_cost, notes,
                                     created_by, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
               RETURNING *"#,
        )
        .bind(vehicle_id)
        .bind(driver_id)
        .bind(supplier_id)
        .bind(fuel_type_id)
        .bind(fueling_date)
        .bind(odometer_km)
        .bind(quantity_liters)
        .bind(unit_price)
        .bind(total_cost)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        fueling_date: Option<DateTime<Utc>>,
        odometer_km: Option<i32>,
        quantity_liters: Option<Decimal>,
        unit_price: Option<Decimal>,
        total_cost: Option<Decimal>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FuelingDto, RepositoryError> {
        sqlx::query_as::<_, FuelingDto>(
            r#"UPDATE fuelings SET
                vehicle_id = COALESCE($2, vehicle_id),
                driver_id = COALESCE($3, driver_id),
                supplier_id = COALESCE($4, supplier_id),
                fuel_type_id = COALESCE($5, fuel_type_id),
                fueling_date = COALESCE($6, fueling_date),
                odometer_km = COALESCE($7, odometer_km),
                quantity_liters = COALESCE($8, quantity_liters),
                unit_price = COALESCE($9, unit_price),
                total_cost = COALESCE($10, total_cost),
                notes = COALESCE($11, notes),
                updated_by = COALESCE($12, updated_by)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(vehicle_id)
        .bind(driver_id)
        .bind(supplier_id)
        .bind(fuel_type_id)
        .bind(fueling_date)
        .bind(odometer_km)
        .bind(quantity_liters)
        .bind(unit_price)
        .bind(total_cost)
        .bind(notes)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM fuelings WHERE id = $1")
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
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
    ) -> Result<(Vec<FuelingWithDetailsDto>, i64), RepositoryError> {
        let mut where_clauses = Vec::new();
        let mut param_index = 1u32;

        if vehicle_id.is_some() {
            where_clauses.push(format!("f.vehicle_id = ${}", param_index));
            param_index += 1;
        }
        if driver_id.is_some() {
            where_clauses.push(format!("f.driver_id = ${}", param_index));
            param_index += 1;
        }
        if supplier_id.is_some() {
            where_clauses.push(format!("f.supplier_id = ${}", param_index));
            param_index += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) AS total FROM fuelings f {}", where_sql);
        let list_sql = format!(
            r#"SELECT f.id, f.vehicle_id, v.license_plate AS vehicle_license_plate,
                      f.driver_id, d.full_name AS driver_name,
                      f.supplier_id, s.legal_name AS supplier_name,
                      f.fuel_type_id, ft.name AS fuel_type_name,
                      f.fueling_date, f.odometer_km, f.quantity_liters,
                      f.unit_price, f.total_cost, f.notes,
                      f.created_at, f.updated_at
               FROM fuelings f
               LEFT JOIN vehicles v ON v.id = f.vehicle_id
               LEFT JOIN drivers d ON d.id = f.driver_id
               LEFT JOIN suppliers s ON s.id = f.supplier_id
               LEFT JOIN vehicle_fuel_types ft ON ft.id = f.fuel_type_id
               {}
               ORDER BY f.fueling_date DESC
               LIMIT ${} OFFSET ${}"#,
            where_sql, param_index, param_index + 1
        );

        let mut count_query = sqlx::query(&count_sql);
        let mut list_query = sqlx::query_as::<_, FuelingWithDetailsDto>(&list_sql);

        if let Some(vid) = vehicle_id {
            count_query = count_query.bind(vid);
            list_query = list_query.bind(vid);
        }
        if let Some(did) = driver_id {
            count_query = count_query.bind(did);
            list_query = list_query.bind(did);
        }
        if let Some(sid) = supplier_id {
            count_query = count_query.bind(sid);
            list_query = list_query.bind(sid);
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
