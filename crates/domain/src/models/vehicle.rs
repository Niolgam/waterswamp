use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums — FSM Dual-Axis (DRS v3.2, seção 4.1.1)
// ============================

/// Eixo 1 — aptidão operacional do veículo.
///
/// Transições válidas:
///   ATIVO → MANUTENCAO    (abertura de OS ou alerta preventivo)
///   ATIVO → INDISPONIVEL  (sinistro ou início de processo de baixa)
///   MANUTENCAO → ATIVO    (conclusão de OS com allocation_status = LIVRE)
///   INDISPONIVEL → ATIVO  (recuperação de sinistro aprovada pelo Gestor)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "operational_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationalStatus {
    Ativo,
    Manutencao,
    Indisponivel,
}

/// Eixo 2 — vínculo do veículo a viagens.
///
/// Transições válidas (exigem operational_status = ATIVO — RN-FSM-01):
///   LIVRE → RESERVADO  (aprovação de viagem e alocação — RF-VIG-04)
///   RESERVADO → EM_USO (check-out — RF-VIG-09)
///   EM_USO → LIVRE     (check-in com odômetro validado — RF-VIG-11)
///   RESERVADO → LIVRE  (cancelamento — RF-VIG-05)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "allocation_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AllocationStatus {
    Livre,
    Reservado,
    EmUso,
}

/// Enum legado — usado apenas para entradas do histórico anteriores à v3.2.
/// Novos registros de histórico usam `old_operational_status` / `new_operational_status`.
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
    // FSM — Eixo 1: aptidão operacional (DRS 4.1.1)
    pub operational_status: OperationalStatus,
    // FSM — Eixo 2: vínculo a viagens (DRS 4.1.1)
    pub allocation_status: AllocationStatus,
    // Status legado (mantido enquanto consumidores migram)
    pub status: VehicleStatus,
    // Notes
    pub notes: Option<String>,
    // Last odometer (atualizado pelo trigger de fuelings; será substituído
    // pela série temporal leituras_hodometro no Ticket 0.2)
    pub last_odometer_km: Option<i32>,
    pub last_odometer_date: Option<DateTime<Utc>>,
    pub last_fueling_id: Option<Uuid>,
    // OCC — versão para concorrência otimista (DRS 4.2 / RNF-01)
    pub version: i32,
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

/// Veículo com nomes resolvidos via JOIN (make/category via model)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleWithDetailsDto {
    pub id: Uuid,
    pub license_plate: String,
    pub chassis_number: String,
    pub renavam: String,
    pub engine_number: Option<String>,
    pub injection_pump: Option<String>,
    pub gearbox: Option<String>,
    pub differential: Option<String>,
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
    pub manufacture_year: i32,
    pub model_year: i32,
    pub fleet_code: Option<String>,
    pub cost_sharing: bool,
    pub initial_mileage: Option<Decimal>,
    pub fuel_tank_capacity: Option<Decimal>,
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    // FSM dual-axis
    pub operational_status: OperationalStatus,
    pub allocation_status: AllocationStatus,
    // Status legado
    pub status: VehicleStatus,
    pub notes: Option<String>,
    pub last_odometer_km: Option<i32>,
    pub last_odometer_date: Option<DateTime<Utc>>,
    pub last_fueling_id: Option<Uuid>,
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
    pub injection_pump: Option<String>,
    pub gearbox: Option<String>,
    pub differential: Option<String>,
    pub model_id: Uuid,
    pub color_id: Uuid,
    pub fuel_type_id: Uuid,
    pub transmission_type_id: Option<Uuid>,
    pub manufacture_year: i32,
    pub model_year: i32,
    pub fleet_code: Option<String>,
    pub cost_sharing: Option<bool>,
    pub initial_mileage: Option<Decimal>,
    pub fuel_tank_capacity: Option<Decimal>,
    pub acquisition_type: AcquisitionType,
    pub acquisition_date: Option<NaiveDate>,
    pub purchase_value: Option<Decimal>,
    pub patrimony_number: Option<String>,
    pub department_id: Option<Uuid>,
    /// Ignorado — novos veículos sempre iniciam com ATIVO/LIVRE.
    /// Mantido apenas para compatibilidade com testes existentes.
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
    /// Ignorado — transições de estado passam por `ChangeOperationalStatusPayload`.
    /// Mantido apenas para compatibilidade com testes existentes.
    pub status: Option<VehicleStatus>,
    pub notes: Option<String>,
    /// Versão atual para OCC. Obrigatório quando qualquer campo for enviado.
    pub version: Option<i32>,
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
    // ── Eixo legado (entradas pré-v3.2) ──────────────────────────────────
    pub old_status: Option<VehicleStatus>,
    pub new_status: VehicleStatus,
    // ── FSM dual-axis (v3.2+) ─────────────────────────────────────────────
    pub old_operational_status: Option<OperationalStatus>,
    pub new_operational_status: Option<OperationalStatus>,
    pub old_allocation_status: Option<AllocationStatus>,
    pub new_allocation_status: Option<AllocationStatus>,
    // ──────────────────────────────────────────────────────────────────────
    pub reason: Option<String>,
    pub changed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Transição de status legado (backward-compat com endpoints existentes).
/// Novos consumidores devem usar `ChangeOperationalStatusPayload`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChangeVehicleStatusPayload {
    pub status: VehicleStatus,
    pub reason: Option<String>,
}

/// Transição explícita do Eixo 1 via FSM (DRS 4.1.1 / RF-AST-05).
/// O cliente deve enviar a `version` atual para garantir OCC (RNF-01).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChangeOperationalStatusPayload {
    pub operational_status: OperationalStatus,
    pub reason: Option<String>,
    /// Versão atual do veículo. Obrigatório para OCC.
    pub version: i32,
}
