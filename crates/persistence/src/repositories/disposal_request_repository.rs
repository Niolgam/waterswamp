use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::warehouse::*,
    ports::warehouse::DisposalRequestRepositoryPort,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DisposalRequestRepository {
    pool: PgPool,
}

impl DisposalRequestRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DisposalRequestRepositoryPort for DisposalRequestRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DisposalRequestDto>, RepositoryError> {
        sqlx::query_as::<_, DisposalRequestDto>(
            "SELECT * FROM disposal_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn find_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<DisposalRequestWithItemsDto>, RepositoryError> {
        let req = match self.find_by_id(id).await? {
            Some(r) => r,
            None => return Ok(None),
        };

        let warehouse_name: Option<String> =
            sqlx::query_scalar("SELECT name FROM warehouses WHERE id = $1")
                .bind(req.warehouse_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let items = self.list_items_by_request(id).await?;

        Ok(Some(DisposalRequestWithItemsDto {
            request: req,
            warehouse_name,
            items,
        }))
    }

    async fn list_by_warehouse(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<DisposalRequestStatus>,
    ) -> Result<(Vec<DisposalRequestDto>, i64), RepositoryError> {
        let rows = sqlx::query_as::<_, DisposalRequestDto>(
            r#"SELECT * FROM disposal_requests
               WHERE warehouse_id = $1
                 AND ($2::disposal_request_status_enum IS NULL OR status = $2)
               ORDER BY requested_at DESC
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
            r#"SELECT COUNT(*) FROM disposal_requests
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
        sei_process_number: &str,
        justification: &str,
        technical_opinion_url: &str,
        notes: Option<&str>,
        requested_by: Uuid,
    ) -> Result<DisposalRequestDto, RepositoryError> {
        sqlx::query_as::<_, DisposalRequestDto>(
            r#"INSERT INTO disposal_requests (
                warehouse_id, sei_process_number, justification,
                technical_opinion_url, notes, requested_by
               ) VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(warehouse_id)
        .bind(sei_process_number)
        .bind(justification)
        .bind(technical_opinion_url)
        .bind(notes)
        .bind(requested_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn create_item(
        &self,
        disposal_request_id: Uuid,
        catalog_item_id: Uuid,
        unit_raw_id: Uuid,
        unit_conversion_id: Option<Uuid>,
        quantity_raw: Decimal,
        conversion_factor: Decimal,
        batch_number: Option<&str>,
        notes: Option<&str>,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"INSERT INTO disposal_request_items (
                disposal_request_id, catalog_item_id, unit_raw_id, unit_conversion_id,
                quantity_raw, conversion_factor, batch_number, notes
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(disposal_request_id)
        .bind(catalog_item_id)
        .bind(unit_raw_id)
        .bind(unit_conversion_id)
        .bind(quantity_raw)
        .bind(conversion_factor)
        .bind(batch_number)
        .bind(notes)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }

    async fn transition_to_signed(
        &self,
        id: Uuid,
        signed_by: Uuid,
    ) -> Result<DisposalRequestDto, RepositoryError> {
        sqlx::query_as::<_, DisposalRequestDto>(
            r#"UPDATE disposal_requests
               SET status = 'SIGNED', signed_by = $2, signed_at = NOW(), updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(signed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn transition_to_cancelled(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
        cancellation_reason: &str,
    ) -> Result<DisposalRequestDto, RepositoryError> {
        sqlx::query_as::<_, DisposalRequestDto>(
            r#"UPDATE disposal_requests
               SET status = 'CANCELLED',
                   cancelled_by = $2,
                   cancelled_at = NOW(),
                   cancellation_reason = $3,
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(cancelled_by)
        .bind(cancellation_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    async fn set_item_movement(
        &self,
        item_id: Uuid,
        movement_id: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE disposal_request_items SET movement_id = $2 WHERE id = $1",
        )
        .bind(item_id)
        .bind(movement_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }
}

impl DisposalRequestRepository {
    async fn list_items_by_request(
        &self,
        disposal_request_id: Uuid,
    ) -> Result<Vec<DisposalRequestItemDto>, RepositoryError> {
        sqlx::query_as::<_, DisposalRequestItemDto>(
            r#"SELECT
                dri.id,
                dri.disposal_request_id,
                dri.catalog_item_id,
                ci.description AS catalog_item_name,
                ci.code AS catalog_item_code,
                dri.unit_raw_id,
                um.symbol AS unit_symbol,
                dri.unit_conversion_id,
                dri.quantity_raw,
                dri.conversion_factor,
                dri.batch_number,
                dri.notes,
                dri.movement_id,
                dri.created_at
               FROM disposal_request_items dri
               LEFT JOIN catmat_items ci ON ci.id = dri.catalog_item_id
               LEFT JOIN units_of_measure um ON um.id = dri.unit_raw_id
               WHERE dri.disposal_request_id = $1
               ORDER BY ci.description"#,
        )
        .bind(disposal_request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }
}
