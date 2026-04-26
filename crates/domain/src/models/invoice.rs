use crate::models::catalog::MaterialClassification;
use crate::models::invoice_adjustment::AdjustmentItemStatus;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "invoice_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    Pending,
    Checking,
    Checked,
    Posted,
    Rejected,
    Cancelled,
}

// ============================
// Invoice DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InvoiceDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub series: Option<String>,
    pub access_key: Option<String>,
    pub issue_date: DateTime<Utc>,

    pub supplier_id: Uuid,
    pub warehouse_id: Uuid,

    pub total_products: Decimal,
    pub total_freight: Decimal,
    pub total_discount: Decimal,
    pub total_value: Decimal,

    pub status: InvoiceStatus,

    pub received_at: Option<DateTime<Utc>>,
    pub received_by: Option<Uuid>,
    pub checked_at: Option<DateTime<Utc>>,
    pub checked_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,

    pub commitment_number: Option<String>,
    pub purchase_order_number: Option<String>,
    pub contract_number: Option<String>,

    pub notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub pdf_url: Option<String>,
    pub xml_url: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Invoice with supplier and warehouse names joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InvoiceWithDetailsDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub series: Option<String>,
    pub access_key: Option<String>,
    pub issue_date: DateTime<Utc>,

    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub warehouse_id: Uuid,
    pub warehouse_name: Option<String>,

    pub total_products: Decimal,
    pub total_freight: Decimal,
    pub total_discount: Decimal,
    pub total_value: Decimal,

    pub status: InvoiceStatus,

    pub received_at: Option<DateTime<Utc>>,
    pub received_by: Option<Uuid>,
    pub checked_at: Option<DateTime<Utc>>,
    pub checked_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,

    pub commitment_number: Option<String>,
    pub purchase_order_number: Option<String>,
    pub contract_number: Option<String>,

    pub notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub pdf_url: Option<String>,
    pub xml_url: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// Invoice Item DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InvoiceItemDto {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub catalog_item_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub unit_raw_id: Uuid,

    pub quantity_raw: Decimal,
    pub unit_value_raw: Decimal,
    pub total_value: Decimal,
    pub conversion_factor: Decimal,
    pub quantity_base: Decimal,
    pub unit_value_base: Decimal,

    pub ncm: Option<String>,
    pub cfop: Option<String>,
    pub cest: Option<String>,

    pub batch_number: Option<String>,
    pub manufacturing_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,

    pub created_at: DateTime<Utc>,
}

/// Invoice item with catalog item, unit names, PDM classification and adjustment totals joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvoiceItemWithDetailsDto {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub unit_conversion_id: Option<Uuid>,
    pub unit_raw_id: Uuid,
    pub unit_raw_name: Option<String>,
    pub unit_raw_symbol: Option<String>,

    /// Classificação herdada do PDM: STOCKABLE, PERMANENT ou DIRECT_USE
    pub material_classification: MaterialClassification,

    pub quantity_raw: Decimal,
    pub unit_value_raw: Decimal,
    pub total_value: Decimal,
    pub conversion_factor: Decimal,
    pub quantity_base: Decimal,
    pub unit_value_base: Decimal,

    pub ncm: Option<String>,
    pub cfop: Option<String>,
    pub cest: Option<String>,

    pub batch_number: Option<String>,
    pub manufacturing_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,

    pub created_at: DateTime<Utc>,

    // ── Campos calculados via LEFT JOIN com invoice_adjustment_items ──────────
    /// Quantidade total ajustada (soma de todos os ajustes para este item)
    pub adjusted_quantity: Decimal,
    /// Valor total ajustado (soma de todos os ajustes para este item)
    pub adjusted_value: Decimal,
    /// Status calculado: REGULAR, PARTIAL_ADJUSTMENT ou TOTAL_ADJUSTMENT
    pub adjustment_status: AdjustmentItemStatus,
}

