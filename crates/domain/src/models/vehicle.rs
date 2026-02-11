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
    pub category_id: Option<Uuid>,
    pub name: String,
    // Technical specifications
    pub passenger_capacity: Option<i32>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub load_capacity: Option<Decimal>,
    // Fuel consumption averages (km/l)
    pub avg_consumption_min: Option<Decimal>,
    pub avg_consumption_max: Option<Decimal>,
    pub avg_consumption_target: Option<Decimal>,
    //
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Model with joined make and category names
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleModelWithDetailsDto {
    pub id: Uuid,
    pub make_id: Uuid,
    pub make_name: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub name: String,
    pub passenger_capacity: Option<i32>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub load_capacity: Option<Decimal>,
    pub avg_consumption_min: Option<Decimal>,
    pub avg_consumption_max: Option<Decimal>,
    pub avg_consumption_target: Option<Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleModelPayload {
    pub make_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub passenger_capacity: Option<i32>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub load_capacity: Option<Decimal>,
    pub avg_consumption_min: Option<Decimal>,
    pub avg_consumption_max: Option<Decimal>,
    pub avg_consumption_target: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleModelPayload {
    pub name: Option<String>,
    pub category_id: Option<Uuid>,
    pub passenger_capacity: Option<i32>,
    pub engine_displacement: Option<i32>,
    pub horsepower: Option<i32>,
    pub load_capacity: Option<Decimal>,
    pub avg_consumption_min: Option<Decimal>,
    pub avg_consumption_max: Option<Decimal>,
    pub avg_consumption_target: Option<Decimal>,
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
    // Identification
    pub license_plate: String,
    pub chassis_number: String,
    pub renavam: String,
    pub engine_number: Option<String>,
    // Mechanical components
    pub injection_pump: Option<String>,
    pub gearbox: Option<String>,
    pub differential: Option<String>,
    // Classification
    pub model_id: Uuid,
    pub color_id: Uuid,
    pub fuel_type_id: Uuid,
    pub transmission_type_id: Option<Uuid>,
    // Year
    pub manufacture_year: i32,
    pub model_year: i32,
    // Operational
    pub fleet_code: Option<String>,
    pub cost_sharing: bool,
    pub initial_mileage: Option<Decimal>,
    pub fuel_tank_capacity: Option<Decimal>,
    // Acquisition
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    // Institutional
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    // Status
    pub status: VehicleStatus,
    // Notes
    pub notes: Option<String>,
    // Soft delete
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Vehicle with all joined names (make/category come through model)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleWithDetailsDto {
    pub id: Uuid,
    pub license_plate: String,
    pub chassis_number: String,
    pub renavam: String,
    pub engine_number: Option<String>,
    // Mechanical components
    pub injection_pump: Option<String>,
    pub gearbox: Option<String>,
    pub differential: Option<String>,
    // Classification with names (via model)
    pub model_id: Uuid,
    pub model_name: String,
    pub make_name: String,
    pub category_name: Option<String>,
    pub color_id: Uuid,
    pub color_name: String,
    pub fuel_type_id: Uuid,
    pub fuel_type_name: String,
    pub transmission_type_id: Option<Uuid>,
    pub transmission_type_name: Option<String>,
    // Year
    pub manufacture_year: i32,
    pub model_year: i32,
    // Operational
    pub fleet_code: Option<String>,
    pub cost_sharing: bool,
    pub initial_mileage: Option<Decimal>,
    pub fuel_tank_capacity: Option<Decimal>,
    // Acquisition
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    // Institutional
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    // Status
    pub status: VehicleStatus,
    pub notes: Option<String>,
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
    // Mechanical components
    pub injection_pump: Option<String>,
    pub gearbox: Option<String>,
    pub differential: Option<String>,
    // Classification
    pub model_id: Uuid,
    pub color_id: Uuid,
    pub fuel_type_id: Uuid,
    pub transmission_type_id: Option<Uuid>,
    // Year
    pub manufacture_year: i32,
    pub model_year: i32,
    // Operational
    pub fleet_code: Option<String>,
    pub cost_sharing: Option<bool>,
    pub initial_mileage: Option<Decimal>,
    pub fuel_tank_capacity: Option<Decimal>,
    // Acquisition
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    // Institutional
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    pub status: Option<VehicleStatus>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehiclePayload {
    pub license_plate: Option<String>,
    pub chassis_number: Option<String>,
    pub renavam: Option<String>,
    pub engine_number: Option<String>,
    pub injection_pump: Option<String>,
    pub gearbox: Option<String>,
    pub differential: Option<String>,
    pub model_id: Option<Uuid>,
    pub color_id: Option<Uuid>,
    pub fuel_type_id: Option<Uuid>,
    pub transmission_type_id: Option<Uuid>,
    pub manufacture_year: Option<i32>,
    pub model_year: Option<i32>,
    pub fleet_code: Option<String>,
    pub cost_sharing: Option<bool>,
    pub initial_mileage: Option<Decimal>,
    pub fuel_tank_capacity: Option<Decimal>,
    pub acquisition_type: Option<AcquisitionType>,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    pub status: Option<VehicleStatus>,
    pub notes: Option<String>,
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
