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

/// Invoice item with catalog item, unit names and PDM classification joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InvoiceItemWithDetailsDto {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub unit_conversion_id: Option<Uuid>,
    pub unit_raw_id: Uuid,
    pub unit_raw_name: Option<String>,
    pub unit_raw_symbol: Option<String>,

    /// Herdado do PDM: item entrará no estoque ao postar a NF
    pub is_stockable: bool,
    /// Herdado do PDM: item é bem permanente (patrimônio) — não entra no estoque
    pub is_permanent: bool,

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
