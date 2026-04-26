use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "financial_event_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinancialEventType {
    GlosaCriada,
    DevolucaoCriada,
    EstornoLancamento,
    EmpenhoValidado,
    EmpenhoInsuficiente,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FinancialEventDto {
    pub id: Uuid,
    pub event_type: FinancialEventType,
    pub invoice_id: Option<Uuid>,
    pub invoice_adjustment_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    pub commitment_number: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CreateFinancialEventInput {
    pub event_type: FinancialEventType,
    pub invoice_id: Option<Uuid>,
    pub invoice_adjustment_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    pub commitment_number: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_by: Option<Uuid>,
}
