use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "abc_curve_classification_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbcClassification {
    A,
    B,
    C,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct AbcAnalysisResultDto {
    pub id: Uuid,
    pub run_at: DateTime<Utc>,
    pub warehouse_id: Option<Uuid>,
    pub catalog_item_id: Uuid,
    pub classification: AbcClassification,
    pub total_value: Decimal,
    pub cumulative_percentage: Decimal,
    pub rank_position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RunAbcInput {
    pub warehouse_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AbcSummary {
    pub warehouse_id: Option<Uuid>,
    pub run_at: DateTime<Utc>,
    pub total_items: i64,
    pub class_a_count: i64,
    pub class_b_count: i64,
    pub class_c_count: i64,
    pub class_a_value: Decimal,
    pub class_b_value: Decimal,
    pub class_c_value: Decimal,
}
