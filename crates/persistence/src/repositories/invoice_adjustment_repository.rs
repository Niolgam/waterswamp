use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::invoice_adjustment::*,
    ports::invoice_adjustment::*,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct InvoiceAdjustmentRepository {
    pool: PgPool,
}

impl InvoiceAdjustmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvoiceAdjustmentRepositoryPort for InvoiceAdjustmentRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<InvoiceAdjustmentDto>, RepositoryError> {
        sqlx::query_as::<_, InvoiceAdjustmentDto>(
            "SELECT * FROM invoice_adjustments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_invoice(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceAdjustmentWithItemsDto>, RepositoryError> {
        let adjustments = sqlx::query_as::<_, InvoiceAdjustmentDto>(
            "SELECT * FROM invoice_adjustments WHERE invoice_id = $1 ORDER BY created_at ASC",
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let mut result = Vec::with_capacity(adjustments.len());
        for adj in adjustments {
            #[derive(sqlx::FromRow)]
            struct ItemRow {
                id: Uuid,
                adjustment_id: Uuid,
                invoice_item_id: Uuid,
                catalog_item_name: Option<String>,
                adjusted_quantity: Decimal,
                adjusted_value: Decimal,
                notes: Option<String>,
                created_at: DateTime<Utc>,
            }

            let items = sqlx::query_as::<_, ItemRow>(
                r#"SELECT iai.id, iai.adjustment_id, iai.invoice_item_id,
                          ci.description AS catalog_item_name,
                          iai.adjusted_quantity, iai.adjusted_value,
                          iai.notes, iai.created_at
                   FROM invoice_adjustment_items iai
                   LEFT JOIN invoice_items ii ON ii.id = iai.invoice_item_id
                   LEFT JOIN catmat_items ci ON ci.id = ii.catalog_item_id
                   WHERE iai.adjustment_id = $1
                   ORDER BY iai.created_at ASC"#,
            )
            .bind(adj.id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

            result.push(InvoiceAdjustmentWithItemsDto {
                id: adj.id,
                invoice_id: adj.invoice_id,
                reason: adj.reason,
                created_by: adj.created_by,
                created_at: adj.created_at,
                items: items
                    .into_iter()
                    .map(|r| InvoiceAdjustmentItemDetailDto {
                        id: r.id,
                        adjustment_id: r.adjustment_id,
                        invoice_item_id: r.invoice_item_id,
                        catalog_item_name: r.catalog_item_name,
                        adjusted_quantity: r.adjusted_quantity,
                        adjusted_value: r.adjusted_value,
                        notes: r.notes,
                        created_at: r.created_at,
                    })
                    .collect(),
            });
        }
        Ok(result)
    }

    async fn create(
        &self,
        invoice_id: Uuid,
        reason: &str,
        created_by: Uuid,
    ) -> Result<InvoiceAdjustmentDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceAdjustmentDto>(
            r#"INSERT INTO invoice_adjustments (invoice_id, reason, created_by)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(invoice_id)
        .bind(reason)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn create_item(
        &self,
        adjustment_id: Uuid,
        invoice_item_id: Uuid,
        adjusted_quantity: Decimal,
        adjusted_value: Decimal,
        notes: Option<&str>,
    ) -> Result<InvoiceAdjustmentItemDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceAdjustmentItemDto>(
            r#"INSERT INTO invoice_adjustment_items (
                adjustment_id, invoice_item_id, adjusted_quantity, adjusted_value, notes
               ) VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(adjustment_id)
        .bind(invoice_item_id)
        .bind(adjusted_quantity)
        .bind(adjusted_value)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
