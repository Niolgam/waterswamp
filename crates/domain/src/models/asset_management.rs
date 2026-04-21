use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================
// RF-AST-06 — Transferência Departamental
// ============================================================

/// Registro imutável de uma transferência de departamento (RF-AST-06).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDepartmentTransferDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub dept_origem_id: Option<Uuid>,
    pub dept_destino_id: Uuid,
    pub data_efetiva: NaiveDate,
    pub motivo: String,
    pub documento_sei: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Payload para registrar uma transferência departamental.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleDepartmentTransferPayload {
    pub dept_destino_id: Uuid,
    pub data_efetiva: NaiveDate,
    pub motivo: String,
    pub documento_sei: Option<String>,
    pub notes: Option<String>,
}

// ============================================================
// RF-AST-11 — Configuração de Depreciação por Categoria
// ============================================================

/// Configuração de depreciação linear por categoria de veículo (RF-AST-11).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct DepreciationConfigDto {
    pub id: Uuid,
    pub vehicle_category_id: Uuid,
    pub useful_life_years: Decimal,
    pub residual_value_min: Decimal,
    pub method: String,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para criar/atualizar configuração de depreciação.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpsertDepreciationConfigPayload {
    pub vehicle_category_id: Uuid,
    pub useful_life_years: Decimal,
    pub residual_value_min: Decimal,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

/// Resultado do cálculo de depreciação de um veículo em uma data.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DepreciationCalculationDto {
    pub vehicle_id: Uuid,
    pub purchase_value: Decimal,
    pub acquisition_date: NaiveDate,
    pub useful_life_years: Decimal,
    pub residual_value_min: Decimal,
    pub months_elapsed: i64,
    pub depreciation_monthly: Decimal,
    pub accumulated_depreciation: Decimal,
    /// Valor residual estimado atual: max(purchase_value - accumulated, residual_value_min)
    pub estimated_residual_value: Decimal,
    pub is_fully_depreciated: bool,
}

// ============================================================
// RF-AST-12 — Sinistros
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "vehicle_incident_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VehicleIncidentType {
    Acidente,
    RouboFurto,
    Incendio,
    Alagamento,
    Vandalismo,
    Outro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "vehicle_incident_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VehicleIncidentStatus {
    Aberto,
    EmApuracao,
    EncerrradoRecuperado,
    EncerradoPerdaTotal,
}

/// Registro de sinistro de veículo (RF-AST-12).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleIncidentDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub tipo: VehicleIncidentType,
    pub status: VehicleIncidentStatus,
    pub data_ocorrencia: DateTime<Utc>,
    pub local_ocorrencia: Option<String>,
    pub numero_bo: String,
    pub numero_seguradora: Option<String>,
    pub apolice_id: Option<Uuid>,
    pub descricao: Option<String>,
    pub notas_resolucao: Option<String>,
    pub encerrado_em: Option<DateTime<Utc>>,
    pub encerrado_por: Option<Uuid>,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para registrar um sinistro. Aciona `operational_status → INDISPONIVEL`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleIncidentPayload {
    pub tipo: VehicleIncidentType,
    pub data_ocorrencia: DateTime<Utc>,
    pub local_ocorrencia: Option<String>,
    /// Número do Boletim de Ocorrência — obrigatório (RF-AST-12).
    pub numero_bo: String,
    pub numero_seguradora: Option<String>,
    pub descricao: Option<String>,
    /// Versão atual do veículo para OCC.
    pub vehicle_version: i32,
}

/// Payload para encerrar/atualizar status do sinistro.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleIncidentPayload {
    pub status: VehicleIncidentStatus,
    pub notas_resolucao: Option<String>,
    pub numero_seguradora: Option<String>,
    pub version: i32,
}
