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

pub struct WarehouseService {
    pool: PgPool,
    warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
    stock_repo: Arc<dyn WarehouseStockRepositoryPort>,
    stock_movement_service: Arc<StockMovementService>,
    disposal_request_repo: Arc<dyn DisposalRequestRepositoryPort>,
}

impl WarehouseService {
    pub fn new(
        pool: PgPool,
        warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
        stock_repo: Arc<dyn WarehouseStockRepositoryPort>,
        stock_movement_service: Arc<StockMovementService>,
        disposal_request_repo: Arc<dyn DisposalRequestRepositoryPort>,
    ) -> Self {
        Self {
            pool,
            warehouse_repo,
            stock_repo,
            stock_movement_service,
            disposal_request_repo,
        }
    }

    // ============================
    // Warehouse CRUD
    // ============================

    pub async fn create_warehouse(
        &self,
        payload: CreateWarehousePayload,
    ) -> Result<WarehouseWithDetailsDto, ServiceError> {
        if payload.name.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Nome do almoxarifado é obrigatório".to_string(),
            ));
        }
        if payload.code.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Código do almoxarifado é obrigatório".to_string(),
            ));
        }

        if self
            .warehouse_repo
            .exists_by_code(&payload.code)
            .await
            .map_err(ServiceError::from)?
        {
            return Err(ServiceError::Conflict(format!(
                "Almoxarifado com código '{}' já existe",
                payload.code
            )));
        }

        let allows_transfers = payload.allows_transfers.unwrap_or(true);
        let is_budgetary = payload.is_budgetary.unwrap_or(false);

        let warehouse = self
            .warehouse_repo
            .create(
                &payload.name,
                &payload.code,
                payload.warehouse_type,
                payload.city_id,
                payload.responsible_user_id,
                payload.responsible_unit_id,
                allows_transfers,
                is_budgetary,
                payload.address.as_deref(),
                payload.phone.as_deref(),
                payload.email.as_deref(),
            )
            .await
            .map_err(ServiceError::from)?;

        self.warehouse_repo
            .find_with_details_by_id(warehouse.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar almoxarifado criado".to_string(),
            ))
    }

    pub async fn get_warehouse(&self, id: Uuid) -> Result<WarehouseWithDetailsDto, ServiceError> {
        self.warehouse_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))
    }

    pub async fn update_warehouse(
        &self,
        id: Uuid,
        payload: UpdateWarehousePayload,
    ) -> Result<WarehouseWithDetailsDto, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        if let Some(ref code) = payload.code {
            if self
                .warehouse_repo
                .exists_by_code_excluding(code, id)
                .await
                .map_err(ServiceError::from)?
            {
                return Err(ServiceError::Conflict(format!(
                    "Almoxarifado com código '{}' já existe",
                    code
                )));
            }
        }

        let _ = self
            .warehouse_repo
            .update(
                id,
                payload.name.as_deref(),
                payload.code.as_deref(),
                payload.warehouse_type,
                payload.city_id,
                payload.responsible_user_id,
                payload.responsible_unit_id,
                payload.allows_transfers,
                payload.is_budgetary,
                payload.address.as_deref(),
                payload.phone.as_deref(),
                payload.email.as_deref(),
                payload.is_active,
            )
            .await
            .map_err(ServiceError::from)?;

        self.warehouse_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar almoxarifado atualizado".to_string(),
            ))
    }

    pub async fn delete_warehouse(&self, id: Uuid) -> Result<bool, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        self.warehouse_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_warehouses(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        warehouse_type: Option<WarehouseType>,
        city_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<WarehouseWithDetailsDto>, i64), ServiceError> {
        self.warehouse_repo
            .list(limit, offset, search, warehouse_type, city_id, is_active)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Warehouse Stock
    // ============================

    pub async fn get_stock(&self, id: Uuid) -> Result<WarehouseStockWithDetailsDto, ServiceError> {
        self.stock_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))
    }

    pub async fn list_warehouse_stocks(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        search: Option<String>,
        is_blocked: Option<bool>,
    ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        self.stock_repo
            .list_by_warehouse(warehouse_id, limit, offset, search, is_blocked)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_stock_params(
        &self,
        id: Uuid,
        payload: UpdateStockParamsPayload,
    ) -> Result<WarehouseStockDto, ServiceError> {
        let _ = self
            .stock_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))?;

        self.stock_repo
            .update_params(
                id,
                payload.min_stock,
                payload.max_stock,
                payload.reorder_point,
                payload.resupply_days,
                payload.location.as_deref(),
                payload.secondary_location.as_deref(),
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn block_stock(
        &self,
        id: Uuid,
        payload: BlockStockPayload,
        blocked_by: Uuid,
    ) -> Result<WarehouseStockDto, ServiceError> {
        let current = self
            .stock_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))?;

        if current.is_blocked {
            return Err(ServiceError::BadRequest(
                "Estoque já está bloqueado".to_string(),
            ));
        }

        if payload.block_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo de bloqueio é obrigatório".to_string(),
            ));
        }

        self.stock_repo
            .block(id, &payload.block_reason, blocked_by)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn unblock_stock(&self, id: Uuid) -> Result<WarehouseStockDto, ServiceError> {
        let current = self
            .stock_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))?;

        if !current.is_blocked {
            return Err(ServiceError::BadRequest(
                "Estoque não está bloqueado".to_string(),
            ));
        }

        self.stock_repo
            .unblock(id)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Stock Movement Operations
    // ============================

    /// RF-009: Entrada Avulsa — donation (DONATION_IN) or inventory adjustment surplus (ADJUSTMENT_ADD).
    /// Requires origin_description (CPF/CNPJ or description).
    pub async fn create_standalone_entry(
        &self,
        warehouse_id: Uuid,
        payload: StandaloneEntryPayload,
        user_id: Uuid,
    ) -> Result<StandaloneEntryResult, ServiceError> {
        // Validate warehouse exists and check hierarchy (RN-001)
        let warehouse = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        if warehouse.warehouse_type == WarehouseType::Sector {
            return Err(ServiceError::BadRequest(
                "Entradas avulsas só são permitidas em almoxarifados do tipo CENTRAL (RN-001)."
                    .to_string(),
            ));
        }

        if payload.origin_description.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Origem da entrada avulsa é obrigatória".to_string(),
            ));
        }

        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Informe ao menos um item para a entrada".to_string(),
            ));
        }

        let movement_type = match payload.entry_type {
            StandaloneEntryType::Donation => StockMovementType::DonationIn,
            StandaloneEntryType::InventoryAdjustment => StockMovementType::AdjustmentAdd,
        };

        let entry_type_str = format!("{:?}", payload.entry_type);
        let doc_number = payload
            .document_number
            .clone()
            .unwrap_or_else(|| format!("AVULSA/{}", payload.origin_description));

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let count = payload.items.len();
        for item in &payload.items {
            if item.quantity_raw <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade deve ser maior que zero".to_string(),
                ));
            }
            if item.unit_price_base < Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Preço unitário não pode ser negativo".to_string(),
                ));
            }

            let quantity_base = item.quantity_raw * item.conversion_factor;

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: movement_type.clone(),
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: item.unit_conversion_id,
                        quantity_raw: item.quantity_raw,
                        conversion_factor: item.conversion_factor,
                        quantity_base,
                        unit_price_base: item.unit_price_base,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        related_warehouse_id: None,
                        document_number: Some(doc_number.clone()),
                        notes: item.item_notes.clone().or_else(|| payload.notes.clone()),
                        user_id,
                        batch_number: item.batch_number.clone(),
                        expiration_date: item.expiration_date,
                        divergence_justification: item.divergence_justification.clone(),
                    },
                )
                .await?;
        }

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(StandaloneEntryResult {
            movements_created: count,
            entry_type: entry_type_str,
            origin_description: payload.origin_description,
            warehouse_id,
        })
    }

    /// RF-011: Devolução de Requisição — items returned from a fulfilled requisition back to stock (RETURN).
    pub async fn create_return_entry(
        &self,
        warehouse_id: Uuid,
        payload: ReturnEntryPayload,
        user_id: Uuid,
    ) -> Result<ReturnEntryResult, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Informe ao menos um item para a devolução".to_string(),
            ));
        }

        let req_status: Option<String> =
            sqlx::query_scalar("SELECT status::TEXT FROM requisitions WHERE id = $1")
                .bind(payload.requisition_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

        match req_status.as_deref() {
            None => {
                return Err(ServiceError::NotFound(
                    "Requisição não encontrada".to_string(),
                ))
            }
            Some(s) if !matches!(s, "FULFILLED" | "PARTIALLY_FULFILLED") => {
                return Err(ServiceError::BadRequest(format!(
                    "Devolução só é permitida para requisições FULFILLED ou PARTIALLY_FULFILLED. Status: {}",
                    s
                )));
            }
            _ => {}
        }

        let req_number: String =
            sqlx::query_scalar("SELECT requisition_number FROM requisitions WHERE id = $1")
                .bind(payload.requisition_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let count = payload.items.len();
        for item in &payload.items {
            if item.quantity_raw <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade deve ser maior que zero".to_string(),
                ));
            }

            let quantity_base = item.quantity_raw * item.conversion_factor;

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: StockMovementType::Return,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: item.unit_conversion_id,
                        quantity_raw: item.quantity_raw,
                        conversion_factor: item.conversion_factor,
                        quantity_base,
                        unit_price_base: Decimal::ZERO,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: Some(payload.requisition_id),
                        requisition_item_id: None,
                        related_warehouse_id: None,
                        document_number: Some(format!("DEV/{}", req_number)),
                        notes: item.item_notes.clone().or_else(|| payload.notes.clone()),
                        user_id,
                        batch_number: item.batch_number.clone(),
                        expiration_date: item.expiration_date,
                        divergence_justification: None,
                    },
                )
                .await?;
        }

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(ReturnEntryResult {
            movements_created: count,
            requisition_id: payload.requisition_id,
            warehouse_id,
        })
    }

    /// RF-016: Cria pedido de desfazimento em AWAITING_SIGNATURE — estoque não é deduzido ainda.
    /// Gov.br signature required before stock deduction (RN-005, Ticket 1.1).
    pub async fn create_disposal_request(
        &self,
        warehouse_id: Uuid,
        payload: CreateDisposalRequestPayload,
        user_id: Uuid,
    ) -> Result<DisposalRequestWithItemsDto, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        if payload.justification.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para desfazimento (RN-005)".to_string(),
            ));
        }
        if payload.technical_opinion_url.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "URL do Parecer Técnico é obrigatória para desfazimento (RN-005/RF-016)"
                    .to_string(),
            ));
        }

        let sei_regex = regex::Regex::new(SEI_REGEX).unwrap();
        if !sei_regex.is_match(&payload.sei_process_number) {
            return Err(ServiceError::BadRequest(format!(
                "Número de processo SEI inválido. Formato esperado: NNNNN.NNNNNN/YYYY-NN. Recebido: '{}'",
                payload.sei_process_number
            )));
        }

        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Informe ao menos um item para o desfazimento".to_string(),
            ));
        }

        for item in &payload.items {
            if item.quantity_raw <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade deve ser maior que zero".to_string(),
                ));
            }
        }

        let request = self
            .disposal_request_repo
            .create(
                warehouse_id,
                &payload.sei_process_number,
                &payload.justification,
                &payload.technical_opinion_url,
                payload.notes.as_deref(),
                user_id,
            )
            .await
            .map_err(ServiceError::from)?;

        for item in &payload.items {
            self.disposal_request_repo
                .create_item(
                    request.id,
                    item.catalog_item_id,
                    item.unit_raw_id,
                    item.unit_conversion_id,
                    item.quantity_raw,
                    item.conversion_factor,
                    item.batch_number.as_deref(),
                    item.notes.as_deref(),
                )
                .await
                .map_err(ServiceError::from)?;
        }

        self.disposal_request_repo
            .find_with_items(request.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar pedido de desfazimento criado".to_string(),
            ))
    }

    /// RF-016: Confirma assinatura Gov.br e deduz estoque (LOSS) para cada item.
    pub async fn confirm_disposal_signature(
        &self,
        request_id: Uuid,
        signed_by: Uuid,
    ) -> Result<DisposalRequestWithItemsDto, ServiceError> {
        let with_items = self
            .disposal_request_repo
            .find_with_items(request_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Pedido de desfazimento não encontrado".to_string(),
            ))?;

        if with_items.request.status != DisposalRequestStatus::AwaitingSignature {
            return Err(ServiceError::BadRequest(format!(
                "Pedido não pode ser assinado. Status atual: {:?}",
                with_items.request.status
            )));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let warehouse_id = with_items.request.warehouse_id;
        let sei = with_items.request.sei_process_number.clone();
        let justification = with_items.request.justification.clone();
        let technical_opinion_url = with_items.request.technical_opinion_url.clone();

        for item in &with_items.items {
            let quantity_base = item.quantity_raw * item.conversion_factor;

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: StockMovementType::Loss,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: item.unit_conversion_id,
                        quantity_raw: item.quantity_raw,
                        conversion_factor: item.conversion_factor,
                        quantity_base,
                        unit_price_base: Decimal::ZERO,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        related_warehouse_id: None,
                        document_number: Some(sei.clone()),
                        notes: Some(format!(
                            "DESFAZIMENTO/GOV.BR — SEI: {} — Justificativa: {} — Parecer: {}",
                            sei, justification, technical_opinion_url
                        )),
                        user_id: signed_by,
                        batch_number: item.batch_number.clone(),
                        expiration_date: None,
                        divergence_justification: None,
                    },
                )
                .await?;

            let movement_id: Uuid = sqlx::query_scalar(
                r#"SELECT id FROM stock_movements
                   WHERE warehouse_id = $1 AND catalog_item_id = $2
                     AND movement_type = 'LOSS' AND user_id = $3
                   ORDER BY created_at DESC LIMIT 1"#,
            )
            .bind(warehouse_id)
            .bind(item.catalog_item_id)
            .bind(signed_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            self.disposal_request_repo
                .set_item_movement(item.id, movement_id)
                .await
                .map_err(ServiceError::from)?;
        }

        self.disposal_request_repo
            .transition_to_signed(request_id, signed_by)
            .await
            .map_err(ServiceError::from)?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.disposal_request_repo
            .find_with_items(request_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar pedido atualizado".to_string(),
            ))
    }

    /// Cancela um pedido de desfazimento em AWAITING_SIGNATURE.
    pub async fn cancel_disposal_request(
        &self,
        request_id: Uuid,
        payload: CancelDisposalRequestPayload,
        cancelled_by: Uuid,
    ) -> Result<DisposalRequestWithItemsDto, ServiceError> {
        if payload.cancellation_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo de cancelamento é obrigatório".to_string(),
            ));
        }

        let req = self
            .disposal_request_repo
            .find_by_id(request_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Pedido de desfazimento não encontrado".to_string(),
            ))?;

        if req.status == DisposalRequestStatus::Signed {
            return Err(ServiceError::BadRequest(
                "Pedido já assinado não pode ser cancelado".to_string(),
            ));
        }
        if req.status == DisposalRequestStatus::Cancelled {
            return Err(ServiceError::BadRequest(
                "Pedido já está cancelado".to_string(),
            ));
        }

        self.disposal_request_repo
            .transition_to_cancelled(request_id, cancelled_by, &payload.cancellation_reason)
            .await
            .map_err(ServiceError::from)?;

        self.disposal_request_repo
            .find_with_items(request_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar pedido atualizado".to_string(),
            ))
    }

    pub async fn get_disposal_request(
        &self,
        request_id: Uuid,
    ) -> Result<DisposalRequestWithItemsDto, ServiceError> {
        self.disposal_request_repo
            .find_with_items(request_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Pedido de desfazimento não encontrado".to_string(),
            ))
    }

    pub async fn list_disposal_requests(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<DisposalRequestStatus>,
    ) -> Result<(Vec<DisposalRequestDto>, i64), ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        self.disposal_request_repo
            .list_by_warehouse(warehouse_id, limit, offset, status)
            .await
            .map_err(ServiceError::from)
    }

    /// List stock movements for a warehouse (audit trail).
    pub async fn list_stock_movements(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        catalog_item_id: Option<Uuid>,
        movement_type: Option<String>,
    ) -> Result<(Vec<StockMovementDto>, i64), ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        let movements = sqlx::query_as::<_, StockMovementDto>(
            r#"SELECT
                sm.id,
                sm.warehouse_id,
                w.name AS warehouse_name,
                sm.catalog_item_id,
                ci.description AS catalog_item_name,
                ci.code AS catalog_item_code,
                sm.movement_type,
                sm.movement_date,
                sm.quantity_raw,
                sm.quantity_base,
                sm.unit_price_base,
                sm.total_value,
                sm.balance_before,
                sm.balance_after,
                sm.average_before,
                sm.average_after,
                sm.invoice_id,
                sm.requisition_id,
                sm.related_warehouse_id,
                rw.name AS related_warehouse_name,
                sm.document_number,
                sm.notes,
                sm.batch_number,
                sm.expiration_date,
                sm.requires_review,
                sm.user_id,
                u.username AS user_name,
                sm.created_at
               FROM stock_movements sm
               LEFT JOIN warehouses w ON w.id = sm.warehouse_id
               LEFT JOIN catmat_items ci ON ci.id = sm.catalog_item_id
               LEFT JOIN warehouses rw ON rw.id = sm.related_warehouse_id
               LEFT JOIN users u ON u.id = sm.user_id
               WHERE sm.warehouse_id = $1
                 AND ($2::UUID IS NULL OR sm.catalog_item_id = $2)
                 AND ($3::text IS NULL OR sm.movement_type::text = $3)
               ORDER BY sm.movement_date DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .bind(movement_type.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM stock_movements
               WHERE warehouse_id = $1
                 AND ($2::UUID IS NULL OR catalog_item_id = $2)
                 AND ($3::text IS NULL OR movement_type::text = $3)"#,
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .bind(movement_type.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok((movements, total))
    }

    /// RF-017: Saída por Ordem de Serviço — manual or OS-based exit (EXIT or LOSS).
    pub async fn create_manual_exit(
        &self,
        warehouse_id: Uuid,
        payload: ManualExitPayload,
        user_id: Uuid,
    ) -> Result<ManualExitResult, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        if payload.document_number.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Número do documento (OS) é obrigatório".to_string(),
            ));
        }
        if payload.justification.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para saída manual".to_string(),
            ));
        }
        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Informe ao menos um item para a saída".to_string(),
            ));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let count = payload.items.len();
        for item in &payload.items {
            if item.quantity_raw <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade deve ser maior que zero".to_string(),
                ));
            }

            let quantity_base = item.quantity_raw * item.conversion_factor;

            self.stock_movement_service
                .process_movement(
                    &mut tx,
                    ProcessMovementInput {
                        warehouse_id,
                        catalog_item_id: item.catalog_item_id,
                        movement_type: StockMovementType::Exit,
                        unit_raw_id: item.unit_raw_id,
                        unit_conversion_id: item.unit_conversion_id,
                        quantity_raw: item.quantity_raw,
                        conversion_factor: item.conversion_factor,
                        quantity_base,
                        unit_price_base: Decimal::ZERO,
                        invoice_id: None,
                        invoice_item_id: None,
                        requisition_id: None,
                        requisition_item_id: None,
                        related_warehouse_id: None,
                        document_number: Some(payload.document_number.clone()),
                        notes: Some(format!(
                            "Saída por OS — Justificativa: {}{}",
                            payload.justification,
                            item.item_notes
                                .as_ref()
                                .map(|n| format!(" — {}", n))
                                .unwrap_or_default()
                        )),
                        user_id,
                        batch_number: item.batch_number.clone(),
                        expiration_date: None,
                        divergence_justification: None,
                    },
                )
                .await?;
        }

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(ManualExitResult {
            movements_created: count,
            document_number: payload.document_number,
            warehouse_id,
        })
    }
}
