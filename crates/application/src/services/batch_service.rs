use crate::errors::ServiceError;
use crate::services::stock_movement_service::{ProcessMovementInput, StockMovementType};
use domain::{
    models::batch::*,
    ports::batch::{BatchQualityOccurrenceRepositoryPort, WarehouseBatchStockRepositoryPort},
    ports::warehouse::WarehouseRepositoryPort,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct BatchService {
    pool: PgPool,
    batch_stock_repo: Arc<dyn WarehouseBatchStockRepositoryPort>,
    batch_quality_repo: Arc<dyn BatchQualityOccurrenceRepositoryPort>,
    warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
}

impl BatchService {
    pub fn new(
        pool: PgPool,
        batch_stock_repo: Arc<dyn WarehouseBatchStockRepositoryPort>,
        batch_quality_repo: Arc<dyn BatchQualityOccurrenceRepositoryPort>,
        warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
    ) -> Self {
        Self { pool, batch_stock_repo, batch_quality_repo, warehouse_repo }
    }

    // ========================================================================
    // Ticket 3.1 — Motor FEFO (RF-021)
    // ========================================================================

    /// Executes a FEFO-driven exit from warehouse stock.
    ///
    /// If `payload.batch_number` is Some, exits only that batch.
    /// Otherwise iterates batches in FEFO order (earliest expiration first) until
    /// the requested quantity is satisfied. Creates one EXIT movement per batch consumed.
    pub async fn fefo_exit(
        &self,
        warehouse_id: Uuid,
        payload: FefoExitPayload,
        user_id: Uuid,
    ) -> Result<FefoExitResult, ServiceError> {
        // Validate warehouse exists
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Almoxarifado '{}' não encontrado", warehouse_id))
            })?;

        if payload.quantity_raw <= Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Quantidade de saída deve ser maior que zero".to_string(),
            ));
        }

        let quantity_base = payload.quantity_raw * payload.conversion_factor;

        // Check FEFO enabled setting
        let fefo_enabled: bool = sqlx::query_scalar(
            "SELECT (value::text)::boolean FROM system_settings WHERE key = 'fefo.enabled'",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .flatten()
        .unwrap_or(true);

        // Check if expired exits are allowed
        let allow_expired: bool = sqlx::query_scalar(
            "SELECT (value::text)::boolean FROM system_settings WHERE key = 'fefo.allow_expired_exit'",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .flatten()
        .unwrap_or(false);

        let batches: Vec<WarehouseBatchStockDto> = if let Some(ref bn) = payload.batch_number {
            // Explicit batch requested
            let b = self
                .batch_stock_repo
                .find(warehouse_id, payload.catalog_item_id, bn)
                .await
                .map_err(ServiceError::from)?
                .ok_or_else(|| {
                    ServiceError::NotFound(format!(
                        "Lote '{}' não encontrado no almoxarifado",
                        bn
                    ))
                })?;
            vec![b]
        } else if fefo_enabled {
            // FEFO auto-select
            self.batch_stock_repo
                .list_fefo(warehouse_id, payload.catalog_item_id)
                .await
                .map_err(ServiceError::from)?
        } else {
            vec![]
        };

        if batches.is_empty() {
            // No batch tracking — create a single EXIT without batch info
            let input = ProcessMovementInput {
                warehouse_id,
                catalog_item_id: payload.catalog_item_id,
                movement_type: StockMovementType::Exit,
                unit_raw_id: payload.unit_raw_id,
                unit_conversion_id: payload.unit_conversion_id,
                quantity_raw: payload.quantity_raw,
                conversion_factor: payload.conversion_factor,
                quantity_base,
                unit_price_base: Decimal::ZERO,
                invoice_id: None,
                invoice_item_id: None,
                requisition_id: payload.requisition_id,
                requisition_item_id: payload.requisition_item_id,
                related_warehouse_id: None,
                document_number: Some(payload.document_number.clone()),
                notes: payload.notes.clone(),
                user_id,
                batch_number: None,
                expiration_date: None,
                divergence_justification: None,
            };
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

            // We need StockMovementService here — but to avoid circular deps we call
            // the raw SQL inline (same logic as process_movement but simplified for EXIT)
            self.execute_exit_movement(&mut tx, input).await?;
            tx.commit()
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

            return Ok(FefoExitResult {
                warehouse_id,
                catalog_item_id: payload.catalog_item_id,
                total_quantity_exited: quantity_base,
                batches_consumed: vec![],
            });
        }

        // FEFO: iterate batches, consuming from earliest-expiring first
        let mut remaining = quantity_base;
        let mut consumptions: Vec<BatchConsumptionDetail> = Vec::new();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        for batch in &batches {
            if remaining <= Decimal::ZERO {
                break;
            }
            if batch.is_quarantined {
                continue; // skip quarantined batches
            }
            if !allow_expired {
                if let Some(exp) = batch.expiration_date {
                    if exp < chrono::Local::now().date_naive() {
                        return Err(ServiceError::BadRequest(format!(
                            "Lote '{}' está vencido ({}) e saídas de lotes vencidos estão desabilitadas (fefo.allow_expired_exit=false).",
                            batch.batch_number, exp
                        )));
                    }
                }
            }

            let available = batch.quantity;
            if available <= Decimal::ZERO {
                continue;
            }

            let to_consume = remaining.min(available);

            // Proportional raw quantity
            let raw_for_batch = if quantity_base > Decimal::ZERO {
                payload.quantity_raw * (to_consume / quantity_base)
            } else {
                to_consume
            };

            let input = ProcessMovementInput {
                warehouse_id,
                catalog_item_id: payload.catalog_item_id,
                movement_type: StockMovementType::Exit,
                unit_raw_id: payload.unit_raw_id,
                unit_conversion_id: payload.unit_conversion_id,
                quantity_raw: raw_for_batch,
                conversion_factor: payload.conversion_factor,
                quantity_base: to_consume,
                unit_price_base: batch.unit_cost,
                invoice_id: None,
                invoice_item_id: None,
                requisition_id: payload.requisition_id,
                requisition_item_id: payload.requisition_item_id,
                related_warehouse_id: None,
                document_number: Some(payload.document_number.clone()),
                notes: payload.notes.clone(),
                user_id,
                batch_number: Some(batch.batch_number.clone()),
                expiration_date: batch.expiration_date,
                divergence_justification: None,
            };

            self.execute_exit_movement(&mut tx, input).await?;

            // Update batch stock (negative delta = exit)
            sqlx::query(
                "UPDATE warehouse_batch_stocks SET quantity = quantity - $3, updated_at = NOW()
                 WHERE warehouse_id = $1 AND catalog_item_id = $2 AND batch_number = $4",
            )
            .bind(warehouse_id)
            .bind(payload.catalog_item_id)
            .bind(to_consume)
            .bind(&batch.batch_number)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            consumptions.push(BatchConsumptionDetail {
                batch_number: batch.batch_number.clone(),
                expiration_date: batch.expiration_date,
                quantity_consumed: to_consume,
                movement_id: None,
            });

            remaining -= to_consume;
        }

        if remaining > Decimal::ZERO {
            return Err(ServiceError::BadRequest(format!(
                "Saldo insuficiente nos lotes disponíveis. Faltam {} unidades para completar a saída FEFO.",
                remaining
            )));
        }

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(FefoExitResult {
            warehouse_id,
            catalog_item_id: payload.catalog_item_id,
            total_quantity_exited: quantity_base,
            batches_consumed: consumptions,
        })
    }

    /// Inline EXIT movement without pulling in StockMovementService (avoids circular dep).
    /// Validates available balance and updates warehouse_stocks.
    async fn execute_exit_movement(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        input: ProcessMovementInput,
    ) -> Result<(), ServiceError> {
        // Fetch current balance with pessimistic lock
        let stock: Option<(Decimal, Decimal)> = sqlx::query_as(
            "SELECT quantity, average_unit_value FROM warehouse_stocks
             WHERE warehouse_id = $1 AND catalog_item_id = $2 FOR UPDATE",
        )
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let (curr_qty, curr_avg) = stock.unwrap_or((Decimal::ZERO, Decimal::ZERO));

        if input.quantity_base > curr_qty {
            return Err(ServiceError::BadRequest(format!(
                "Saldo insuficiente. Disponível: {}, Solicitado: {}",
                curr_qty, input.quantity_base
            )));
        }

        let new_qty = curr_qty - input.quantity_base;
        let exit_price = if input.unit_price_base > Decimal::ZERO {
            input.unit_price_base
        } else {
            curr_avg
        };
        let total_value = input.quantity_base * exit_price;

        sqlx::query(
            r#"INSERT INTO stock_movements (
                warehouse_id, catalog_item_id, movement_type,
                unit_raw_id, unit_conversion_id,
                quantity_raw, conversion_factor, quantity_base,
                unit_price_base, total_value,
                balance_before, balance_after, average_before, average_after,
                requisition_id, requisition_item_id, document_number, notes,
                user_id, batch_number, expiration_date
            ) VALUES (
                $1,$2,$3::stock_movement_type_enum,$4,$5,
                $6,$7,$8,$9,$10,
                $11,$12,$13,$13,
                $14,$15,$16,$17,$18,$19,$20
            )"#,
        )
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .bind(input.movement_type.as_str())
        .bind(input.unit_raw_id)
        .bind(input.unit_conversion_id)
        .bind(input.quantity_raw)
        .bind(input.conversion_factor)
        .bind(input.quantity_base)
        .bind(exit_price)
        .bind(total_value)
        .bind(curr_qty)
        .bind(new_qty)
        .bind(curr_avg)
        .bind(input.requisition_id)
        .bind(input.requisition_item_id)
        .bind(input.document_number.as_deref())
        .bind(input.notes.as_deref())
        .bind(input.user_id)
        .bind(input.batch_number.as_deref())
        .bind(input.expiration_date)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        sqlx::query(
            r#"UPDATE warehouse_stocks SET
                quantity = $3, last_exit_at = NOW(), updated_at = NOW()
               WHERE warehouse_id = $1 AND catalog_item_id = $2"#,
        )
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .bind(new_qty)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(())
    }

    /// List batches in FEFO order for an item in a warehouse.
    pub async fn list_batches(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
    ) -> Result<Vec<WarehouseBatchStockDto>, ServiceError> {
        self.batch_stock_repo
            .list_fefo(warehouse_id, catalog_item_id)
            .await
            .map_err(ServiceError::from)
    }

    /// List batches near expiry across all (or one) warehouse.
    pub async fn list_near_expiry(
        &self,
        warehouse_id: Option<Uuid>,
        days_ahead: Option<i32>,
    ) -> Result<Vec<WarehouseBatchStockDto>, ServiceError> {
        let days = days_ahead.unwrap_or_else(|| self.default_expiry_alert_days());
        self.batch_stock_repo
            .list_near_expiry(warehouse_id, days)
            .await
            .map_err(ServiceError::from)
    }

    fn default_expiry_alert_days(&self) -> i32 {
        30
    }

    // ========================================================================
    // Ticket 3.2 — Ocorrência de Qualidade de Lotes (RF-043)
    // ========================================================================

    /// Creates a batch quality occurrence.
    /// For HIGH/CRITICAL severity: automatically quarantines the batch.
    pub async fn create_occurrence(
        &self,
        payload: CreateBatchQualityOccurrencePayload,
        reported_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, ServiceError> {
        if payload.description.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Descrição da ocorrência é obrigatória".to_string(),
            ));
        }
        if payload.batch_number.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Número do lote é obrigatório".to_string(),
            ));
        }

        let quarantine = payload.severity.triggers_quarantine();

        let occurrence = self
            .batch_quality_repo
            .create(&payload, reported_by, quarantine)
            .await
            .map_err(ServiceError::from)?;

        // Auto-quarantine batch if severity demands it
        if quarantine {
            let reason = format!(
                "Quarentena automática — ocorrência {:?} #{}: {}",
                occurrence.severity, occurrence.id, occurrence.description
            );
            let _ = self
                .batch_stock_repo
                .set_quarantine(
                    payload.warehouse_id,
                    payload.catalog_item_id,
                    &payload.batch_number,
                    true,
                    Some(&reason),
                    Some(reported_by),
                )
                .await;
            tracing::warn!(
                batch = %payload.batch_number,
                warehouse_id = %payload.warehouse_id,
                severity = ?occurrence.severity,
                "Batch quarantined due to quality occurrence"
            );
        }

        Ok(occurrence)
    }

    /// Resolves a quality occurrence and optionally releases quarantine.
    pub async fn resolve_occurrence(
        &self,
        id: Uuid,
        payload: ResolveOccurrencePayload,
        resolved_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, ServiceError> {
        let occ = self
            .batch_quality_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Ocorrência não encontrada".to_string()))?;

        if occ.status != BatchOccurrenceStatus::Open {
            return Err(ServiceError::BadRequest(format!(
                "Somente ocorrências com status OPEN podem ser resolvidas (status atual: {:?})",
                occ.status
            )));
        }
        if payload.corrective_action.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Ação corretiva é obrigatória para resolver uma ocorrência".to_string(),
            ));
        }

        let release = payload.release_quarantine.unwrap_or(false);
        let resolved = self
            .batch_quality_repo
            .resolve(id, &payload, resolved_by)
            .await
            .map_err(ServiceError::from)?;

        if release && occ.quarantine_triggered {
            let _ = self
                .batch_stock_repo
                .set_quarantine(
                    occ.warehouse_id,
                    occ.catalog_item_id,
                    &occ.batch_number,
                    false,
                    None,
                    None,
                )
                .await;
        }

        Ok(resolved)
    }

    /// Closes an occurrence (without necessarily resolving; e.g., false alarm).
    pub async fn close_occurrence(
        &self,
        id: Uuid,
        payload: CloseOccurrencePayload,
        closed_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, ServiceError> {
        let occ = self
            .batch_quality_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Ocorrência não encontrada".to_string()))?;

        if occ.status == BatchOccurrenceStatus::Closed {
            return Err(ServiceError::BadRequest(
                "Ocorrência já está fechada".to_string(),
            ));
        }

        self.batch_quality_repo
            .close(id, &payload, closed_by)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_occurrence(&self, id: Uuid) -> Result<BatchQualityOccurrenceDto, ServiceError> {
        self.batch_quality_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Ocorrência não encontrada".to_string()))
    }

    pub async fn list_occurrences(
        &self,
        warehouse_id: Option<Uuid>,
        catalog_item_id: Option<Uuid>,
        batch_number: Option<String>,
        status: Option<BatchOccurrenceStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<BatchQualityOccurrenceDto>, i64), ServiceError> {
        self.batch_quality_repo
            .list(
                warehouse_id,
                catalog_item_id,
                batch_number.as_deref(),
                status,
                limit,
                offset,
            )
            .await
            .map_err(ServiceError::from)
    }
}
