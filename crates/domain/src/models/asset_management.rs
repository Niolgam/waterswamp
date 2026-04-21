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
    #[sqlx(rename = "dept_origem_id")]
    pub source_dept_id: Option<Uuid>,
    #[sqlx(rename = "dept_destino_id")]
    pub target_dept_id: Uuid,
    #[sqlx(rename = "data_efetiva")]
    pub effective_date: NaiveDate,
    #[sqlx(rename = "motivo")]
    pub reason: String,
    pub documento_sei: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Payload para registrar uma transferência departamental.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleDepartmentTransferPayload {
    pub target_dept_id: Uuid,
    pub effective_date: NaiveDate,
    pub reason: String,
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
    #[sqlx(rename = "ACIDENTE")]
    Accident,
    #[sqlx(rename = "ROUBO_FURTO")]
    TheftRobbery,
    #[sqlx(rename = "INCENDIO")]
    Fire,
    #[sqlx(rename = "ALAGAMENTO")]
    Flood,
    #[sqlx(rename = "VANDALISMO")]
    Vandalism,
    #[sqlx(rename = "OUTRO")]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "vehicle_incident_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VehicleIncidentStatus {
    #[sqlx(rename = "ABERTO")]
    Open,
    #[sqlx(rename = "EM_APURACAO")]
    UnderInvestigation,
    #[sqlx(rename = "ENCERRADO_RECUPERADO")]
    ClosedRecovered,
    #[sqlx(rename = "ENCERRADO_PERDA_TOTAL")]
    ClosedTotalLoss,
}

/// Registro de sinistro de veículo (RF-AST-12).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleIncidentDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    #[sqlx(rename = "tipo")]
    pub incident_type: VehicleIncidentType,
    pub status: VehicleIncidentStatus,
    #[sqlx(rename = "data_ocorrencia")]
    pub occurred_at: DateTime<Utc>,
    #[sqlx(rename = "local_ocorrencia")]
    pub location: Option<String>,
    #[sqlx(rename = "numero_bo")]
    pub police_report_number: String,
    #[sqlx(rename = "numero_seguradora")]
    pub insurance_number: Option<String>,
    #[sqlx(rename = "apolice_id")]
    pub policy_id: Option<Uuid>,
    #[sqlx(rename = "descricao")]
    pub description: Option<String>,
    #[sqlx(rename = "notas_resolucao")]
    pub resolution_notes: Option<String>,
    #[sqlx(rename = "encerrado_em")]
    pub closed_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "encerrado_por")]
    pub closed_by: Option<Uuid>,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para registrar um sinistro. Aciona `operational_status → INDISPONIVEL`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVehicleIncidentPayload {
    pub incident_type: VehicleIncidentType,
    pub occurred_at: DateTime<Utc>,
    pub location: Option<String>,
    /// Número do Boletim de Ocorrência — obrigatório (RF-AST-12).
    pub police_report_number: String,
    pub insurance_number: Option<String>,
    pub description: Option<String>,
    /// Versão atual do veículo para OCC.
    pub vehicle_version: i32,
}

/// Payload para encerrar/atualizar status do sinistro.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVehicleIncidentPayload {
    pub status: VehicleIncidentStatus,
    pub resolution_notes: Option<String>,
    pub insurance_number: Option<String>,
    pub version: i32,
}

// ============================================================
// RF-AST-09/10 — Processo de Baixa e Desfazimento Patrimonial
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "disposal_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisposalStatus {
    #[sqlx(rename = "INICIADO")]
    Initiated,
    #[sqlx(rename = "EM_ANDAMENTO")]
    InProgress,
    #[sqlx(rename = "CONCLUIDO")]
    Completed,
    #[sqlx(rename = "CANCELADO")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "disposal_destination_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisposalDestination {
    #[sqlx(rename = "DOACAO")]
    Donation,
    #[sqlx(rename = "LEILAO")]
    Auction,
    #[sqlx(rename = "SUCATA")]
    Scrap,
    #[sqlx(rename = "TRANSFERENCIA_OUTRO_ORGAO")]
    TransferenciaOutroOrgao,
    #[sqlx(rename = "OUTRO")]
    Other,
}

