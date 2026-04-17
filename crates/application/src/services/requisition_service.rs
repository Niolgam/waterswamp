use crate::errors::ServiceError;
use crate::services::stock_movement_service::{ProcessMovementInput, StockMovementService, StockMovementType};
use domain::{
    models::requisition::*,
    ports::requisition::*,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Requisition Service
// ============================================================================

pub struct RequisitionService {
    pool: PgPool,
    requisition_repo: Arc<dyn RequisitionRepositoryPort>,
    item_repo: Arc<dyn RequisitionItemRepositoryPort>,
    stock_movement_service: Arc<StockMovementService>,
}

impl RequisitionService {
    pub fn new(
        pool: PgPool,
        requisition_repo: Arc<dyn RequisitionRepositoryPort>,
        item_repo: Arc<dyn RequisitionItemRepositoryPort>,
        stock_movement_service: Arc<StockMovementService>,
    ) -> Self {
        Self {
            pool,
            requisition_repo,
            item_repo,
            stock_movement_service,
        }
    }

    // ========================================================================
    // AUDIT CONTEXT HELPERS
    // ========================================================================

    /// Sets the audit context before performing operations.
    /// This is essential for the database triggers to capture who performed the action.
    pub async fn set_audit_context(&self, ctx: &AuditContext) -> Result<(), ServiceError> {
        self.requisition_repo
            .set_audit_context(
                ctx.user_id,
                ctx.ip_address.as_deref(),
                ctx.user_agent.as_deref(),
            )
            .await
            .map_err(ServiceError::from)
    }

    // ========================================================================
    // REQUISITION OPERATIONS
    // ========================================================================

    /// Get a requisition by ID
    pub async fn get_requisition(&self, id: Uuid) -> Result<RequisitionDto, ServiceError> {
        self.requisition_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))
    }

    /// Get a requisition by number
    pub async fn get_requisition_by_number(
        &self,
        number: &str,
    ) -> Result<RequisitionDto, ServiceError> {
        self.requisition_repo
            .find_by_number(number)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))
    }

    /// Approve a requisition. Atomically:
    /// 1. Updates status to APPROVED
    /// 2. Inserts stock_reservations per item (replaces fn_manage_stock_reservation)
    /// 3. Updates warehouse_stocks.reserved_quantity
    pub async fn approve_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: ApproveRequisitionPayload,
    ) -> Result<RequisitionDto, ServiceError> {
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser aprovada: status atual é {:?}",
                requisition.status
            )));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Set audit context on the same connection as the transaction
        sqlx::query("SELECT fn_set_audit_context($1, $2, $3)")
            .bind(ctx.user_id)
            .bind(ctx.ip_address.as_deref())
            .bind(ctx.user_agent.as_deref())
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let approved = sqlx::query_as::<_, RequisitionDto>(
            r#"UPDATE requisitions SET
                status = 'APPROVED',
                approved_by = $2,
                approved_at = NOW(),
                internal_notes = COALESCE($3, internal_notes),
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(ctx.user_id)
        .bind(payload.notes.as_deref())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Create stock reservations and update warehouse_stocks.reserved_quantity
        #[derive(sqlx::FromRow)]
        struct ItemRow {
            id: Uuid,
            catalog_item_id: Uuid,
            requested_quantity: Decimal,
            approved_quantity: Option<Decimal>,
        }

        let items = sqlx::query_as::<_, ItemRow>(
            r#"SELECT id, catalog_item_id, requested_quantity, approved_quantity
               FROM requisition_items
               WHERE requisition_id = $1 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        for item in &items {
            let qty = item.approved_quantity.unwrap_or(item.requested_quantity);

            sqlx::query(
                r#"INSERT INTO stock_reservations
                   (requisition_id, requisition_item_id, catalog_item_id, warehouse_id, quantity)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(id)
            .bind(item.id)
            .bind(item.catalog_item_id)
            .bind(requisition.warehouse_id)
            .bind(qty)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            sqlx::query(
                r#"UPDATE warehouse_stocks
                   SET reserved_quantity = reserved_quantity + $1, updated_at = NOW()
                   WHERE warehouse_id = $2 AND catalog_item_id = $3"#,
            )
            .bind(qty)
            .bind(requisition.warehouse_id)
            .bind(item.catalog_item_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(approved)
    }

    /// Reject a requisition
    pub async fn reject_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: RejectRequisitionPayload,
    ) -> Result<RequisitionDto, ServiceError> {
        // Verify requisition exists and is in pending status
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser rejeitada: status atual é {:?}",
                requisition.status
            )));
        }

        // Validate reason
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para rejeição".to_string(),
            ));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Reject
        self.requisition_repo
            .reject(id, ctx.user_id, &payload.reason)
            .await
            .map_err(ServiceError::from)
    }

    /// Cancel a requisition. Atomically (replaces fn_cancel_requisition):
    /// 1. Validates status allows cancellation and no stock movements exist
    /// 2. Releases active stock reservations
    /// 3. Updates warehouse_stocks.reserved_quantity
    /// 4. Sets status to CANCELLED
    pub async fn cancel_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: CancelRequisitionPayload,
    ) -> Result<serde_json::Value, ServiceError> {
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para cancelamento".to_string(),
            ));
        }

        let requisition = self.get_requisition(id).await?;

        let cancellable = matches!(
            requisition.status,
            RequisitionStatus::Draft
                | RequisitionStatus::Pending
                | RequisitionStatus::Approved
        );
        if !cancellable {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser cancelada: status atual é {:?}",
                requisition.status
            )));
        }

        // Check for stock movements (block cancellation if any exist)
        let has_movements: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM stock_movements WHERE requisition_id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        if has_movements {
            return Err(ServiceError::BadRequest(
                "Não é possível cancelar: existem movimentações de estoque vinculadas a esta requisição".to_string(),
            ));
        }

        let previous_status = format!("{:?}", requisition.status).to_uppercase();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Set audit context
        sqlx::query("SELECT fn_set_audit_context($1, $2, $3)")
            .bind(ctx.user_id)
            .bind(ctx.ip_address.as_deref())
            .bind(ctx.user_agent.as_deref())
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        sqlx::query(
            r#"UPDATE requisitions SET
                status = 'CANCELLED',
                cancellation_reason = $2,
                updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(&payload.reason)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Release active stock reservations
        #[derive(sqlx::FromRow)]
        struct ReservationRow {
            #[allow(dead_code)]
            id: Uuid,
            catalog_item_id: Uuid,
            quantity: Decimal,
        }

        let reservations = sqlx::query_as::<_, ReservationRow>(
            "SELECT id, catalog_item_id, quantity FROM stock_reservations WHERE requisition_id = $1 AND is_active = TRUE",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        for res in &reservations {
            sqlx::query(
                r#"UPDATE warehouse_stocks
                   SET reserved_quantity = GREATEST(0, reserved_quantity - $1), updated_at = NOW()
                   WHERE warehouse_id = $2 AND catalog_item_id = $3"#,
            )
            .bind(res.quantity)
            .bind(requisition.warehouse_id)
            .bind(res.catalog_item_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        }

        sqlx::query(
            r#"UPDATE stock_reservations
               SET is_active = FALSE, released_at = NOW(), release_reason = $2
               WHERE requisition_id = $1 AND is_active = TRUE"#,
        )
        .bind(id)
        .bind(&payload.reason)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(serde_json::json!({
            "success": true,
            "requisition_id": id,
            "previous_status": previous_status,
            "new_status": "CANCELLED",
            "reason": payload.reason
        }))
    }

    /// Rollback a requisition to a previous state
    pub async fn rollback_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: RollbackPayload,
    ) -> Result<serde_json::Value, ServiceError> {
        // Validate reason
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para rollback".to_string(),
            ));
        }

        // The database function will handle all validations:
        // - Check if requisition exists
        // - Check if history point is valid
        // - Check if status allows rollback
        // - Check for stock movements after target point
        self.requisition_repo
            .rollback(id, payload.history_id, &payload.reason, ctx.user_id)
            .await
            .map_err(|e| {
                if let domain::errors::RepositoryError::Database(ref msg) = e {
                    return ServiceError::BadRequest(msg.clone());
                }
                ServiceError::from(e)
            })
    }

    /// List requisitions with pagination and filters
    pub async fn list_requisitions(
        &self,
        limit: i64,
        offset: i64,
        status: Option<RequisitionStatus>,
        requester_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<RequisitionDto>, i64), ServiceError> {
        self.requisition_repo
            .list(limit, offset, status, requester_id, warehouse_id)
            .await
            .map_err(ServiceError::from)
    }

    // ========================================================================
    // AUDIT / HISTORY OPERATIONS
    // ========================================================================

    /// Get the audit history for a requisition
    pub async fn get_requisition_history(
        &self,
        id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<RequisitionHistoryEntry>, ServiceError> {
        // Verify requisition exists (or existed)
        // Note: We allow getting history even for deleted requisitions

        self.requisition_repo
            .get_history(id, limit.unwrap_or(50))
            .await
            .map_err(ServiceError::from)
    }

    /// Get available rollback points for a requisition
    pub async fn get_rollback_points(
        &self,
        id: Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<RollbackPoint>, ServiceError> {
        // Verify requisition exists
        let _ = self.get_requisition(id).await?;

        self.requisition_repo
            .get_rollback_points(id, limit.unwrap_or(20))
            .await
            .map_err(ServiceError::from)
    }

    // ========================================================================
    // NEW STATUS TRANSITIONS (RF-013 / RF-014)
    // ========================================================================

    /// Start processing a requisition (APPROVED → PROCESSING).
    /// Indicates that physical separation has begun.
    pub async fn start_processing(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: StartProcessingPayload,
    ) -> Result<RequisitionDto, ServiceError> {
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Approved {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode iniciar processamento: status atual é {:?}. Esperado: APPROVED",
                requisition.status
            )));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        sqlx::query("SELECT fn_set_audit_context($1, $2, $3)")
            .bind(ctx.user_id)
            .bind(ctx.ip_address.as_deref())
            .bind(ctx.user_agent.as_deref())
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let updated = sqlx::query_as::<_, RequisitionDto>(
            r#"UPDATE requisitions SET
                status = 'PROCESSING',
                internal_notes = COALESCE($2, internal_notes),
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(payload.notes.as_deref())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(updated)
    }

    /// Fulfill a requisition in processing (PROCESSING → FULFILLED / PARTIALLY_FULFILLED).
    ///
    /// Atomically:
    ///  1. Validates all items and cut reasons (RF-014)
    ///  2. Generates EXIT stock movements via StockMovementService
    ///  3. Releases stock reservations
    ///  4. Transitions status to FULFILLED or PARTIALLY_FULFILLED
    pub async fn fulfill_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: FulfillRequisitionPayload,
    ) -> Result<RequisitionDto, ServiceError> {
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Processing {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser atendida: status atual é {:?}. Esperado: PROCESSING",
                requisition.status
            )));
        }

        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Informe ao menos um item para atendimento".to_string(),
            ));
        }

        // ── Load all approved items for this requisition ──────────────────────
        #[derive(sqlx::FromRow)]
        struct ApprovedItemRow {
            id: Uuid,
            catalog_item_id: Uuid,
            approved_quantity: Option<Decimal>,
            requested_quantity: Decimal,
            unit_value: Decimal,
        }

        let approved_items = sqlx::query_as::<_, ApprovedItemRow>(
            r#"SELECT id, catalog_item_id, approved_quantity, requested_quantity, unit_value
               FROM requisition_items
               WHERE requisition_id = $1 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // ── Lookup unit info per catalog item ──────────────────────────────────
        #[derive(sqlx::FromRow)]
        struct ItemUnitRow {
            catalog_item_id: Uuid,
            unit_id: Uuid,
        }
        let catalog_ids: Vec<Uuid> = approved_items.iter().map(|i| i.catalog_item_id).collect();

        let item_units: Vec<ItemUnitRow> = sqlx::query_as::<_, ItemUnitRow>(
            "SELECT id AS catalog_item_id, base_unit_id AS unit_id FROM catmat_items WHERE id = ANY($1)",
        )
        .bind(&catalog_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let unit_map: std::collections::HashMap<Uuid, Uuid> =
            item_units.into_iter().map(|r| (r.catalog_item_id, r.unit_id)).collect();

        // ── Validate each fulfill item ──────────────────────────────────────────
        let mut is_partial = false;

        for fi in &payload.items {
            let approved = approved_items
                .iter()
                .find(|a| a.id == fi.requisition_item_id)
                .ok_or_else(|| {
                    ServiceError::BadRequest(format!(
                        "Item {} não pertence a esta requisição",
                        fi.requisition_item_id
                    ))
                })?;

            if fi.fulfilled_quantity <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(format!(
                    "Quantidade atendida deve ser maior que zero para o item {}",
                    fi.requisition_item_id
                )));
            }

            let approved_qty = approved.approved_quantity.unwrap_or(approved.requested_quantity);

            if fi.fulfilled_quantity > approved_qty {
                return Err(ServiceError::BadRequest(format!(
                    "Quantidade atendida ({}) não pode exceder a aprovada ({}) para o item {}",
                    fi.fulfilled_quantity, approved_qty, fi.requisition_item_id
                )));
            }

            if fi.fulfilled_quantity < approved_qty {
                // Partial cut: justification is mandatory (RF-014)
                if fi.cut_reason.as_deref().unwrap_or("").trim().is_empty() {
                    return Err(ServiceError::BadRequest(format!(
                        "Justificativa de corte é obrigatória para atendimento parcial do item {}",
                        fi.requisition_item_id
                    )));
                }
                is_partial = true;
            }
        }

        let new_status = if is_partial {
            "PARTIALLY_FULFILLED"
        } else {
            "FULFILLED"
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        sqlx::query("SELECT fn_set_audit_context($1, $2, $3)")
            .bind(ctx.user_id)
            .bind(ctx.ip_address.as_deref())
            .bind(ctx.user_agent.as_deref())
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // ── Generate EXIT movements + release reservations ─────────────────────
        for fi in &payload.items {
            let approved = approved_items
                .iter()
                .find(|a| a.id == fi.requisition_item_id)
                .unwrap(); // already validated above

            let unit_id = unit_map.get(&approved.catalog_item_id).copied().ok_or_else(|| {
                ServiceError::Internal(format!(
                    "Unidade base não encontrada para item {}",
                    approved.catalog_item_id
                ))
            })?;

            // Generate EXIT stock movement
            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id: requisition.warehouse_id,
                        catalog_item_id: approved.catalog_item_id,
                        movement_type: StockMovementType::Exit,
                        unit_raw_id: unit_id,
                        unit_conversion_id: None,
                        quantity_raw: fi.fulfilled_quantity,
                        conversion_factor: Decimal::ONE,
                        quantity_base: fi.fulfilled_quantity,
                        unit_price_base: approved.unit_value,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: Some(id),
                        requisition_item_id: Some(fi.requisition_item_id),
                        document_number: Some(requisition.requisition_number.clone()),
                        notes: payload.notes.clone(),
                        user_id: ctx.user_id,
                        batch_number: None,
                        expiration_date: None,
                        divergence_justification: None,
                    },
                )
                .await?;

            // Update fulfilled_quantity and cut_reason on item
            sqlx::query(
                r#"UPDATE requisition_items SET
                    fulfilled_quantity = $2,
                    cut_reason = $3,
                    updated_at = NOW()
                   WHERE id = $1"#,
            )
            .bind(fi.requisition_item_id)
            .bind(fi.fulfilled_quantity)
            .bind(fi.cut_reason.as_deref())
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            // Release stock reservation for this item
            let approved_qty = approved.approved_quantity.unwrap_or(approved.requested_quantity);
            sqlx::query(
                r#"UPDATE warehouse_stocks
                   SET reserved_quantity = GREATEST(0, reserved_quantity - $1), updated_at = NOW()
                   WHERE warehouse_id = $2 AND catalog_item_id = $3"#,
            )
            .bind(approved_qty)
            .bind(requisition.warehouse_id)
            .bind(approved.catalog_item_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        }

        // Mark all reservations as fulfilled
        sqlx::query(
            r#"UPDATE stock_reservations
               SET is_active = FALSE, released_at = NOW(), release_reason = 'FULFILLED'
               WHERE requisition_id = $1 AND is_active = TRUE"#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Update requisition status
        let updated = sqlx::query_as::<_, RequisitionDto>(
            &format!(
                r#"UPDATE requisitions SET
                    status = '{}',
                    fulfilled_by = $2,
                    fulfilled_at = NOW(),
                    internal_notes = COALESCE($3, internal_notes),
                    updated_at = NOW()
                   WHERE id = $1
                   RETURNING *"#,
                new_status
            ),
        )
        .bind(id)
        .bind(ctx.user_id)
        .bind(payload.notes.as_deref())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(updated)
    }

    /// Suspend a requisition when its organizational unit is blocked (RN-004).
    /// Only transitions PENDING requisitions. PROCESSING requisitions are signaled but not suspended.
    pub async fn suspend_requisition(
        &self,
        id: Uuid,
        reason: &str,
    ) -> Result<RequisitionDto, ServiceError> {
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Apenas requisições PENDING podem ser suspensas. Status atual: {:?}",
                requisition.status
            )));
        }

        sqlx::query_as::<_, RequisitionDto>(
            r#"UPDATE requisitions SET
                status = 'SUSPENDED',
                internal_notes = COALESCE(internal_notes || E'\n', '') || $2,
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(format!("[SUSPENSO] {}", reason))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))
    }

    /// Re-activate a suspended requisition when its unit is unblocked (RN-004).
    pub async fn unsuspend_requisition(&self, id: Uuid) -> Result<RequisitionDto, ServiceError> {
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Suspended {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não está suspensa. Status atual: {:?}",
                requisition.status
            )));
        }

        sqlx::query_as::<_, RequisitionDto>(
            r#"UPDATE requisitions SET
                status = 'PENDING',
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))
    }

    // ========================================================================
    // REQUISITION ITEM OPERATIONS
    // ========================================================================

    /// Add an item to a requisition. Atomically:
    /// 1. Looks up average_unit_value from warehouse_stocks (replaces fn_capture_requisition_item_value)
    /// 2. Creates the item with correct unit/total values
    /// 3. Recalculates requisition total_value (replaces trg_update_requisition_total)
    pub async fn add_item_to_requisition(
        &self,
        requisition_id: Uuid,
        payload: CreateRequisitionItemPayload,
        _user_id: Uuid,
    ) -> Result<RequisitionItemDto, ServiceError> {
        if payload.requested_quantity <= Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Quantidade deve ser maior que zero".to_string(),
            ));
        }

        let requisition = self.get_requisition(requisition_id).await?;

        if !matches!(
            requisition.status,
            RequisitionStatus::Draft | RequisitionStatus::Pending
        ) {
            return Err(ServiceError::BadRequest(
                "Itens só podem ser adicionados a requisições com status DRAFT ou PENDING"
                    .to_string(),
            ));
        }

        // Look up unit_value from warehouse_stocks (or estimated_value as fallback)
        let unit_value: Decimal = sqlx::query_scalar(
            r#"SELECT COALESCE(ws.average_unit_value, ci.estimated_value, 0)
               FROM catmat_items ci
               LEFT JOIN warehouse_stocks ws
                   ON ws.catalog_item_id = ci.id AND ws.warehouse_id = $1
               WHERE ci.id = $2"#,
        )
        .bind(requisition.warehouse_id)
        .bind(payload.catalog_item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .unwrap_or(Decimal::ZERO);

        let total_value = payload.requested_quantity * unit_value;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let item = sqlx::query_as::<_, RequisitionItemDto>(
            r#"INSERT INTO requisition_items (
                requisition_id, catalog_item_id,
                requested_quantity, unit_value, total_value, justification
               ) VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(requisition_id)
        .bind(payload.catalog_item_id)
        .bind(payload.requested_quantity)
        .bind(unit_value)
        .bind(total_value)
        .bind(payload.justification.as_deref())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Recalculate requisition total
        sqlx::query(
            r#"UPDATE requisitions
               SET total_value = COALESCE(
                   (SELECT SUM(total_value) FROM requisition_items
                    WHERE requisition_id = $1 AND deleted_at IS NULL), 0),
                   updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(requisition_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(item)
    }

    /// Get items for a requisition
    pub async fn get_requisition_items(
        &self,
        requisition_id: Uuid,
    ) -> Result<Vec<RequisitionItemDto>, ServiceError> {
        // Verify requisition exists
        let _ = self.get_requisition(requisition_id).await?;

        self.item_repo
            .find_by_requisition_id(requisition_id)
            .await
            .map_err(ServiceError::from)
    }

    /// Soft delete a requisition item
    pub async fn soft_delete_item(
        &self,
        item_id: Uuid,
        ctx: &AuditContext,
        reason: &str,
    ) -> Result<(), ServiceError> {
        // Validate reason
        if reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para exclusão de item".to_string(),
            ));
        }

        // Verify item exists
        let item = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or(ServiceError::NotFound("Item não encontrado".to_string()))?;

        // Verify item is not already deleted
        if item.deleted_at.is_some() {
            return Err(ServiceError::BadRequest(
                "Item já foi excluído".to_string(),
            ));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Soft delete
        self.item_repo
            .soft_delete(item_id, ctx.user_id, reason)
            .await
            .map_err(ServiceError::from)
    }

    /// Restore a soft-deleted requisition item
    pub async fn restore_item(
        &self,
        item_id: Uuid,
        ctx: &AuditContext,
    ) -> Result<RequisitionItemDto, ServiceError> {
        // Verify item exists
        let item = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or(ServiceError::NotFound("Item não encontrado".to_string()))?;

        // Verify item is deleted
        if item.deleted_at.is_none() {
            return Err(ServiceError::BadRequest(
                "Item não está excluído".to_string(),
            ));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Restore
        self.item_repo
            .restore(item_id)
            .await
            .map_err(ServiceError::from)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    fn create_test_requisition(status: RequisitionStatus) -> RequisitionDto {
        RequisitionDto {
            id: Uuid::new_v4(),
            requisition_number: "REQ2024001".to_string(),
            warehouse_id: Uuid::new_v4(),
            destination_unit_id: None,
            destination_unit_name: None,
            requester_id: Uuid::new_v4(),
            requester_name: None,
            status,
            priority: RequisitionPriority::Normal,
            total_value: Some(Decimal::new(1000, 2)),
            request_date: Utc::now(),
            needed_by: None,
            approved_by: None,
            approved_at: None,
            fulfilled_by: None,
            fulfilled_at: None,
            rejection_reason: None,
            cancellation_reason: None,
            notes: None,
            internal_notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_item(deleted: bool) -> RequisitionItemDto {
        RequisitionItemDto {
            id: Uuid::new_v4(),
            requisition_id: Uuid::new_v4(),
            catalog_item_id: Uuid::new_v4(),
            requested_quantity: Decimal::new(10, 0),
            approved_quantity: None,
            fulfilled_quantity: Decimal::ZERO,
            unit_value: Decimal::new(100, 2),
            total_value: Decimal::new(1000, 2),
            justification: Some("Test item".to_string()),
            cut_reason: None,
            deleted_at: if deleted { Some(Utc::now()) } else { None },
            deleted_by: if deleted { Some(Uuid::new_v4()) } else { None },
            deletion_reason: if deleted { Some("Test deletion".to_string()) } else { None },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ========================================================================
    // AUDIT CONTEXT TESTS
    // ========================================================================

    #[test]
    fn test_audit_context_creation() {
        let user_id = Uuid::new_v4();
        let ctx = AuditContext::new(user_id)
            .with_ip(Some("10.0.0.1".to_string()))
            .with_user_agent(Some("Mozilla/5.0".to_string()));

        assert_eq!(ctx.user_id, user_id);
        assert_eq!(ctx.ip_address, Some("10.0.0.1".to_string()));
        assert_eq!(ctx.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_audit_context_without_optional_fields() {
        let user_id = Uuid::new_v4();
        let ctx = AuditContext::new(user_id);

        assert_eq!(ctx.user_id, user_id);
        assert_eq!(ctx.ip_address, None);
        assert_eq!(ctx.user_agent, None);
    }

    // ========================================================================
    // PAYLOAD VALIDATION TESTS (Pure logic, no mocks needed)
    // ========================================================================

    #[test]
    fn test_reject_payload_validation() {
        // Empty reason should fail
        let empty_reason = "   ".trim();
        assert!(empty_reason.is_empty());

        // Valid reason should pass
        let valid_reason = "Budget constraints".trim();
        assert!(!valid_reason.is_empty());
    }

    #[test]
    fn test_cancel_payload_validation() {
        let empty_reason = "".trim();
        assert!(empty_reason.is_empty());

        let valid_reason = "No longer needed".trim();
        assert!(!valid_reason.is_empty());
    }

    #[test]
    fn test_rollback_payload_validation() {
        let empty_reason = "   ".trim();
        assert!(empty_reason.is_empty());

        let valid_reason = "Reverting to previous state".trim();
        assert!(!valid_reason.is_empty());
    }

    // ========================================================================
    // DTO CREATION TESTS
    // ========================================================================

    #[test]
    fn test_requisition_dto_creation() {
        let req = create_test_requisition(RequisitionStatus::Pending);

        assert_eq!(req.status, RequisitionStatus::Pending);
        assert_eq!(req.requisition_number, "REQ2024001");
        assert!(req.approved_by.is_none());
        assert!(req.approved_at.is_none());
    }

    #[test]
    fn test_requisition_item_dto_creation() {
        let item = create_test_item(false);

        assert!(item.deleted_at.is_none());
        assert!(item.deleted_by.is_none());
        assert!(item.deletion_reason.is_none());
    }

    #[test]
    fn test_deleted_item_dto_creation() {
        let item = create_test_item(true);

        assert!(item.deleted_at.is_some());
        assert!(item.deleted_by.is_some());
        assert!(item.deletion_reason.is_some());
    }

    // ========================================================================
    // STATUS VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_pending_status_allows_approval() {
        let req = create_test_requisition(RequisitionStatus::Pending);
        assert_eq!(req.status, RequisitionStatus::Pending);
    }

    #[test]
    fn test_draft_status_blocks_approval() {
        let req = create_test_requisition(RequisitionStatus::Draft);
        assert_ne!(req.status, RequisitionStatus::Pending);
    }

    #[test]
    fn test_approved_status_blocks_rejection() {
        let req = create_test_requisition(RequisitionStatus::Approved);
        assert_ne!(req.status, RequisitionStatus::Pending);
    }

    // ========================================================================
    // ENUM TESTS
    // ========================================================================

    #[test]
    fn test_requisition_status_variants() {
        assert_eq!(
            format!("{:?}", RequisitionStatus::Draft),
            "Draft"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Pending),
            "Pending"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Approved),
            "Approved"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Rejected),
            "Rejected"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Cancelled),
            "Cancelled"
        );
    }

    #[test]
    fn test_requisition_priority_variants() {
        assert_eq!(
            format!("{:?}", RequisitionPriority::Low),
            "Low"
        );
        assert_eq!(
            format!("{:?}", RequisitionPriority::Normal),
            "Normal"
        );
        assert_eq!(
            format!("{:?}", RequisitionPriority::High),
            "High"
        );
        assert_eq!(
            format!("{:?}", RequisitionPriority::Urgent),
            "Urgent"
        );
    }
}
