use async_trait::async_trait;
use chrono::NaiveDate;
use domain::{
    errors::RepositoryError,
    models::batch::*,
    ports::batch::*,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================================================================
// Warehouse Batch Stock Repository
// ============================================================================

pub struct WarehouseBatchStockRepository {
    pool: PgPool,
}

impl WarehouseBatchStockRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WarehouseBatchStockRepositoryPort for WarehouseBatchStockRepository {
    async fn upsert(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        batch_number: &str,
        expiration_date: Option<NaiveDate>,
        quantity_delta: Decimal,
        unit_cost: Decimal,
    ) -> Result<WarehouseBatchStockDto, RepositoryError> {
        sqlx::query_as::<_, WarehouseBatchStockDto>(
            r#"INSERT INTO warehouse_batch_stocks
                (warehouse_id, catalog_item_id, batch_number, expiration_date, quantity, unit_cost)
               VALUES ($1, $2, $3, $4, GREATEST(0, $5), $6)
               ON CONFLICT (warehouse_id, catalog_item_id, batch_number)
               DO UPDATE SET
                quantity = GREATEST(0, warehouse_batch_stocks.quantity + $5),
                unit_cost = CASE
                    WHEN $5 > 0 AND $6 > 0 THEN
                        (warehouse_batch_stocks.quantity * warehouse_batch_stocks.unit_cost
                         + $5 * $6)
                        / NULLIF(warehouse_batch_stocks.quantity + $5, 0)
                    ELSE warehouse_batch_stocks.unit_cost
                END,
                expiration_date = COALESCE(warehouse_batch_stocks.expiration_date, $4),
                updated_at = NOW()
               RETURNING *"#,
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .bind(batch_number)
        .bind(expiration_date)
        .bind(quantity_delta)
        .bind(unit_cost)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_fefo(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
    ) -> Result<Vec<WarehouseBatchStockDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseBatchStockDto>(
            r#"SELECT * FROM warehouse_batch_stocks
               WHERE warehouse_id = $1 AND catalog_item_id = $2
               ORDER BY
                 is_quarantined ASC,           -- non-quarantined first
                 expiration_date ASC NULLS LAST -- FEFO: earliest expiry first
            "#,
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        batch_number: &str,
    ) -> Result<Option<WarehouseBatchStockDto>, RepositoryError> {
        sqlx::query_as::<_, WarehouseBatchStockDto>(
            "SELECT * FROM warehouse_batch_stocks WHERE warehouse_id=$1 AND catalog_item_id=$2 AND batch_number=$3",
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .bind(batch_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn set_quarantine(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        batch_number: &str,
        quarantined: bool,
        reason: Option<&str>,
        quarantined_by: Option<Uuid>,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"UPDATE warehouse_batch_stocks SET
                is_quarantined = $4,
                quarantine_reason = CASE WHEN $4 THEN $5 ELSE NULL END,
                quarantined_at    = CASE WHEN $4 THEN NOW() ELSE NULL END,
                quarantined_by    = CASE WHEN $4 THEN $6 ELSE NULL END,
                updated_at = NOW()
               WHERE warehouse_id = $1 AND catalog_item_id = $2 AND batch_number = $3"#,
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .bind(batch_number)
        .bind(quarantined)
        .bind(reason)
        .bind(quarantined_by)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn list_near_expiry(
        &self,
        warehouse_id: Option<Uuid>,
        days_ahead: i32,
    ) -> Result<Vec<WarehouseBatchStockDto>, RepositoryError> {
        if let Some(wh_id) = warehouse_id {
            sqlx::query_as::<_, WarehouseBatchStockDto>(
                r#"SELECT * FROM warehouse_batch_stocks
                   WHERE warehouse_id = $1
                     AND expiration_date IS NOT NULL
                     AND expiration_date <= CURRENT_DATE + ($2 || ' days')::INTERVAL
                     AND quantity > 0
                   ORDER BY expiration_date ASC"#,
            )
            .bind(wh_id)
            .bind(days_ahead)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
        } else {
            sqlx::query_as::<_, WarehouseBatchStockDto>(
                r#"SELECT * FROM warehouse_batch_stocks
                   WHERE expiration_date IS NOT NULL
                     AND expiration_date <= CURRENT_DATE + ($1 || ' days')::INTERVAL
                     AND quantity > 0
                   ORDER BY expiration_date ASC"#,
            )
            .bind(days_ahead)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
        }
    }
}

// ============================================================================
// Batch Quality Occurrence Repository
// ============================================================================

pub struct BatchQualityOccurrenceRepository {
    pool: PgPool,
}

impl BatchQualityOccurrenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BatchQualityOccurrenceRepositoryPort for BatchQualityOccurrenceRepository {
    async fn create(
        &self,
        payload: &CreateBatchQualityOccurrencePayload,
        reported_by: Uuid,
        quarantine_triggered: bool,
    ) -> Result<BatchQualityOccurrenceDto, RepositoryError> {
        sqlx::query_as::<_, BatchQualityOccurrenceDto>(
            r#"INSERT INTO batch_quality_occurrences (
                warehouse_id, catalog_item_id, batch_number,
                occurrence_type, severity, description,
                evidence_url, sei_process_number,
                reported_by, quarantine_triggered
               ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               RETURNING *"#,
        )
        .bind(payload.warehouse_id)
        .bind(payload.catalog_item_id)
        .bind(&payload.batch_number)
        .bind(&payload.occurrence_type)
        .bind(&payload.severity)
        .bind(&payload.description)
        .bind(payload.evidence_url.as_deref())
        .bind(payload.sei_process_number.as_deref())
        .bind(reported_by)
        .bind(quarantine_triggered)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BatchQualityOccurrenceDto>, RepositoryError> {
        sqlx::query_as::<_, BatchQualityOccurrenceDto>(
            "SELECT * FROM batch_quality_occurrences WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(
        &self,
        warehouse_id: Option<Uuid>,
        catalog_item_id: Option<Uuid>,
        batch_number: Option<&str>,
        status: Option<BatchOccurrenceStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<BatchQualityOccurrenceDto>, i64), RepositoryError> {
        let mut conditions = vec!["1=1".to_string()];
        let mut param = 1usize;

        if warehouse_id.is_some() {
            conditions.push(format!("warehouse_id = ${}", param));
            param += 1;
        }
        if catalog_item_id.is_some() {
            conditions.push(format!("catalog_item_id = ${}", param));
            param += 1;
        }
        if batch_number.is_some() {
            conditions.push(format!("batch_number = ${}", param));
            param += 1;
        }
        if status.is_some() {
            conditions.push(format!("status = ${}", param));
            param += 1;
        }

        let where_clause = conditions.join(" AND ");
        let count_sql = format!("SELECT COUNT(*) FROM batch_quality_occurrences WHERE {}", where_clause);
        let list_sql = format!(
            "SELECT * FROM batch_quality_occurrences WHERE {} ORDER BY occurred_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param, param + 1
        );

        let mut count_q = sqlx::query(&count_sql);
        let mut list_q = sqlx::query_as::<_, BatchQualityOccurrenceDto>(&list_sql);

        if let Some(v) = warehouse_id {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }
        if let Some(v) = catalog_item_id {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }
        if let Some(v) = batch_number {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }
        if let Some(ref v) = status {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }

        count_q = count_q.bind(limit).bind(offset);
        list_q = list_q.bind(limit).bind(offset);

        let total: i64 = count_q
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get(0);

        let rows = list_q
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

        Ok((rows, total))
    }

    async fn resolve(
        &self,
        id: Uuid,
        payload: &ResolveOccurrencePayload,
        resolved_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, RepositoryError> {
        sqlx::query_as::<_, BatchQualityOccurrenceDto>(
            r#"UPDATE batch_quality_occurrences SET
                status = 'RESOLVED',
                corrective_action = $2,
                resolved_notes = $3,
                resolved_at = NOW(),
                resolved_by = $4,
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(&payload.corrective_action)
        .bind(payload.resolved_notes.as_deref())
        .bind(resolved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn close(
        &self,
        id: Uuid,
        payload: &CloseOccurrencePayload,
        closed_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, RepositoryError> {
        sqlx::query_as::<_, BatchQualityOccurrenceDto>(
            r#"UPDATE batch_quality_occurrences SET
                status = 'CLOSED',
                resolved_notes = COALESCE($2, resolved_notes),
                closed_at = NOW(),
                closed_by = $3,
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(payload.resolved_notes.as_deref())
        .bind(closed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
