use crate::errors::ServiceError;
use crate::services::stock_movement_service::{
    ProcessMovementInput, StockMovementService, StockMovementType,
};
use domain::{models::warehouse::*, ports::warehouse::*};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// Format: NNNNN.NNNNNN/YYYY-NN (e.g., 23108.012345/2026-07)
const SEI_REGEX: &str = r"^\d{5}\.\d{6}/\d{4}-\d{2}$";

pub struct InventoryService {
    pool: PgPool,
    session_repo: Arc<dyn InventorySessionRepositoryPort>,
    warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
    stock_movement_service: Arc<StockMovementService>,
}

impl InventoryService {
    pub fn new(
        pool: PgPool,
        session_repo: Arc<dyn InventorySessionRepositoryPort>,
        warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
        stock_movement_service: Arc<StockMovementService>,
    ) -> Self {
        Self {
            pool,
            session_repo,
            warehouse_repo,
            stock_movement_service,
        }
    }

    // ============================
    // Session CRUD
    // ============================

    /// RF-019: Cria sessão de inventário em status OPEN.
    /// Se tolerance_percentage não informado, usa valor de system_settings.
    pub async fn create_session(
        &self,
        warehouse_id: Uuid,
        payload: CreateInventorySessionPayload,
        created_by: Uuid,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        // Check no active session exists for this warehouse
        let active_exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                SELECT 1 FROM inventory_sessions
                WHERE warehouse_id = $1
                  AND status NOT IN ('COMPLETED', 'CANCELLED')
            )"#,
        )
        .bind(warehouse_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        if active_exists {
            return Err(ServiceError::Conflict(
                "Já existe uma sessão de inventário ativa para este almoxarifado".to_string(),
            ));
        }

        let tolerance = match payload.tolerance_percentage {
            Some(t) => {
                if t < Decimal::ZERO || t > Decimal::ONE {
                    return Err(ServiceError::BadRequest(
                        "tolerance_percentage deve estar entre 0 e 1 (ex: 0.02 = 2%)"
                            .to_string(),
                    ));
                }
                t
            }
            None => {
                // Read from system_settings
                let val: Option<String> = sqlx::query_scalar(
                    "SELECT value FROM system_settings WHERE key = 'inventory.tolerance_percentage'",
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

                val.and_then(|v| v.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::new(2, 2)) // default 0.02
            }
        };

        let session = self
            .session_repo
            .create(warehouse_id, tolerance, payload.notes.as_deref(), created_by)
            .await
            .map_err(ServiceError::from)?;

        self.session_repo
            .find_with_items(session.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar sessão criada".to_string(),
            ))
    }

    pub async fn get_session(
        &self,
        session_id: Uuid,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        self.session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))
    }

    pub async fn list_sessions(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<InventorySessionStatus>,
    ) -> Result<(Vec<InventorySessionDto>, i64), ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        self.session_repo
            .list_by_warehouse(warehouse_id, limit, offset, status)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // State Machine
    // ============================

    /// OPEN → COUNTING: snapshot de estoque atual e inicia contagem.
    pub async fn start_counting(
        &self,
        session_id: Uuid,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))?;

        if session.status != InventorySessionStatus::Open {
            return Err(ServiceError::BadRequest(format!(
                "Sessão não pode iniciar contagem. Status atual: {:?}",
                session.status
            )));
        }

        self.session_repo
            .transition_to_counting(session_id)
            .await
            .map_err(ServiceError::from)?;

        self.session_repo
            .snapshot_stock_items(session_id, session.warehouse_id)
            .await
            .map_err(ServiceError::from)?;

        self.session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar sessão atualizada".to_string(),
            ))
    }

    /// Registra contagem física de um item (status deve ser COUNTING).
    pub async fn submit_count(
        &self,
        session_id: Uuid,
        payload: SubmitCountPayload,
    ) -> Result<InventorySessionItemDto, ServiceError> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))?;

        if session.status != InventorySessionStatus::Counting {
            return Err(ServiceError::BadRequest(format!(
                "Contagem só é permitida no status COUNTING. Status atual: {:?}",
                session.status
            )));
        }

        if payload.counted_quantity < Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Quantidade contada não pode ser negativa".to_string(),
            ));
        }

        self.session_repo
            .upsert_item_count(
                session_id,
                payload.catalog_item_id,
                payload.counted_quantity,
                payload.notes.as_deref(),
            )
            .await
            .map_err(ServiceError::from)
    }

    /// COUNTING → RECONCILING: valida que todos os itens foram contados.
    pub async fn start_reconciliation(
        &self,
        session_id: Uuid,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        let with_items = self
            .session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))?;

        if with_items.session.status != InventorySessionStatus::Counting {
            return Err(ServiceError::BadRequest(format!(
                "Sessão não está em COUNTING. Status atual: {:?}",
                with_items.session.status
            )));
        }

        let uncounted: Vec<_> = with_items
            .items
            .iter()
            .filter(|i| i.counted_quantity.is_none())
            .collect();

        if !uncounted.is_empty() {
            return Err(ServiceError::BadRequest(format!(
                "{} item(s) ainda não foram contados. Registre a contagem de todos os itens antes de reconciliar.",
                uncounted.len()
            )));
        }

        self.session_repo
            .transition_to_reconciling(session_id)
            .await
            .map_err(ServiceError::from)?;

        self.session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar sessão atualizada".to_string(),
            ))
    }

    /// RECONCILING → COMPLETED: processa ajustes de estoque e conclui.
    /// RN-012: se algum item tiver divergência > tolerance_percentage, sei_process_number é obrigatório.
    pub async fn reconcile(
        &self,
        session_id: Uuid,
        payload: ReconcileInventoryPayload,
        user_id: Uuid,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        let with_items = self
            .session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))?;

        if with_items.session.status != InventorySessionStatus::Reconciling {
            return Err(ServiceError::BadRequest(format!(
                "Sessão não está em RECONCILING. Status atual: {:?}",
                with_items.session.status
            )));
        }

        let tolerance = with_items.session.tolerance_percentage;

        // Check RN-012: SEI required if any item exceeds tolerance
        let has_divergence_above_tolerance = with_items.items.iter().any(|item| {
            item.divergence_percentage
                .map(|dp| dp > tolerance)
                .unwrap_or(false)
        });

        if has_divergence_above_tolerance {
            if payload
                .sei_process_number
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(ServiceError::BadRequest(
                    "Número de processo SEI é obrigatório quando há divergências acima da tolerância (RN-012)".to_string(),
                ));
            }

            let sei = payload.sei_process_number.as_ref().unwrap();
            let sei_regex = regex::Regex::new(SEI_REGEX).unwrap();
            if !sei_regex.is_match(sei) {
                return Err(ServiceError::BadRequest(format!(
                    "Número de processo SEI inválido. Formato esperado: NNNNN.NNNNNN/YYYY-NN. Recebido: '{}'",
                    sei
                )));
            }
        }

        let warehouse_id = with_items.session.warehouse_id;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        for item in &with_items.items {
            let system_qty = item.system_quantity;
            let counted_qty = match item.counted_quantity {
                Some(q) => q,
                None => continue, // should not happen after start_reconciliation validates
            };

            let diff = counted_qty - system_qty;
            if diff == Decimal::ZERO {
                continue;
            }

            let (movement_type, quantity_base) = if diff > Decimal::ZERO {
                (StockMovementType::AdjustmentAdd, diff)
            } else {
                (StockMovementType::AdjustmentSub, -diff)
            };

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: None,
                        quantity_raw: quantity_base,
                        conversion_factor: Decimal::ONE,
                        quantity_base,
                        unit_price_base: Decimal::ZERO,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        related_warehouse_id: None,
                        document_number: payload.sei_process_number.clone(),
                        notes: Some(format!(
                            "Ajuste de inventário — sistema: {} / contado: {}",
                            system_qty, counted_qty
                        )),
                        user_id,
                        batch_number: None,
                        expiration_date: None,
                        divergence_justification: None,
                    },
                )
                .await?;

            let movement_type_str = if diff > Decimal::ZERO {
                "ADJUSTMENT_ADD"
            } else {
                "ADJUSTMENT_SUB"
            };

            let movement_id: Uuid = sqlx::query_scalar(
                r#"SELECT id FROM stock_movements
                   WHERE warehouse_id = $1 AND catalog_item_id = $2
                     AND movement_type = $3 AND user_id = $4
                   ORDER BY created_at DESC LIMIT 1"#,
            )
            .bind(warehouse_id)
            .bind(item.catalog_item_id)
            .bind(movement_type_str)
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            self.session_repo
                .set_item_movement(item.id, movement_id)
                .await
                .map_err(ServiceError::from)?;
        }

        self.session_repo
            .transition_to_completed(
                session_id,
                payload.sei_process_number.as_deref(),
            )
            .await
            .map_err(ServiceError::from)?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar sessão atualizada".to_string(),
            ))
    }

    /// RF-049: Confirma assinatura Gov.br da sessão de inventário.
    pub async fn confirm_govbr_signature(
        &self,
        session_id: Uuid,
        signed_by: Uuid,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))?;

        if session.status != InventorySessionStatus::Completed {
            return Err(ServiceError::BadRequest(format!(
                "Assinatura Gov.br só é permitida após COMPLETED. Status atual: {:?}",
                session.status
            )));
        }

        if session.govbr_signed_at.is_some() {
            return Err(ServiceError::BadRequest(
                "Sessão já foi assinada via Gov.br".to_string(),
            ));
        }

        self.session_repo
            .confirm_govbr_signature(session_id, signed_by)
            .await
            .map_err(ServiceError::from)?;

        self.session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar sessão atualizada".to_string(),
            ))
    }

    /// Cancela sessão em OPEN, COUNTING ou RECONCILING.
    pub async fn cancel_session(
        &self,
        session_id: Uuid,
        payload: CancelInventorySessionPayload,
    ) -> Result<InventorySessionWithItemsDto, ServiceError> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Sessão de inventário não encontrada".to_string(),
            ))?;

        if session.status == InventorySessionStatus::Completed {
            return Err(ServiceError::BadRequest(
                "Sessão concluída não pode ser cancelada".to_string(),
            ));
        }
        if session.status == InventorySessionStatus::Cancelled {
            return Err(ServiceError::BadRequest(
                "Sessão já está cancelada".to_string(),
            ));
        }

        self.session_repo
            .transition_to_cancelled(session_id)
            .await
            .map_err(ServiceError::from)?;

        // Store reason in notes if provided
        if let Some(reason) = payload.reason.as_deref() {
            if !reason.trim().is_empty() {
                sqlx::query(
                    "UPDATE inventory_sessions SET notes = COALESCE(notes || ' | CANCELAMENTO: ', 'CANCELAMENTO: ') || $2 WHERE id = $1",
                )
                .bind(session_id)
                .bind(reason)
                .execute(&self.pool)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            }
        }

        self.session_repo
            .find_with_items(session_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar sessão atualizada".to_string(),
            ))
    }
}
