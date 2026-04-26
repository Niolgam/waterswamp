use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "import_job_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImportJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "import_entity_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImportEntityType {
    Supplier,
    CatalogItem,
    InitialStock,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct ImportJobDto {
    pub id: Uuid,
    pub entity_type: ImportEntityType,
    pub status: ImportJobStatus,
    pub submitted_by: Option<Uuid>,
    pub total_records: i32,
    pub processed_records: i32,
    pub success_records: i32,
    pub failed_records: i32,
    pub error_log: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Import record payloads ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SupplierImportRecord {
    /// CNPJ/CPF — used as unique key (ON CONFLICT DO UPDATE)
    pub document_number: String,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogItemImportRecord {
    /// CATMAT code — used as unique key (ON CONFLICT DO UPDATE)
    pub code: String,
    pub description: String,
    /// Abbreviation of the unit of measure (e.g. "UN", "KG", "CX")
    pub unit_abbreviation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InitialStockImportRecord {
    pub warehouse_code: String,
    pub catalog_item_code: String,
    pub batch_number: Option<String>,
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    /// ISO date string YYYY-MM-DD
    pub expiration_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportSuppliersPayload {
    pub records: Vec<SupplierImportRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportCatalogItemsPayload {
    pub records: Vec<CatalogItemImportRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportInitialStockPayload {
    pub records: Vec<InitialStockImportRecord>,
}