/// Struct intermediária para FromRow — campos brutos antes do cálculo do adjustment_status
#[derive(Debug, sqlx::FromRow)]
pub struct InvoiceItemWithDetailsRow {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub unit_conversion_id: Option<Uuid>,
    pub unit_raw_id: Uuid,
    pub unit_raw_name: Option<String>,
    pub unit_raw_symbol: Option<String>,
    pub material_classification: MaterialClassification,
    pub quantity_raw: Decimal,
    pub unit_value_raw: Decimal,
    pub total_value: Decimal,
    pub conversion_factor: Decimal,
    pub quantity_base: Decimal,
    pub unit_value_base: Decimal,
    pub ncm: Option<String>,
    pub cfop: Option<String>,
    pub cest: Option<String>,
    pub batch_number: Option<String>,
    pub manufacturing_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub adjusted_quantity: Decimal,
    pub adjusted_value: Decimal,
}

impl From<InvoiceItemWithDetailsRow> for InvoiceItemWithDetailsDto {
    fn from(row: InvoiceItemWithDetailsRow) -> Self {
        let adjustment_status =
            AdjustmentItemStatus::calculate(row.adjusted_quantity, row.quantity_base);
        InvoiceItemWithDetailsDto {
            id: row.id,
            invoice_id: row.invoice_id,
            catalog_item_id: row.catalog_item_id,
            catalog_item_name: row.catalog_item_name,
            unit_conversion_id: row.unit_conversion_id,
            unit_raw_id: row.unit_raw_id,
            unit_raw_name: row.unit_raw_name,
            unit_raw_symbol: row.unit_raw_symbol,
            material_classification: row.material_classification,
            quantity_raw: row.quantity_raw,
            unit_value_raw: row.unit_value_raw,
            total_value: row.total_value,
            conversion_factor: row.conversion_factor,
            quantity_base: row.quantity_base,
            unit_value_base: row.unit_value_base,
            ncm: row.ncm,
            cfop: row.cfop,
            cest: row.cest,
            batch_number: row.batch_number,
            manufacturing_date: row.manufacturing_date,
            expiration_date: row.expiration_date,
            created_at: row.created_at,
            adjusted_quantity: row.adjusted_quantity,
            adjusted_value: row.adjusted_value,
            adjustment_status,
        }
    }
}

// ============================
// Request Payloads
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInvoiceItemPayload {
    pub catalog_item_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub unit_raw_id: Uuid,
    pub quantity_raw: Decimal,
    pub unit_value_raw: Decimal,
    pub conversion_factor: Option<Decimal>,
    pub ncm: Option<String>,
    pub cfop: Option<String>,
    pub cest: Option<String>,
    pub batch_number: Option<String>,
    pub manufacturing_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInvoicePayload {
    pub invoice_number: String,
    pub series: Option<String>,
    pub access_key: Option<String>,
    pub issue_date: DateTime<Utc>,
    pub supplier_id: Uuid,
    pub warehouse_id: Uuid,
    pub total_freight: Option<Decimal>,
    pub total_discount: Option<Decimal>,
    pub commitment_number: Option<String>,
    pub purchase_order_number: Option<String>,
    pub contract_number: Option<String>,
    pub notes: Option<String>,
    pub pdf_url: Option<String>,
    pub xml_url: Option<String>,
    pub items: Vec<CreateInvoiceItemPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateInvoicePayload {
    pub invoice_number: Option<String>,
    pub series: Option<String>,
    pub access_key: Option<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub supplier_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub total_freight: Option<Decimal>,
    pub total_discount: Option<Decimal>,
    pub commitment_number: Option<String>,
    pub purchase_order_number: Option<String>,
    pub contract_number: Option<String>,
    pub notes: Option<String>,
    pub pdf_url: Option<String>,
    pub xml_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StartCheckingPayload {
    pub received_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckInvoicePayload {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostInvoicePayload {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RejectInvoicePayload {
    pub rejection_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CancelInvoicePayload {
    pub notes: Option<String>,
}

/// RN-008 — Lançamento compensatório de NF lançada no estoque.
/// Só é aceito dentro da janela de 24h após o posting.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompensatoryReversalPayload {
    /// Motivo obrigatório do estorno compensatório.
    pub reason: String,
}
