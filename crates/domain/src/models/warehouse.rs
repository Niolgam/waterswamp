use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "warehouse_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarehouseType {
    Central,
    Sector,
}

// ============================
// Warehouse DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub warehouse_type: WarehouseType,
    pub city_id: Uuid,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: bool,
    pub is_budgetary: bool,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse with city and state names joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseWithDetailsDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub warehouse_type: WarehouseType,
    pub city_id: Uuid,
    pub city_name: Option<String>,
    pub state_abbreviation: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: bool,
    pub is_budgetary: bool,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// Warehouse Stock DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseStockDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,

    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    pub average_unit_value: Decimal,

    pub min_stock: Option<Decimal>,
    pub max_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub resupply_days: Option<i32>,

    pub location: Option<String>,
    pub secondary_location: Option<String>,

    pub is_blocked: bool,
    pub block_reason: Option<String>,
    pub blocked_at: Option<DateTime<Utc>>,
    pub blocked_by: Option<Uuid>,

    pub last_entry_at: Option<DateTime<Utc>>,
    pub last_exit_at: Option<DateTime<Utc>>,
    pub last_inventory_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse stock with catalog item name, unit, and warehouse name joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseStockWithDetailsDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: Option<String>,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub catalog_item_code: Option<String>,
    pub unit_symbol: Option<String>,
    pub unit_name: Option<String>,

    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    /// quantity - reserved_quantity (when not blocked)
    pub available_quantity: Decimal,
    pub average_unit_value: Decimal,
    pub total_value: Decimal,

    pub min_stock: Option<Decimal>,
    pub max_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub resupply_days: Option<i32>,

    pub location: Option<String>,
    pub secondary_location: Option<String>,

    pub is_blocked: bool,
    pub block_reason: Option<String>,
    pub blocked_at: Option<DateTime<Utc>>,
    pub blocked_by: Option<Uuid>,

    pub last_entry_at: Option<DateTime<Utc>>,
    pub last_exit_at: Option<DateTime<Utc>>,
    pub last_inventory_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// Request Payloads
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWarehousePayload {
    pub name: String,
    pub code: String,
    pub warehouse_type: WarehouseType,
    pub city_id: Uuid,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: Option<bool>,
    pub is_budgetary: Option<bool>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateWarehousePayload {
    pub name: Option<String>,
    pub code: Option<String>,
    pub warehouse_type: Option<WarehouseType>,
    pub city_id: Option<Uuid>,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: Option<bool>,
    pub is_budgetary: Option<bool>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

/// Update stock control parameters (min/max/reorder/location) — does NOT affect quantity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateStockParamsPayload {
    pub min_stock: Option<Decimal>,
    pub max_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub resupply_days: Option<i32>,
    pub location: Option<String>,
    pub secondary_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockStockPayload {
    pub block_reason: String,
}
