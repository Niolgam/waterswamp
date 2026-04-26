use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "stock_alert_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockAlertType {
    LowStock,
    BatchExpiring,
    BatchExpired,
    RequisitionOverdue,
    QuotaExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "stock_alert_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockAlertStatus {
    Open,
    Acknowledged,
    Resolved,
    SlaBreached,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct StockAlertDto {
    pub id: Uuid,
    pub alert_type: StockAlertType,
    pub status: StockAlertStatus,
    pub warehouse_id: Option<Uuid>,
    pub catalog_item_id: Option<Uuid>,
    pub batch_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub sla_deadline: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub sla_breached_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateStockAlertInput {
    pub alert_type: StockAlertType,
    pub warehouse_id: Option<Uuid>,
    pub catalog_item_id: Option<Uuid>,
    pub batch_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub sla_hours: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AcknowledgeAlertPayload {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListAlertsQuery {
    pub warehouse_id: Option<Uuid>,
    pub status: Option<StockAlertStatus>,
    pub alert_type: Option<StockAlertType>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
