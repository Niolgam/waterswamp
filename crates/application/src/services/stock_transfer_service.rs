/// StockTransferService — RF-018: Transferência entre Almoxarifados
///
/// Implements a two-step transfer flow with pessimistic locking (RN-011):
///   1. `initiate_transfer()` — source warehouse manager starts the transfer,
///      generating a TRANSFER_OUT movement and reserving stock.
///   2. `confirm_transfer()` — destination manager confirms receipt,
///      generating a TRANSFER_IN movement atomically.
///   3. `reject_transfer()` — destination rejects; source reservation is released
///      via a compensatory TRANSFER_IN (returns the stock).
///   4. `cancel_transfer()` — source cancels before confirmation.

use crate::errors::ServiceError;
use crate::services::stock_movement_service::{ProcessMovementInput, StockMovementService, StockMovementType};
use chrono::Utc;
use domain::models::warehouse::{
    CancelTransferPayload, ConfirmTransferPayload, InitiateTransferPayload, RejectTransferPayload,
    StockTransferDto, StockTransferItemDto, StockTransferStatus, StockTransferWithItemsDto,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct StockTransferService {
    pool: PgPool,
    stock_movement_service: Arc<StockMovementService>,
}

impl StockTransferService {
    pub fn new(pool: PgPool, stock_movement_service: Arc<StockMovementService>) -> Self {
        Self {
            pool,
            stock_movement_service,
        }
    }

    // ========================================================================
    // READ
    // ========================================================================

    pub async fn get_transfer(&self, id: Uuid) -> Result<StockTransferWithItemsDto, ServiceError> {
        let transfer = sqlx::query_as::<_, StockTransferDto>(
            r#"SELECT
                t.id, t.transfer_number,
                t.source_warehouse_id,
                sw.name AS source_warehouse_name,
                t.destination_warehouse_id,
                dw.name AS destination_warehouse_name,
                t.status,
                t.notes, t.rejection_reason, t.cancellation_reason,
                t.initiated_by, t.confirmed_by, t.rejected_by, t.cancelled_by,
                t.initiated_at, t.confirmed_at, t.rejected_at, t.cancelled_at,
                t.expires_at, t.created_at, t.updated_at
               FROM stock_transfers t
               LEFT JOIN warehouses sw ON sw.id = t.source_warehouse_id
               LEFT JOIN warehouses dw ON dw.id = t.destination_warehouse_id
               WHERE t.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or(ServiceError::NotFound("Transferência não encontrada".to_string()))?;

        let items = self.get_transfer_items(id).await?;

        Ok(StockTransferWithItemsDto { transfer, items })
    }

    pub async fn list_transfers(
        &self,
        limit: i64,
        offset: i64,
        source_warehouse_id: Option<Uuid>,
        destination_warehouse_id: Option<Uuid>,
        status: Option<StockTransferStatus>,
    ) -> Result<(Vec<StockTransferDto>, i64), ServiceError> {
        let transfers = sqlx::query_as::<_, StockTransferDto>(
            r#"SELECT
                t.id, t.transfer_number,
                t.source_warehouse_id,
                sw.name AS source_warehouse_name,
                t.destination_warehouse_id,
                dw.name AS destination_warehouse_name,
                t.status,
                t.notes, t.rejection_reason, t.cancellation_reason,
                t.initiated_by, t.confirmed_by, t.rejected_by, t.cancelled_by,
                t.initiated_at, t.confirmed_at, t.rejected_at, t.cancelled_at,
                t.expires_at, t.created_at, t.updated_at
               FROM stock_transfers t
               LEFT JOIN warehouses sw ON sw.id = t.source_warehouse_id
               LEFT JOIN warehouses dw ON dw.id = t.destination_warehouse_id
               WHERE ($1::UUID IS NULL OR t.source_warehouse_id = $1)
                 AND ($2::UUID IS NULL OR t.destination_warehouse_id = $2)
                 AND ($3::stock_transfer_status_enum IS NULL OR t.status = $3)
               ORDER BY t.created_at DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(source_warehouse_id)
        .bind(destination_warehouse_id)
        .bind(status.as_ref().map(|s| format!("{:?}", s).to_uppercase()))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM stock_transfers t
               WHERE ($1::UUID IS NULL OR t.source_warehouse_id = $1)
                 AND ($2::UUID IS NULL OR t.destination_warehouse_id = $2)
                 AND ($3::stock_transfer_status_enum IS NULL OR t.status = $3)"#,
        )
        .bind(source_warehouse_id)
        .bind(destination_warehouse_id)
        .bind(status.as_ref().map(|s| format!("{:?}", s).to_uppercase()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok((transfers, total))
    }

    async fn get_transfer_items(
        &self,
        transfer_id: Uuid,
    ) -> Result<Vec<StockTransferItemDto>, ServiceError> {
        sqlx::query_as::<_, StockTransferItemDto>(
            r#"SELECT
                ti.id, ti.transfer_id, ti.catalog_item_id,
                ci.description AS catalog_item_name,
                ci.catmat_code AS catalog_item_code,
                ti.quantity_requested, ti.quantity_confirmed,
                ti.unit_raw_id,
                u.symbol AS unit_symbol,
                ti.conversion_factor, ti.batch_number, ti.expiration_date, ti.notes
               FROM stock_transfer_items ti
               LEFT JOIN catmat_items ci ON ci.id = ti.catalog_item_id
               LEFT JOIN units_of_measure u ON u.id = ti.unit_raw_id
               WHERE ti.transfer_id = $1"#,
        )
        .bind(transfer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))
    }

    // ========================================================================
    // INITIATE (Step 1)
    // ========================================================================

    /// Initiate a transfer from source warehouse (RF-018 step 1).
    /// Generates TRANSFER_OUT movements on the source warehouse (pessimistic locking via StockMovementService).
    pub async fn initiate_transfer(
        &self,
        source_warehouse_id: Uuid,
        payload: InitiateTransferPayload,
        initiated_by: Uuid,
    ) -> Result<StockTransferWithItemsDto, ServiceError> {
        if source_warehouse_id == payload.destination_warehouse_id {
            return Err(ServiceError::BadRequest(
                "Almoxarifado de origem e destino não podem ser iguais".to_string(),
            ));
        }

        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Informe ao menos um item para a transferência".to_string(),
            ));
        }

        // Validate destination warehouse exists
        let dest_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM warehouses WHERE id = $1 AND is_active = TRUE)",
        )
        .bind(payload.destination_warehouse_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        if !dest_exists {
            return Err(ServiceError::NotFound(
                "Almoxarifado de destino não encontrado ou inativo".to_string(),
            ));
        }

        let expires_at = payload
            .expires_in_hours
            .map(|h| Utc::now() + chrono::Duration::hours(h));

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Create transfer record
        let transfer_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO stock_transfers (
                transfer_number, source_warehouse_id, destination_warehouse_id,
                status, notes, initiated_by, expires_at
               ) VALUES (
                '', $1, $2, 'PENDING', $3, $4, $5
               ) RETURNING id"#,
        )
        .bind(source_warehouse_id)
        .bind(payload.destination_warehouse_id)
        .bind(payload.notes.as_deref())
        .bind(initiated_by)
        .bind(expires_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Process TRANSFER_OUT for each item
        for item in &payload.items {
            if item.quantity_raw <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade deve ser maior que zero".to_string(),
                ));
            }

            let quantity_base = item.quantity_raw * item.conversion_factor;

            // Generate TRANSFER_OUT movement (this acquires pessimistic lock on warehouse_stocks)
            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id: source_warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: StockMovementType::TransferOut,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: item.unit_conversion_id,
                        quantity_raw: item.quantity_raw,
                        conversion_factor: item.conversion_factor,
                        quantity_base,
                        unit_price_base: Decimal::ZERO, // uses average cost
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        document_number: None, // filled after insert with transfer_number
                        notes: item.notes.clone(),
                        user_id: initiated_by,
                        batch_number: item.batch_number.clone(),
                        expiration_date: item.expiration_date,
                        divergence_justification: None,
                    },
                )
                .await?;

            // Get the movement we just inserted
            let source_movement_id: Uuid = sqlx::query_scalar(
                r#"SELECT id FROM stock_movements
                   WHERE warehouse_id = $1 AND catalog_item_id = $2
                     AND movement_type = 'TRANSFER_OUT' AND user_id = $3
                   ORDER BY created_at DESC LIMIT 1"#,
            )
            .bind(source_warehouse_id)
            .bind(item.catalog_item_id)
            .bind(initiated_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            // Store transfer item
            sqlx::query(
                r#"INSERT INTO stock_transfer_items (
                    transfer_id, catalog_item_id, quantity_requested,
                    unit_raw_id, unit_conversion_id, conversion_factor,
                    batch_number, expiration_date, notes, source_movement_id
                   ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            )
            .bind(transfer_id)
            .bind(item.catalog_item_id)
            .bind(quantity_base)
            .bind(item.unit_raw_id)
            .bind(item.unit_conversion_id)
            .bind(item.conversion_factor)
            .bind(item.batch_number.as_deref())
            .bind(item.expiration_date)
            .bind(item.notes.as_deref())
            .bind(source_movement_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        }

        // Update document_number in movements with the generated transfer_number
        let transfer_number: String =
            sqlx::query_scalar("SELECT transfer_number FROM stock_transfers WHERE id = $1")
                .bind(transfer_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

        sqlx::query(
            r#"UPDATE stock_movements SET
                document_number = $1,
                related_warehouse_id = $2
               WHERE warehouse_id = $3
                 AND movement_type = 'TRANSFER_OUT'
                 AND user_id = $4
                 AND document_number IS NULL
                 AND created_at >= NOW() - INTERVAL '1 minute'"#,
        )
        .bind(&transfer_number)
        .bind(payload.destination_warehouse_id)
        .bind(source_warehouse_id)
        .bind(initiated_by)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.get_transfer(transfer_id).await
    }

    // ========================================================================
    // CONFIRM (Step 2a)
    // ========================================================================

    /// Confirm receipt at destination warehouse (RF-018 step 2a).
    /// Generates TRANSFER_IN movements for confirmed quantities.
    pub async fn confirm_transfer(
        &self,
        transfer_id: Uuid,
        payload: ConfirmTransferPayload,
        confirmed_by: Uuid,
    ) -> Result<StockTransferWithItemsDto, ServiceError> {
        let transfer = self.get_transfer(transfer_id).await?;

        if transfer.transfer.status != StockTransferStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Transferência não pode ser confirmada. Status atual: {:?}",
                transfer.transfer.status
            )));
        }

        // Check if expired
        if let Some(expires_at) = transfer.transfer.expires_at {
            if Utc::now() > expires_at {
                return Err(ServiceError::BadRequest(
                    "Transferência expirada. Cancele e inicie uma nova.".to_string(),
                ));
            }
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Process TRANSFER_IN for each confirmed item
        for conf in &payload.items {
            if conf.quantity_confirmed <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade confirmada deve ser maior que zero".to_string(),
                ));
            }

            // Find the corresponding transfer item
            let titem = transfer
                .items
                .iter()
                .find(|i| i.id == conf.transfer_item_id)
                .ok_or_else(|| {
                    ServiceError::BadRequest(format!(
                        "Item de transferência {} não encontrado",
                        conf.transfer_item_id
                    ))
                })?;

            if conf.quantity_confirmed > titem.quantity_requested {
                return Err(ServiceError::BadRequest(format!(
                    "Quantidade confirmada ({}) não pode exceder a solicitada ({})",
                    conf.quantity_confirmed, titem.quantity_requested
                )));
            }

            // Get unit_price from the TRANSFER_OUT movement for this item
            let source_price: Decimal = sqlx::query_scalar(
                "SELECT unit_price_base FROM stock_movements WHERE id = $1",
            )
            .bind(titem.source_movement_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?
            .unwrap_or(Decimal::ZERO);

            // Generate TRANSFER_IN movement at destination
            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id: transfer.transfer.destination_warehouse_id,
                        catalog_item_id: titem.catalog_item_id,
                        movement_type: StockMovementType::TransferIn,
                        unit_raw_id: titem.unit_raw_id,
                        unit_conversion_id: None,
                        quantity_raw: conf.quantity_confirmed,
                        conversion_factor: titem.conversion_factor,
                        quantity_base: conf.quantity_confirmed,
                        unit_price_base: source_price, // preserve cost from source
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        document_number: Some(transfer.transfer.transfer_number.clone()),
                        notes: payload.notes.clone(),
                        user_id: confirmed_by,
                        batch_number: titem.batch_number.clone(),
                        expiration_date: titem.expiration_date,
                        divergence_justification: None,
                    },
                )
                .await?;

            // Get destination movement id
            let dest_movement_id: Uuid = sqlx::query_scalar(
                r#"SELECT id FROM stock_movements
                   WHERE warehouse_id = $1 AND catalog_item_id = $2
                     AND movement_type = 'TRANSFER_IN' AND user_id = $3
                   ORDER BY created_at DESC LIMIT 1"#,
            )
            .bind(transfer.transfer.destination_warehouse_id)
            .bind(titem.catalog_item_id)
            .bind(confirmed_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            // Update transfer item with confirmed quantity and movement link
            sqlx::query(
                r#"UPDATE stock_transfer_items SET
                    quantity_confirmed = $2,
                    destination_movement_id = $3
                   WHERE id = $1"#,
            )
            .bind(conf.transfer_item_id)
            .bind(conf.quantity_confirmed)
            .bind(dest_movement_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            // Link the two movements (related_movement_id)
            if let Some(src_mv) = titem.source_movement_id {
                sqlx::query(
                    "UPDATE stock_movements SET related_movement_id = $1 WHERE id = $2",
                )
                .bind(dest_movement_id)
                .bind(src_mv)
                .execute(&mut *tx)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

                sqlx::query(
                    "UPDATE stock_movements SET related_movement_id = $1 WHERE id = $2",
                )
                .bind(src_mv)
                .bind(dest_movement_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            }
        }

        // Update transfer status to CONFIRMED
        sqlx::query(
            r#"UPDATE stock_transfers SET
                status = 'CONFIRMED',
                confirmed_by = $2,
                confirmed_at = NOW(),
                updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(transfer_id)
        .bind(confirmed_by)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.get_transfer(transfer_id).await
    }

    // ========================================================================
    // REJECT (Step 2b)
    // ========================================================================

    /// Reject transfer at destination (RF-018 step 2b).
    /// Generates compensatory TRANSFER_IN at source to restore stock.
    pub async fn reject_transfer(
        &self,
        transfer_id: Uuid,
        payload: RejectTransferPayload,
        rejected_by: Uuid,
    ) -> Result<StockTransferWithItemsDto, ServiceError> {
        if payload.rejection_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa de rejeição é obrigatória".to_string(),
            ));
        }

        let transfer = self.get_transfer(transfer_id).await?;

        if transfer.transfer.status != StockTransferStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Transferência não pode ser rejeitada. Status atual: {:?}",
                transfer.transfer.status
            )));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Restore stock at source via TRANSFER_IN (compensatory)
        for item in &transfer.items {
            let source_price: Decimal = if let Some(src_mv) = item.source_movement_id {
                sqlx::query_scalar("SELECT unit_price_base FROM stock_movements WHERE id = $1")
                    .bind(src_mv)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| ServiceError::Internal(e.to_string()))?
                    .unwrap_or(Decimal::ZERO)
            } else {
                Decimal::ZERO
            };

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id: transfer.transfer.source_warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: StockMovementType::TransferIn,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: None,
                        quantity_raw: item.quantity_requested,
                        conversion_factor: item.conversion_factor,
                        quantity_base: item.quantity_requested,
                        unit_price_base: source_price,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        document_number: Some(format!(
                            "ESTORNO-TRF/{}",
                            transfer.transfer.transfer_number
                        )),
                        notes: Some(format!(
                            "Estorno de transferência rejeitada — {}",
                            payload.rejection_reason
                        )),
                        user_id: rejected_by,
                        batch_number: item.batch_number.clone(),
                        expiration_date: item.expiration_date,
                        divergence_justification: None,
                    },
                )
                .await?;
        }

        // Update transfer status to REJECTED
        sqlx::query(
            r#"UPDATE stock_transfers SET
                status = 'REJECTED',
                rejected_by = $2,
                rejected_at = NOW(),
                rejection_reason = $3,
                updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(transfer_id)
        .bind(rejected_by)
        .bind(&payload.rejection_reason)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.get_transfer(transfer_id).await
    }

    // ========================================================================
    // CANCEL (by source before confirmation)
    // ========================================================================

    /// Cancel a pending transfer initiated by source warehouse.
    pub async fn cancel_transfer(
        &self,
        transfer_id: Uuid,
        payload: CancelTransferPayload,
        cancelled_by: Uuid,
    ) -> Result<StockTransferWithItemsDto, ServiceError> {
        if payload.cancellation_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa de cancelamento é obrigatória".to_string(),
            ));
        }

        let transfer = self.get_transfer(transfer_id).await?;

        if transfer.transfer.status != StockTransferStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Transferência não pode ser cancelada. Status atual: {:?}",
                transfer.transfer.status
            )));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Restore stock at source via compensatory TRANSFER_IN
        for item in &transfer.items {
            let source_price: Decimal = if let Some(src_mv) = item.source_movement_id {
                sqlx::query_scalar("SELECT unit_price_base FROM stock_movements WHERE id = $1")
                    .bind(src_mv)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| ServiceError::Internal(e.to_string()))?
                    .unwrap_or(Decimal::ZERO)
            } else {
                Decimal::ZERO
            };

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id: transfer.transfer.source_warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: StockMovementType::TransferIn,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: None,
                        quantity_raw: item.quantity_requested,
                        conversion_factor: item.conversion_factor,
                        quantity_base: item.quantity_requested,
                        unit_price_base: source_price,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        document_number: Some(format!(
                            "CANCEL-TRF/{}",
                            transfer.transfer.transfer_number
                        )),
                        notes: Some(format!(
                            "Estorno de transferência cancelada — {}",
                            payload.cancellation_reason
                        )),
                        user_id: cancelled_by,
                        batch_number: item.batch_number.clone(),
                        expiration_date: item.expiration_date,
                        divergence_justification: None,
                    },
                )
                .await?;
        }

        // Update transfer status to CANCELLED
        sqlx::query(
            r#"UPDATE stock_transfers SET
                status = 'CANCELLED',
                cancelled_by = $2,
                cancelled_at = NOW(),
                cancellation_reason = $3,
                updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(transfer_id)
        .bind(cancelled_by)
        .bind(&payload.cancellation_reason)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.get_transfer(transfer_id).await
    }
}
