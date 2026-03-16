use crate::errors::ServiceError;
use domain::{
    models::invoice::*,
    ports::invoice::*,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct InvoiceService {
    invoice_repo: Arc<dyn InvoiceRepositoryPort>,
    invoice_item_repo: Arc<dyn InvoiceItemRepositoryPort>,
}

impl InvoiceService {
    pub fn new(
        invoice_repo: Arc<dyn InvoiceRepositoryPort>,
        invoice_item_repo: Arc<dyn InvoiceItemRepositoryPort>,
    ) -> Self {
        Self {
            invoice_repo,
            invoice_item_repo,
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

        // Deduplicate access_key
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

        // Insert items — the DB trigger fn_update_invoice_totals() recalculates totals
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
        }

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
        // Ensure the invoice exists
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

        // Only allow edits on PENDING invoices
        if current.status != InvoiceStatus::Pending {
            return Err(ServiceError::BadRequest(
                "Apenas notas fiscais com status PENDING podem ser editadas".to_string(),
            ));
        }

        // Deduplicate access_key if changing it
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

        // The DB trigger fn_auto_post_invoice() handles stock movements automatically
        self.invoice_repo
            .transition_to_posted(id, user_id)
            .await
            .map_err(ServiceError::from)?;

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

        // If POSTED, the DB trigger fn_auto_post_invoice() reverses stock movements automatically
        self.invoice_repo
            .transition_to_cancelled(id, user_id)
            .await
            .map_err(ServiceError::from)?;

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