/// Processo de baixa patrimonial de veículo (RF-AST-09/10).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDisposalProcessDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub status: DisposalStatus,
    #[sqlx(rename = "destino")]
    pub destination: DisposalDestination,
    #[sqlx(rename = "justificativa")]
    pub justification: String,
    #[sqlx(rename = "numero_laudo")]
    pub report_number: String,
    pub documento_sei: Option<String>,
    #[sqlx(rename = "concluido_em")]
    pub completed_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "concluido_por")]
    pub completed_by: Option<Uuid>,
    #[sqlx(rename = "cancelado_em")]
    pub cancelled_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "cancelado_por")]
    pub cancelled_by: Option<Uuid>,
    #[sqlx(rename = "motivo_cancelamento")]
    pub cancellation_reason: Option<String>,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para iniciar um processo de baixa (RF-AST-09).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDisposalProcessPayload {
    pub destination: DisposalDestination,
    pub justification: String,
    /// Número do laudo técnico — obrigatório (RF-AST-09).
    pub report_number: String,
    pub documento_sei: Option<String>,
    /// Versão atual do veículo para OCC (aciona INDISPONIVEL + suspende depreciação).
    pub vehicle_version: i32,
}

/// Payload para avançar o processo para EM_ANDAMENTO, CONCLUIDO ou CANCELADO.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdvanceDisposalPayload {
    pub new_status: DisposalStatus,
    pub cancellation_reason: Option<String>,
    pub version: i32,
}

/// Etapa SEI do processo de baixa (RF-AST-10).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDisposalStepDto {
    pub id: Uuid,
    pub disposal_id: Uuid,
    #[sqlx(rename = "descricao")]
    pub description: String,
    pub documento_sei: String,
    #[sqlx(rename = "data_execucao")]
    pub execution_date: NaiveDate,
    pub responsavel_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Payload para adicionar uma etapa ao processo de baixa.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDisposalStepPayload {
    pub description: String,
    /// Documento SEI obrigatório por etapa (RF-AST-10).
    pub documento_sei: String,
    pub execution_date: NaiveDate,
    pub responsavel_id: Option<Uuid>,
    pub notes: Option<String>,
}

// ============================================================
// RF-ADM-07 — Catálogo de Combustíveis
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetFuelCatalogDto {
    pub id: Uuid,
    #[sqlx(rename = "nome")]
    pub name: String,
    pub catmat_item_id: Option<Uuid>,
    #[sqlx(rename = "unidade")]
    pub unit: String,
    #[sqlx(rename = "ativo")]
    pub active: bool,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetFuelCatalogPayload {
    pub name: String,
    pub catmat_item_id: Option<Uuid>,
    pub unit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFleetFuelCatalogPayload {
    pub name: Option<String>,
    pub catmat_item_id: Option<Uuid>,
    pub unit: Option<String>,
    pub active: Option<bool>,
    pub notes: Option<String>,
}

// ============================================================
// RF-ADM-08 — Catálogo de Serviços de Manutenção
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetMaintenanceServiceDto {
    pub id: Uuid,
    #[sqlx(rename = "nome")]
    pub name: String,
    pub catser_item_id: Option<Uuid>,
    #[sqlx(rename = "ativo")]
    pub active: bool,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetMaintenanceServicePayload {
    pub name: String,
    pub catser_item_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFleetMaintenanceServicePayload {
    pub name: Option<String>,
    pub catser_item_id: Option<Uuid>,
    pub active: Option<bool>,
    pub notes: Option<String>,
}

// ============================================================
// RF-ADM-01/02 — Parâmetros do Sistema e Templates de Checklist
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetSystemParamDto {
    pub id: Uuid,
    #[sqlx(rename = "chave")]
    pub key: String,
    #[sqlx(rename = "valor")]
    pub value: String,
    #[sqlx(rename = "descricao")]
    pub description: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpsertFleetSystemParamPayload {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetChecklistTemplateDto {
    pub id: Uuid,
    #[sqlx(rename = "nome")]
    pub name: String,
    #[sqlx(rename = "descricao")]
    pub description: Option<String>,
    #[sqlx(rename = "ativo")]
    pub active: bool,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetChecklistTemplatePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetChecklistItemDto {
    pub id: Uuid,
    pub template_id: Uuid,
    #[sqlx(rename = "descricao")]
    pub description: String,
    #[sqlx(rename = "obrigatorio")]
    pub required: bool,
    #[sqlx(rename = "ordem")]
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetChecklistItemPayload {
    pub description: String,
    pub required: Option<bool>,
    pub order_index: Option<i32>,
}
