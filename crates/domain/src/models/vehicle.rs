use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "vehicle_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VehicleStatus {
    Active,
    InMaintenance,
    Reserved,
    Inactive,
    Decommissioning,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "acquisition_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcquisitionType {
    Purchase,
    Donation,
    Cession,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "document_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    Crlv,
    Invoice,
    DonationTerm,
    InsurancePolicy,
    TechnicalReport,
    Photo,
    Other,
}

// ============================
// Vehicle Category DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleCategoryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleCategoryPayload {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleCategoryPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle Make DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleMakeDto {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleMakePayload {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleMakePayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle Model DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleModelDto {
    pub id: Uuid,
    pub make_id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleModelPayload {
    pub make_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleModelPayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle Color DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleColorDto {
    pub id: Uuid,
    pub name: String,
    pub hex_code: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleColorPayload {
    pub name: String,
    pub hex_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleColorPayload {
    pub name: Option<String>,
    pub hex_code: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle Fuel Type DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleFuelTypeDto {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleFuelTypePayload {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleFuelTypePayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle Transmission Type DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleTransmissionTypeDto {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleTransmissionTypePayload {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleTransmissionTypePayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Vehicle DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDto {
    pub id: Uuid,
    pub license_plate: String,
    pub chassis_number: String,
    pub renavam: String,
    pub engine_number: Option<String>,
    pub category_id: Uuid,
    pub make_id: Uuid,
    pub model_id: Uuid,
    pub color_id: Uuid,
    pub fuel_type_id: Uuid,
    pub transmission_type_id: Option<Uuid>,
    pub manufacture_year: i32,
    pub model_year: i32,
    pub passenger_capacity: Option<i32>,
    pub load_capacity_kg: Option<Decimal>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    pub status: VehicleStatus,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleWithDetailsDto {
    pub id: Uuid,
    pub license_plate: String,
    pub chassis_number: String,
    pub renavam: String,
    pub engine_number: Option<String>,
    // Classification with names
    pub category_id: Uuid,
    pub category_name: String,
    pub make_id: Uuid,
    pub make_name: String,
    pub model_id: Uuid,
    pub model_name: String,
    pub color_id: Uuid,
    pub color_name: String,
    pub fuel_type_id: Uuid,
    pub fuel_type_name: String,
    pub transmission_type_id: Option<Uuid>,
    pub transmission_type_name: Option<String>,
    // Year
    pub manufacture_year: i32,
    pub model_year: i32,
    // Technical specs
    pub passenger_capacity: Option<i32>,
    pub load_capacity_kg: Option<Decimal>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    // Acquisition
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    // Institutional
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    // Status
    pub status: VehicleStatus,
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehiclePayload {
    pub license_plate: String,
    pub chassis_number: String,
    pub renavam: String,
    pub engine_number: Option<String>,
    pub category_id: Uuid,
    pub make_id: Uuid,
    pub model_id: Uuid,
    pub color_id: Uuid,
    pub fuel_type_id: Uuid,
    pub transmission_type_id: Option<Uuid>,
    pub manufacture_year: i32,
    pub model_year: i32,
    pub passenger_capacity: Option<i32>,
    pub load_capacity_kg: Option<Decimal>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    pub status: Option<VehicleStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehiclePayload {
    pub license_plate: Option<String>,
    pub chassis_number: Option<String>,
    pub renavam: Option<String>,
    pub engine_number: Option<String>,
    pub category_id: Option<Uuid>,
    pub make_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub color_id: Option<Uuid>,
    pub fuel_type_id: Option<Uuid>,
    pub transmission_type_id: Option<Uuid>,
    pub manufacture_year: Option<i32>,
    pub model_year: Option<i32>,
    pub passenger_capacity: Option<i32>,
    pub load_capacity_kg: Option<Decimal>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub acquisition_type: Option<AcquisitionType>,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    pub status: Option<VehicleStatus>,
}

// ============================
// Vehicle Document DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDocumentDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub document_type: DocumentType,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub description: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleDocumentPayload {
    pub vehicle_id: Uuid,
    pub document_type: DocumentType,
    pub description: Option<String>,
}

// ============================
// Vehicle Status History DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleStatusHistoryDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub old_status: Option<VehicleStatus>,
    pub new_status: VehicleStatus,
    pub reason: Option<String>,
    pub changed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChangeVehicleStatusPayload {
    pub status: VehicleStatus,
    pub reason: Option<String>,
}
