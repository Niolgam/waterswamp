use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseStockSummaryRow {
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub warehouse_code: String,
    pub total_items: i64,
    pub total_stock_value: Option<Decimal>,
    pub low_stock_count: i64,
    pub blocked_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct DailyMovementStat {
    pub movement_date: NaiveDate,
    pub warehouse_id: Uuid,
    pub movement_type: String,
    pub movement_count: i64,
    pub total_quantity: Decimal,
    pub total_value: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SupplierPerformanceStat {
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub quality_score: Decimal,
    pub total_invoices: i64,
    pub total_adjustments: i64,
    pub avg_invoice_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardRefreshResult {
    pub refreshed_views: Vec<String>,
}
