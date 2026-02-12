use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Fueling DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FuelingDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub fuel_type_id: Uuid,
    pub fueling_date: DateTime<Utc>,
    pub odometer_km: i32,
    pub quantity_liters: Decimal,
    pub unit_price: Decimal,
    pub total_cost: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Fueling with related entity names joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FuelingWithDetailsDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub vehicle_license_plate: Option<String>,
    pub driver_id: Uuid,
    pub driver_name: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub fuel_type_id: Uuid,
    pub fuel_type_name: Option<String>,
    pub fueling_date: DateTime<Utc>,
    pub odometer_km: i32,
    pub quantity_liters: Decimal,
    pub unit_price: Decimal,
    pub total_cost: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFuelingPayload {
    pub vehicle_id: Uuid,
    pub driver_id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub fuel_type_id: Uuid,
    pub fueling_date: DateTime<Utc>,
    pub odometer_km: i32,
    pub quantity_liters: Decimal,
    pub unit_price: Decimal,
    pub total_cost: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFuelingPayload {
    pub vehicle_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub fuel_type_id: Option<Uuid>,
    pub fueling_date: Option<DateTime<Utc>>,
    pub odometer_km: Option<i32>,
    pub quantity_liters: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub notes: Option<String>,
}
