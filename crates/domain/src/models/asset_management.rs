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

// ============================================================
// RF-AST-09/10 — Processo de Baixa e Desfazimento Patrimonial
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "disposal_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisposalStatus {
    Iniciado,
    EmAndamento,
    Concluido,
    Cancelado,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "disposal_destination_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisposalDestination {
    Doacao,
    Leilao,
    Sucata,
    TransferenciaOutroOrgao,
    Outro,
}

/// Processo de baixa patrimonial de veículo (RF-AST-09/10).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDisposalProcessDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub status: DisposalStatus,
    pub destino: DisposalDestination,
    pub justificativa: String,
    pub numero_laudo: String,
    pub documento_sei: Option<String>,
    pub concluido_em: Option<DateTime<Utc>>,
    pub concluido_por: Option<Uuid>,
    pub cancelado_em: Option<DateTime<Utc>>,
    pub cancelado_por: Option<Uuid>,
    pub motivo_cancelamento: Option<String>,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para iniciar um processo de baixa (RF-AST-09).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDisposalProcessPayload {
    pub destino: DisposalDestination,
    pub justificativa: String,
    /// Número do laudo técnico — obrigatório (RF-AST-09).
    pub numero_laudo: String,
    pub documento_sei: Option<String>,
    /// Versão atual do veículo para OCC (aciona INDISPONIVEL + suspende depreciação).
    pub vehicle_version: i32,
}

/// Payload para avançar o processo para EM_ANDAMENTO, CONCLUIDO ou CANCELADO.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdvanceDisposalPayload {
    pub new_status: DisposalStatus,
    pub motivo_cancelamento: Option<String>,
    pub version: i32,
}

/// Etapa SEI do processo de baixa (RF-AST-10).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDisposalStepDto {
    pub id: Uuid,
    pub disposal_id: Uuid,
    pub descricao: String,
    pub documento_sei: String,
    pub data_execucao: NaiveDate,
    pub responsavel_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Payload para adicionar uma etapa ao processo de baixa.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDisposalStepPayload {
    pub descricao: String,
    /// Documento SEI obrigatório por etapa (RF-AST-10).
    pub documento_sei: String,
    pub data_execucao: NaiveDate,
    pub responsavel_id: Option<Uuid>,
    pub notes: Option<String>,
}

// ============================================================
// RF-ADM-07 — Catálogo de Combustíveis
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetFuelCatalogDto {
    pub id: Uuid,
    pub nome: String,
    pub catmat_item_id: Option<Uuid>,
    pub unidade: String,
    pub ativo: bool,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetFuelCatalogPayload {
    pub nome: String,
    pub catmat_item_id: Option<Uuid>,
    pub unidade: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFleetFuelCatalogPayload {
    pub nome: Option<String>,
    pub catmat_item_id: Option<Uuid>,
    pub unidade: Option<String>,
    pub ativo: Option<bool>,
    pub notes: Option<String>,
}

// ============================================================
// RF-ADM-08 — Catálogo de Serviços de Manutenção
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetMaintenanceServiceDto {
    pub id: Uuid,
    pub nome: String,
    pub catser_item_id: Option<Uuid>,
    pub ativo: bool,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetMaintenanceServicePayload {
    pub nome: String,
    pub catser_item_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFleetMaintenanceServicePayload {
    pub nome: Option<String>,
    pub catser_item_id: Option<Uuid>,
    pub ativo: Option<bool>,
    pub notes: Option<String>,
}

// ============================================================
// RF-ADM-01/02 — Parâmetros do Sistema e Templates de Checklist
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetSystemParamDto {
    pub id: Uuid,
    pub chave: String,
    pub valor: String,
    pub descricao: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpsertFleetSystemParamPayload {
    pub chave: String,
    pub valor: String,
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetChecklistTemplateDto {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetChecklistTemplatePayload {
    pub nome: String,
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetChecklistItemDto {
    pub id: Uuid,
    pub template_id: Uuid,
    pub descricao: String,
    pub obrigatorio: bool,
    pub ordem: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFleetChecklistItemPayload {
    pub descricao: String,
    pub obrigatorio: Option<bool>,
    pub ordem: Option<i32>,
}
