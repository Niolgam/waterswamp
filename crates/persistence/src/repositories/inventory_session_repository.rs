use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::warehouse::*,
    ports::warehouse::InventorySessionRepositoryPort,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct InventorySessionRepository {
    pool: PgPool,
}

impl InventorySessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventorySessionRepositoryPort for InventorySessionRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<InventorySessionDto>, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            "SELECT * FROM inventory_sessions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn find_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<InventorySessionWithItemsDto>, RepositoryError> {
        let session = match self.find_by_id(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let warehouse_name: Option<String> =
            sqlx::query_scalar("SELECT name FROM warehouses WHERE id = $1")
                .bind(session.warehouse_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let items = self.list_items(id).await?;

        Ok(Some(InventorySessionWithItemsDto {
            session,
            warehouse_name,
            items,
        }))
    }

    async fn list_by_warehouse(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<InventorySessionStatus>,
    ) -> Result<(Vec<InventorySessionDto>, i64), RepositoryError> {
        let rows = sqlx::query_as::<_, InventorySessionDto>(
            r#"SELECT * FROM inventory_sessions
               WHERE warehouse_id = $1
                 AND ($2::text IS NULL OR status::text = $2)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(warehouse_id)
        .bind(status.as_ref().map(|s| format!("{:?}", s).to_uppercase()))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM inventory_sessions
               WHERE warehouse_id = $1
                 AND ($2::text IS NULL OR status::text = $2)"#,
        )
        .bind(warehouse_id)
        .bind(status.as_ref().map(|s| format!("{:?}", s).to_uppercase()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn create(
        &self,
        warehouse_id: Uuid,
        tolerance_percentage: Decimal,
        notes: Option<&str>,
        created_by: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            r#"INSERT INTO inventory_sessions (warehouse_id, tolerance_percentage, notes, created_by)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(warehouse_id)
        .bind(tolerance_percentage)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn transition_to_counting(
        &self,
        id: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            r#"UPDATE inventory_sessions
               SET status = 'COUNTING', counting_started_at = NOW(), updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn transition_to_reconciling(
        &self,
        id: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            r#"UPDATE inventory_sessions
               SET status = 'RECONCILING', reconciliation_started_at = NOW(), updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn transition_to_completed(
        &self,
        id: Uuid,
        sei_process_number: Option<&str>,
    ) -> Result<InventorySessionDto, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            r#"UPDATE inventory_sessions
               SET status = 'COMPLETED',
                   sei_process_number = COALESCE($2, sei_process_number),
                   completed_at = NOW(),
                   updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(sei_process_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn transition_to_cancelled(
        &self,
        id: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            r#"UPDATE inventory_sessions
               SET status = 'CANCELLED', cancelled_at = NOW(), updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn confirm_govbr_signature(
        &self,
        id: Uuid,
        signed_by: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError> {
        sqlx::query_as::<_, InventorySessionDto>(
            r#"UPDATE inventory_sessions
               SET govbr_signed_at = NOW(), govbr_signed_by = $2, updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(signed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    /// Cria um item por item de warehouse_stocks do almoxarifado dado.
    async fn snapshot_stock_items(
        &self,
        session_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<usize, RepositoryError> {
        let result = sqlx::query(
            r#"INSERT INTO inventory_session_items (session_id, catalog_item_id, unit_raw_id, system_quantity)
               SELECT $1, ws.catalog_item_id, ci.unit_id, ws.quantity
               FROM warehouse_stocks ws
               JOIN catmat_items ci ON ci.id = ws.catalog_item_id
               WHERE ws.warehouse_id = $2
               ON CONFLICT (session_id, catalog_item_id) DO NOTHING"#,
        )
        .bind(session_id)
        .bind(warehouse_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(result.rows_affected() as usize)
    }

    async fn upsert_item_count(
        &self,
        session_id: Uuid,
        catalog_item_id: Uuid,
        counted_quantity: Decimal,
        notes: Option<&str>,
    ) -> Result<InventorySessionItemDto, RepositoryError> {
        // First ensure item exists
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM inventory_session_items WHERE session_id = $1 AND catalog_item_id = $2",
        )
        .bind(session_id)
        .bind(catalog_item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if existing.is_none() {
            return Err(RepositoryError::NotFound);
        }

        sqlx::query(
            r#"UPDATE inventory_session_items
               SET counted_quantity = $3,
                   notes = COALESCE($4, notes),
                   updated_at = NOW()
               WHERE session_id = $1 AND catalog_item_id = $2"#,
        )
        .bind(session_id)
        .bind(catalog_item_id)
        .bind(counted_quantity)
        .bind(notes)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let item = sqlx::query_as::<_, InventorySessionItemDto>(
            r#"SELECT
                isi.id, isi.session_id, isi.catalog_item_id,
                ci.description AS catalog_item_name,
                ci.code AS catalog_item_code,
                isi.unit_raw_id,
                um.symbol AS unit_symbol,
                isi.system_quantity,
                isi.counted_quantity,
                CASE WHEN isi.counted_quantity IS NOT NULL
                     THEN isi.counted_quantity - isi.system_quantity
                     ELSE NULL END AS divergence,
                CASE WHEN isi.counted_quantity IS NOT NULL AND isi.system_quantity > 0
                     THEN ABS(isi.counted_quantity - isi.system_quantity) / isi.system_quantity
                     ELSE NULL END AS divergence_percentage,
                isi.movement_id,
                isi.notes,
                isi.created_at, isi.updated_at
               FROM inventory_session_items isi
               LEFT JOIN catmat_items ci ON ci.id = isi.catalog_item_id
               LEFT JOIN units_of_measure um ON um.id = isi.unit_raw_id
               WHERE isi.session_id = $1 AND isi.catalog_item_id = $2"#,
        )
        .bind(session_id)
        .bind(catalog_item_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(item)
    }

    async fn list_items(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<InventorySessionItemDto>, RepositoryError> {
        sqlx::query_as::<_, InventorySessionItemDto>(
            r#"SELECT
                isi.id, isi.session_id, isi.catalog_item_id,
                ci.description AS catalog_item_name,
                ci.code AS catalog_item_code,
                isi.unit_raw_id,
                um.symbol AS unit_symbol,
                isi.system_quantity,
                isi.counted_quantity,
                CASE WHEN isi.counted_quantity IS NOT NULL
                     THEN isi.counted_quantity - isi.system_quantity
                     ELSE NULL END AS divergence,
                CASE WHEN isi.counted_quantity IS NOT NULL AND isi.system_quantity > 0
                     THEN ABS(isi.counted_quantity - isi.system_quantity) / isi.system_quantity
                     ELSE NULL END AS divergence_percentage,
                isi.movement_id,
                isi.notes,
                isi.created_at, isi.updated_at
               FROM inventory_session_items isi
               LEFT JOIN catmat_items ci ON ci.id = isi.catalog_item_id
               LEFT JOIN units_of_measure um ON um.id = isi.unit_raw_id
               WHERE isi.session_id = $1
               ORDER BY ci.description"#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn set_item_movement(
        &self,
        item_id: Uuid,
        movement_id: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE inventory_session_items SET movement_id = $2 WHERE id = $1",
        )
        .bind(item_id)
        .bind(movement_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }
}
