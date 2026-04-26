use crate::errors::ServiceError;
use crate::external::comprasnet_empenho_client::ComprasnetEmpenhoClient;
use crate::services::financial_event_service::FinancialEventPublisher;
use crate::services::stock_movement_service::StockMovementService;
use chrono::Utc;
use domain::{
    models::invoice::*,
    ports::invoice::*,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct InvoiceService {
    pool: PgPool,
    invoice_repo: Arc<dyn InvoiceRepositoryPort>,
    invoice_item_repo: Arc<dyn InvoiceItemRepositoryPort>,
    stock_movement_service: Arc<StockMovementService>,
    /// Optional Comprasnet empenho client — present when validation is configured
    empenho_client: Option<Arc<ComprasnetEmpenhoClient>>,
    financial_event_publisher: Option<Arc<FinancialEventPublisher>>,
}

impl InvoiceService {
    pub fn new(
        pool: PgPool,
        invoice_repo: Arc<dyn InvoiceRepositoryPort>,
        invoice_item_repo: Arc<dyn InvoiceItemRepositoryPort>,
        stock_movement_service: Arc<StockMovementService>,
    ) -> Self {
        Self {
            pool,
            invoice_repo,
            invoice_item_repo,
            stock_movement_service,
            empenho_client: None,
            financial_event_publisher: None,
        }
    }

    pub fn with_empenho_client(mut self, client: Arc<ComprasnetEmpenhoClient>) -> Self {
        self.empenho_client = Some(client);
        self
    }

    pub fn with_financial_event_publisher(
        mut self,
        publisher: Arc<FinancialEventPublisher>,
    ) -> Self {
        self.financial_event_publisher = Some(publisher);
        self
    }

    /// Checks Comprasnet empenho balance for a commitment number (RF-030/RN-002).
    /// Returns Ok(()) if validation is disabled, empenho is sufficient, or strict_mode=false.
    /// Returns Err(ServiceError::BadRequest) if empenho is exceeded and strict_mode=true.
    async fn validate_empenho_balance(
        &self,
        commitment_number: &str,
        total_value: Decimal,
        invoice_id_hint: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        let client = match &self.empenho_client {
            Some(c) => c,
            None => return Ok(()),
        };

        // Check if validation is enabled in system_settings
        let enabled: Option<bool> = sqlx::query_scalar(
            "SELECT (value::text)::boolean FROM system_settings WHERE key = 'comprasnet.empenho_validation_enabled'",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .flatten();

        if !enabled.unwrap_or(false) {
            return Ok(());
        }

        let strict_mode: Option<bool> = sqlx::query_scalar(
            "SELECT (value::text)::boolean FROM system_settings WHERE key = 'comprasnet.empenho_strict_mode'",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .flatten();
        let strict = strict_mode.unwrap_or(true);

        match client.validate_empenho(commitment_number, total_value).await {
            Ok(result) => {
                if !result.is_sufficient {
                    // Publish insufficiency event (fire-and-forget)
                    if let Some(ref pub_) = self.financial_event_publisher {
                        let _ = pub_
                            .publish_empenho_insuficiente(
                                commitment_number,
                                result.available_balance,
                                total_value,
                            )
                            .await;
                    }
                    return Err(ServiceError::BadRequest(format!(
                        "Saldo do empenho '{}' insuficiente. \
                         Disponível: R$ {:.2}, Solicitado: R$ {:.2} (RN-002).",
                        commitment_number, result.available_balance, total_value
                    )));
                }

                // Publish validation success event
                if let Some(ref pub_) = self.financial_event_publisher {
                    if let Some(inv_id) = invoice_id_hint {
                        if let Some(user_id) = created_by {
                            let _ = pub_
                                .publish_empenho_validado(
                                    inv_id,
                                    commitment_number,
                                    result.available_balance,
                                    total_value,
                                    user_id,
                                )
                                .await;
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    commitment_number = commitment_number,
                    error = %e,
                    "Comprasnet empenho API unavailable"
                );
                if strict {
                    Err(ServiceError::BadRequest(format!(
                        "Não foi possível validar o empenho '{}' na API Comprasnet: {}. \
                         Operação bloqueada (modo estrito ativo). \
                         Configure 'comprasnet.empenho_strict_mode=false' para modo permissivo.",
                        commitment_number, e
                    )))
                } else {
                    // Permissive mode: log and continue
                    Ok(())
                }
            }
        }
    }

    pub async fn create_invoice(
        &self,
        payload: CreateInvoicePayload,
        created_by: Option<Uuid>,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        if payload.invoice_number.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Número da nota fiscal é obrigatório".to_string(),
            ));
        }
        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "A nota fiscal deve ter ao menos um item".to_string(),
            ));
        }

        // RN-001: somente almoxarifados CENTRAL podem receber notas fiscais
        let wh_type: Option<String> = sqlx::query_scalar(
            "SELECT warehouse_type::TEXT FROM warehouses WHERE id = $1",
        )
        .bind(payload.warehouse_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        match wh_type.as_deref() {
            None => {
                return Err(ServiceError::NotFound(format!(
                    "Almoxarifado '{}' não encontrado",
                    payload.warehouse_id
                )))
            }
            Some("SECTOR") => {
                return Err(ServiceError::BadRequest(
                    "Notas fiscais só podem ser recebidas em almoxarifados do tipo CENTRAL (RN-001)."
                        .to_string(),
                ))
            }
            _ => {}
        }

        if let Some(ref key) = payload.access_key {
            if !key.is_empty()
                && self
                    .invoice_repo
                    .exists_by_access_key(key)
                    .await
                    .map_err(ServiceError::from)?
            {
                return Err(ServiceError::Conflict(format!(
                    "Nota fiscal com chave de acesso '{}' já existe",
                    key
                )));
            }
        }

        let total_freight = payload.total_freight.unwrap_or(Decimal::ZERO);
        let total_discount = payload.total_discount.unwrap_or(Decimal::ZERO);

        let invoice = self
            .invoice_repo
            .create(
                &payload.invoice_number,
                payload.series.as_deref(),
                payload.access_key.as_deref(),
                payload.issue_date,
                payload.supplier_id,
                payload.warehouse_id,
                total_freight,
                total_discount,
                payload.commitment_number.as_deref(),
                payload.purchase_order_number.as_deref(),
                payload.contract_number.as_deref(),
                payload.notes.as_deref(),
                payload.pdf_url.as_deref(),
                payload.xml_url.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        // Insert items and compute totals in Rust (replaces trg_update_invoice_totals)
        let mut total_products = Decimal::ZERO;
        for item in &payload.items {
            if item.quantity_raw <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Quantidade dos itens deve ser maior que zero".to_string(),
                ));
            }
            let factor = item.conversion_factor.unwrap_or(Decimal::ONE);
            self.invoice_item_repo
                .create(
                    invoice.id,
                    item.catalog_item_id,
                    item.unit_conversion_id,
                    item.unit_raw_id,
                    item.quantity_raw,
                    item.unit_value_raw,
                    factor,
                    item.ncm.as_deref(),
                    item.cfop.as_deref(),
                    item.cest.as_deref(),
                    item.batch_number.as_deref(),
                    item.manufacturing_date,
                    item.expiration_date,
                )
                .await
                .map_err(ServiceError::from)?;
            total_products += item.quantity_raw * item.unit_value_raw;
        }

        let total_value = total_products + total_freight - total_discount;

        // RF-030/RN-002: Validate empenho balance via Comprasnet if commitment_number provided
        if let Some(ref cn) = payload.commitment_number {
            if !cn.trim().is_empty() {
                self.validate_empenho_balance(
                    cn.trim(),
                    total_value,
                    Some(invoice.id),
                    created_by,
                )
                .await?;
            }
        }

        self.invoice_repo
            .recalculate_totals(invoice.id, total_products, total_value)
            .await
            .map_err(ServiceError::from)?;

        self.invoice_repo
            .find_with_details_by_id(invoice.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar nota fiscal criada".to_string(),
            ))
    }

    pub async fn get_invoice(&self, id: Uuid) -> Result<InvoiceWithDetailsDto, ServiceError> {
        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))
    }

    pub async fn get_invoice_items(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceItemWithDetailsDto>, ServiceError> {
        let _ = self
            .invoice_repo
            .find_by_id(invoice_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        self.invoice_item_repo
            .list_by_invoice(invoice_id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_invoice(
        &self,
        id: Uuid,
        payload: UpdateInvoicePayload,
        updated_by: Option<Uuid>,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if current.status != InvoiceStatus::Pending {
            return Err(ServiceError::BadRequest(
                "Apenas notas fiscais com status PENDING podem ser editadas".to_string(),
            ));
        }

        if let Some(ref key) = payload.access_key {
            if !key.is_empty()
                && self
                    .invoice_repo
                    .exists_by_access_key_excluding(key, id)
                    .await
                    .map_err(ServiceError::from)?
            {
                return Err(ServiceError::Conflict(format!(
                    "Nota fiscal com chave de acesso '{}' já existe",
                    key
                )));
            }
        }

        let _ = self
            .invoice_repo
            .update(
                id,
                payload.invoice_number.as_deref(),
                payload.series.as_deref(),
                payload.access_key.as_deref(),
                payload.issue_date,
                payload.supplier_id,
                payload.warehouse_id,
                payload.total_freight,
                payload.total_discount,
                payload.commitment_number.as_deref(),
                payload.purchase_order_number.as_deref(),
                payload.contract_number.as_deref(),
                payload.notes.as_deref(),
                payload.pdf_url.as_deref(),
                payload.xml_url.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar nota fiscal atualizada".to_string(),
            ))
    }

    pub async fn start_checking(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if current.status != InvoiceStatus::Pending {
            return Err(ServiceError::BadRequest(
                "Somente notas com status PENDING podem iniciar conferência".to_string(),
            ));
        }

        self.invoice_repo
            .transition_to_checking(id, user_id)
            .await
            .map_err(ServiceError::from)?;

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar nota fiscal".to_string()))
    }

    pub async fn finish_checking(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if current.status != InvoiceStatus::Checking {
            return Err(ServiceError::BadRequest(
                "Somente notas com status CHECKING podem ser conferidas".to_string(),
            ));
        }

        self.invoice_repo
            .transition_to_checked(id, user_id)
            .await
            .map_err(ServiceError::from)?;

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar nota fiscal".to_string()))
    }

    /// Posts an invoice to stock. Atomically:
    /// 1. Updates invoice status to POSTED
    /// 2. Creates ENTRY stock movements for all STOCKABLE items (replaces fn_auto_post_invoice)
    pub async fn post_invoice(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if current.status != InvoiceStatus::Checked {
            return Err(ServiceError::BadRequest(
                "Somente notas com status CHECKED podem ser lançadas no estoque".to_string(),
            ));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        sqlx::query(
            r#"UPDATE invoices SET
                status = 'POSTED',
                posted_at = NOW(),
                posted_by = $2,
                updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.stock_movement_service
            .process_invoice_entry(
                &mut tx,
                id,
                current.warehouse_id,
                &current.invoice_number,
                user_id,
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar nota fiscal".to_string()))
    }

    pub async fn reject_invoice(
        &self,
        id: Uuid,
        payload: RejectInvoicePayload,
        user_id: Uuid,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if !matches!(
            current.status,
            InvoiceStatus::Checking | InvoiceStatus::Checked
        ) {
            return Err(ServiceError::BadRequest(
                "Somente notas em conferência ou conferidas podem ser rejeitadas".to_string(),
            ));
        }

        if payload.rejection_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo de rejeição é obrigatório".to_string(),
            ));
        }

        self.invoice_repo
            .transition_to_rejected(id, &payload.rejection_reason, user_id)
            .await
            .map_err(ServiceError::from)?;

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar nota fiscal".to_string()))
    }

    /// Cancels an invoice. If POSTED, atomically reverses stock movements first.
    pub async fn cancel_invoice(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if current.status == InvoiceStatus::Cancelled {
            return Err(ServiceError::BadRequest(
                "Nota fiscal já está cancelada".to_string(),
            ));
        }

        if current.status == InvoiceStatus::Posted {
            // RN-008: NFs lançadas não aceitam cancelamento direto.
            // Use POST /invoices/{id}/compensatory-reversal dentro de 24h.
            return Err(ServiceError::BadRequest(
                "Notas fiscais lançadas no estoque não podem ser canceladas diretamente. \
                 Utilize o lançamento compensatório (POST …/compensatory-reversal) \
                 dentro de 24h do lançamento (RN-008)."
                    .to_string(),
            ));
        } else {
            self.invoice_repo
                .transition_to_cancelled(id, user_id)
                .await
                .map_err(ServiceError::from)?;
        }

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar nota fiscal".to_string()))
    }

    /// RN-008 — Lançamento compensatório: reverte movimentações de estoque de uma NF
    /// já lançada (POSTED) criando ADJUSTMENT_SUB equivalentes.
    /// Só permitido dentro de 24h após o lançamento; após isso, use glosa.
    pub async fn compensatory_reversal(
        &self,
        id: Uuid,
        payload: CompensatoryReversalPayload,
        user_id: Uuid,
    ) -> Result<InvoiceWithDetailsDto, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if current.status != InvoiceStatus::Posted {
            return Err(ServiceError::BadRequest(
                "Lançamento compensatório só é permitido para notas com status POSTED".to_string(),
            ));
        }

        let posted_at = current
            .posted_at
            .ok_or_else(|| ServiceError::Internal("Data de lançamento ausente na NF".to_string()))?;

        let elapsed = Utc::now().signed_duration_since(posted_at);
        if elapsed.num_hours() >= 24 {
            return Err(ServiceError::BadRequest(format!(
                "Janela de 24h expirada para lançamento compensatório. \
                 A NF foi lançada há {} horas. Após 24h, utilize glosa (RN-008).",
                elapsed.num_hours()
            )));
        }

        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo do lançamento compensatório é obrigatório (RN-008)".to_string(),
            ));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.stock_movement_service
            .reverse_invoice_entry(&mut tx, id, &current.invoice_number, user_id)
            .await?;

        sqlx::query(
            "UPDATE invoices SET status = 'CANCELLED', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        self.invoice_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar nota fiscal".to_string()))
    }

    pub async fn delete_invoice(&self, id: Uuid) -> Result<bool, ServiceError> {
        let current = self
            .invoice_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if !matches!(
            current.status,
            InvoiceStatus::Pending | InvoiceStatus::Rejected | InvoiceStatus::Cancelled
        ) {
            return Err(ServiceError::BadRequest(
                "Somente notas com status PENDING, REJECTED ou CANCELLED podem ser excluídas"
                    .to_string(),
            ));
        }

        self.invoice_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_invoices(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        status: Option<InvoiceStatus>,
        supplier_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<InvoiceWithDetailsDto>, i64), ServiceError> {
        self.invoice_repo
            .list(limit, offset, search, status, supplier_id, warehouse_id)
            .await
            .map_err(ServiceError::from)
    }
}
