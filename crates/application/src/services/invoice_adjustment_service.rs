use crate::errors::ServiceError;
use crate::services::financial_event_service::FinancialEventPublisher;
use crate::services::stock_movement_service::StockMovementService;
use crate::services::supplier_service::SupplierService;
use domain::{
    models::catalog::MaterialClassification,
    models::invoice::InvoiceStatus,
    models::invoice_adjustment::*,
    ports::invoice::InvoiceRepositoryPort,
    ports::invoice_adjustment::InvoiceAdjustmentRepositoryPort,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct InvoiceAdjustmentService {
    pool: PgPool,
    invoice_repo: Arc<dyn InvoiceRepositoryPort>,
    adjustment_repo: Arc<dyn InvoiceAdjustmentRepositoryPort>,
    stock_movement_service: Arc<StockMovementService>,
    financial_event_publisher: Option<Arc<FinancialEventPublisher>>,
    supplier_service: Option<Arc<SupplierService>>,
}

impl InvoiceAdjustmentService {
    pub fn new(
        pool: PgPool,
        invoice_repo: Arc<dyn InvoiceRepositoryPort>,
        adjustment_repo: Arc<dyn InvoiceAdjustmentRepositoryPort>,
        stock_movement_service: Arc<StockMovementService>,
    ) -> Self {
        Self {
            pool,
            invoice_repo,
            adjustment_repo,
            stock_movement_service,
            financial_event_publisher: None,
            supplier_service: None,
        }
    }

    pub fn with_financial_event_publisher(mut self, publisher: Arc<FinancialEventPublisher>) -> Self {
        self.financial_event_publisher = Some(publisher);
        self
    }

    pub fn with_supplier_service(mut self, supplier_service: Arc<SupplierService>) -> Self {
        self.supplier_service = Some(supplier_service);
        self
    }

    pub async fn list_adjustments(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceAdjustmentWithItemsDto>, ServiceError> {
        self.invoice_repo
            .find_by_id(invoice_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        self.adjustment_repo
            .list_by_invoice(invoice_id)
            .await
            .map_err(ServiceError::from)
    }

    /// Creates an invoice adjustment (glosa) atomically:
    /// 1. Validates invoice is POSTED
    /// 2. Inserts adjustment header + items in a transaction
    /// 3. For STOCKABLE items with adjusted_quantity > 0, creates ADJUSTMENT_SUB stock movement
    pub async fn create_adjustment(
        &self,
        invoice_id: Uuid,
        payload: CreateInvoiceAdjustmentPayload,
        user_id: Uuid,
    ) -> Result<InvoiceAdjustmentWithItemsDto, ServiceError> {
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo da glosa é obrigatório".to_string(),
            ));
        }
        if payload.items.is_empty() {
            return Err(ServiceError::BadRequest(
                "A glosa deve ter ao menos um item".to_string(),
            ));
        }

        let invoice = self
            .invoice_repo
            .find_by_id(invoice_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Nota fiscal não encontrada".to_string()))?;

        if invoice.status != InvoiceStatus::Posted {
            return Err(ServiceError::BadRequest(
                "Somente notas fiscais POSTED podem receber glosas".to_string(),
            ));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // Insert adjustment header
        let adjustment = sqlx::query_as::<_, InvoiceAdjustmentDto>(
            r#"INSERT INTO invoice_adjustments (invoice_id, reason, created_by)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(invoice_id)
        .bind(&payload.reason)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let mut item_details = Vec::with_capacity(payload.items.len());

        for item_payload in &payload.items {
            let adj_qty = item_payload.adjusted_quantity.unwrap_or(Decimal::ZERO);
            let adj_val = item_payload.adjusted_value.unwrap_or(Decimal::ZERO);

            if adj_qty <= Decimal::ZERO && adj_val <= Decimal::ZERO {
                return Err(ServiceError::BadRequest(
                    "Cada item de glosa deve ter quantidade ou valor ajustado > 0".to_string(),
                ));
            }

            // Verify item belongs to this invoice and get its details
            #[derive(sqlx::FromRow)]
            struct InvoiceItemRow {
                catalog_item_id: Uuid,
                unit_raw_id: Uuid,
                unit_conversion_id: Option<Uuid>,
                material_classification: MaterialClassification,
                catalog_item_name: Option<String>,
            }

            let item_detail = sqlx::query_as::<_, InvoiceItemRow>(
                r#"SELECT ii.catalog_item_id, ii.unit_raw_id, ii.unit_conversion_id,
                          COALESCE(pdm.material_classification, 'STOCKABLE'::material_classification_enum) AS material_classification,
                          ci.description AS catalog_item_name
                   FROM invoice_items ii
                   LEFT JOIN catmat_items ci ON ci.id = ii.catalog_item_id
                   LEFT JOIN catmat_pdms pdm ON pdm.id = ci.pdm_id
                   WHERE ii.id = $1 AND ii.invoice_id = $2"#,
            )
            .bind(item_payload.invoice_item_id)
            .bind(invoice_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?
            .ok_or_else(|| {
                ServiceError::BadRequest(format!(
                    "Item {} não pertence à nota fiscal {}",
                    item_payload.invoice_item_id, invoice_id
                ))
            })?;

            // Insert adjustment item
            let adj_item = sqlx::query_as::<_, InvoiceAdjustmentItemDto>(
                r#"INSERT INTO invoice_adjustment_items (
                    adjustment_id, invoice_item_id, adjusted_quantity, adjusted_value, notes
                   ) VALUES ($1, $2, $3, $4, $5)
                   RETURNING *"#,
            )
            .bind(adjustment.id)
            .bind(item_payload.invoice_item_id)
            .bind(adj_qty)
            .bind(adj_val)
            .bind(item_payload.notes.as_deref())
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

            // For STOCKABLE items with quantity adjustment > 0, create ADJUSTMENT_SUB
            if item_detail.material_classification == MaterialClassification::Stockable
                && adj_qty > Decimal::ZERO
            {
                self.stock_movement_service
                    .process_adjustment_sub(
                        &mut tx,
                        invoice.warehouse_id,
                        item_detail.catalog_item_id,
                        item_detail.unit_raw_id,
                        item_detail.unit_conversion_id,
                        adj_qty,
                        invoice_id,
                        item_payload.invoice_item_id,
                        &format!("GLOSA NF {}", invoice.invoice_number),
                        item_payload.notes.as_deref(),
                        user_id,
                    )
                    .await?;
            }

            item_details.push(InvoiceAdjustmentItemDetailDto {
                id: adj_item.id,
                adjustment_id: adj_item.adjustment_id,
                invoice_item_id: adj_item.invoice_item_id,
                catalog_item_name: item_detail.catalog_item_name,
                adjusted_quantity: adj_item.adjusted_quantity,
                adjusted_value: adj_item.adjusted_value,
                notes: adj_item.notes,
                created_at: adj_item.created_at,
            });
        }

        tx.commit()
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let result = InvoiceAdjustmentWithItemsDto {
            id: adjustment.id,
            invoice_id: adjustment.invoice_id,
            reason: adjustment.reason,
            created_by: adjustment.created_by,
            created_at: adjustment.created_at,
            items: item_details,
        };

        // RF-028: Publish GLOSA_CRIADA financial event (fire-and-forget)
        if let Some(ref pub_) = self.financial_event_publisher {
            let total_adjusted_value: Decimal = result.items.iter()
                .map(|i| i.adjusted_value)
                .sum();
            let _ = pub_
                .publish_glosa_criada(
                    invoice_id,
                    result.id,
                    invoice.supplier_id,
                    invoice.warehouse_id,
                    total_adjusted_value,
                    user_id,
                )
                .await;
        }

        // RF-039: Auto-update supplier quality score
        if let Some(ref svc) = self.supplier_service {
            let _ = svc.penalize_quality_score(invoice.supplier_id).await;
        }

        Ok(result)
    }
}
