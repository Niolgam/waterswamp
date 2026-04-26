use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::financial_event::{CreateFinancialEventInput, FinancialEventDto, FinancialEventType},
    ports::financial_event::FinancialEventRepositoryPort,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct FinancialEventRepository {
    pool: PgPool,
}

impl FinancialEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FinancialEventRepositoryPort for FinancialEventRepository {
    async fn create(
        &self,
        input: CreateFinancialEventInput,
    ) -> Result<FinancialEventDto, RepositoryError> {
        sqlx::query_as::<_, FinancialEventDto>(
            r#"INSERT INTO financial_events (
                event_type, invoice_id, invoice_adjustment_id, supplier_id,
                warehouse_id, amount, commitment_number, metadata, created_by
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#,
        )
        .bind(input.event_type)
        .bind(input.invoice_id)
        .bind(input.invoice_adjustment_id)
        .bind(input.supplier_id)
        .bind(input.warehouse_id)
        .bind(input.amount)
        .bind(input.commitment_number)
        .bind(input.metadata)
        .bind(input.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_invoice(
        &self,
        invoice_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FinancialEventDto>, i64), RepositoryError> {
        let total: i64 = sqlx::query(
            "SELECT COUNT(*) FROM financial_events WHERE invoice_id = $1",
        )
        .bind(invoice_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?
        .get(0);

        let rows = sqlx::query_as::<_, FinancialEventDto>(
            r#"SELECT * FROM financial_events
               WHERE invoice_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(invoice_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((rows, total))
    }

    async fn list_by_supplier(
        &self,
        supplier_id: Uuid,
        event_type: Option<FinancialEventType>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FinancialEventDto>, i64), RepositoryError> {
        if let Some(et) = event_type {
            let total: i64 = sqlx::query(
                "SELECT COUNT(*) FROM financial_events WHERE supplier_id = $1 AND event_type = $2",
            )
            .bind(supplier_id)
            .bind(&et)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get(0);

            let rows = sqlx::query_as::<_, FinancialEventDto>(
                r#"SELECT * FROM financial_events
                   WHERE supplier_id = $1 AND event_type = $2
                   ORDER BY created_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(supplier_id)
            .bind(et)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

            Ok((rows, total))
        } else {
            let total: i64 = sqlx::query(
                "SELECT COUNT(*) FROM financial_events WHERE supplier_id = $1",
            )
            .bind(supplier_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get(0);

            let rows = sqlx::query_as::<_, FinancialEventDto>(
                r#"SELECT * FROM financial_events
                   WHERE supplier_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(supplier_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

            Ok((rows, total))
        }
    }
}
