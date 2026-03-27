use crate::errors::ServiceError;
use crate::services::stock_movement_service::StockMovementService;
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
            // Must reverse stock movements atomically
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
