use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "fine_severity_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FineSeverity {
    #[serde(rename = "LIGHT")]
    Light,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "SERIOUS")]
    Serious,
    #[serde(rename = "VERY_SERIOUS")]
    VerySErious,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "fine_payment_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinePaymentStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "PAID")]
    Paid,
    #[serde(rename = "OVERDUE")]
    Overdue,
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "UNDER_APPEAL")]
    UnderAppeal,
}

// ============================
// Vehicle Fine Type DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleFineTypeDto {
    pub id: Uuid,
    pub code: String,
    pub description: String,
    pub severity: FineSeverity,
    pub points: i32,
    pub fine_amount: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleFineTypePayload {
    pub code: String,
    pub description: String,
    pub severity: FineSeverity,
    pub points: i32,
    pub fine_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleFineTypePayload {
    pub code: Option<String>,
    pub description: Option<String>,
    pub severity: Option<FineSeverity>,
    pub points: Option<i32>,
    pub fine_amount: Option<Decimal>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle Fine DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleFineDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub fine_type_id: Uuid,
    pub supplier_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub auto_number: Option<String>,
    pub fine_date: DateTime<Utc>,
    pub notification_date: Option<DateTime<Utc>>,
    pub due_date: DateTime<Utc>,
    pub location: Option<String>,
    pub sei_process_number: Option<String>,
    pub fine_amount: Decimal,
    pub discount_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub payment_date: Option<DateTime<Utc>>,
    pub payment_status: FinePaymentStatus,
    pub notes: Option<String>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Fine with related entity names joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleFineWithDetailsDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub vehicle_license_plate: Option<String>,
    pub fine_type_id: Uuid,
    pub fine_type_code: Option<String>,
    pub fine_type_description: Option<String>,
    pub fine_type_severity: Option<FineSeverity>,
    pub fine_type_points: Option<i32>,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub driver_id: Option<Uuid>,
    pub driver_name: Option<String>,
    pub auto_number: Option<String>,
    pub fine_date: DateTime<Utc>,
    pub notification_date: Option<DateTime<Utc>>,
    pub due_date: DateTime<Utc>,
    pub location: Option<String>,
    pub sei_process_number: Option<String>,
    pub fine_amount: Decimal,
    pub discount_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub payment_date: Option<DateTime<Utc>>,
    pub payment_status: FinePaymentStatus,
    pub notes: Option<String>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleFinePayload {
    pub vehicle_id: Uuid,
    pub fine_type_id: Uuid,
    pub supplier_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub auto_number: Option<String>,
    pub fine_date: DateTime<Utc>,
    pub notification_date: Option<DateTime<Utc>>,
    pub due_date: DateTime<Utc>,
    pub location: Option<String>,
    pub sei_process_number: Option<String>,
    pub fine_amount: Decimal,
    pub discount_amount: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleFinePayload {
    pub vehicle_id: Option<Uuid>,
    pub fine_type_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub auto_number: Option<String>,
    pub fine_date: Option<DateTime<Utc>>,
    pub notification_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub sei_process_number: Option<String>,
    pub fine_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub paid_amount: Option<Decimal>,
    pub payment_date: Option<DateTime<Utc>>,
    pub payment_status: Option<FinePaymentStatus>,
    pub notes: Option<String>,
}
