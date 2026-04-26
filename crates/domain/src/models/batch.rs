use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Warehouse Batch Stocks (RF-021 FEFO)
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseBatchStockDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,
    pub batch_number: String,
    pub expiration_date: Option<NaiveDate>,
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    pub is_quarantined: bool,
    pub quarantine_reason: Option<String>,
    pub quarantined_at: Option<DateTime<Utc>>,
    pub quarantined_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a FEFO exit — lists which batches were consumed and in what quantities.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FefoExitResult {
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,
    pub total_quantity_exited: Decimal,
    pub batches_consumed: Vec<BatchConsumptionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchConsumptionDetail {
    pub batch_number: String,
    pub expiration_date: Option<NaiveDate>,
    pub quantity_consumed: Decimal,
    pub movement_id: Option<Uuid>,
}

/// Input for a FEFO-driven exit (RF-021)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FefoExitPayload {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    /// If None, FEFO engine auto-selects batches. If Some, exits only from this batch.
    pub batch_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub requisition_item_id: Option<Uuid>,
    pub document_number: String,
    pub notes: Option<String>,
}

// ============================
// Batch Quality Occurrences (RF-043)
// ============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "batch_occurrence_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchOccurrenceStatus {
    Open,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "batch_occurrence_severity_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchOccurrenceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl BatchOccurrenceSeverity {
    /// Returns true if this severity level triggers automatic quarantine.
    pub fn triggers_quarantine(&self) -> bool {
        matches!(self, BatchOccurrenceSeverity::High | BatchOccurrenceSeverity::Critical)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "batch_occurrence_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchOccurrenceType {
    Contamination,
    PhysicalDamage,
    ExpiryNear,
    Expired,
    NonConformance,
    StorageFault,
    QuantityDivergence,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct BatchQualityOccurrenceDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,
    pub batch_number: String,
    pub occurrence_type: BatchOccurrenceType,
    pub severity: BatchOccurrenceSeverity,
    pub status: BatchOccurrenceStatus,
    pub description: String,
    pub evidence_url: Option<String>,
    pub sei_process_number: Option<String>,
    pub corrective_action: Option<String>,
    pub resolved_notes: Option<String>,
    pub quarantine_triggered: bool,
    pub occurred_at: DateTime<Utc>,
    pub reported_by: Uuid,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateBatchQualityOccurrencePayload {
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,
    pub batch_number: String,
    pub occurrence_type: BatchOccurrenceType,
    pub severity: BatchOccurrenceSeverity,
    pub description: String,
    pub evidence_url: Option<String>,
    pub sei_process_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResolveOccurrencePayload {
    pub corrective_action: String,
    pub resolved_notes: Option<String>,
    /// If true, release the quarantine on the batch after resolution.
    pub release_quarantine: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CloseOccurrencePayload {
    pub resolved_notes: Option<String>,
}
