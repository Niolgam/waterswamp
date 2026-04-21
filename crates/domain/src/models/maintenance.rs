use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================
// RF-MNT — Ordens de Serviço de Manutenção
// FSM: ABERTA → EM_EXECUCAO → CONCLUIDA | CANCELADA
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "maintenance_order_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaintenanceOrderStatus {
    Aberta,
    EmExecucao,
    Concluida,
    Cancelada,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "maintenance_order_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaintenanceOrderType {
    Preventiva,
    Corretiva,
    Recall,
    Sinistro,
}

/// Ordem de Serviço de manutenção veicular (RF-MNT-01/02).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MaintenanceOrderDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub tipo: MaintenanceOrderType,
    pub status: MaintenanceOrderStatus,
    pub titulo: String,
    pub descricao: Option<String>,
    pub fornecedor_id: Option<Uuid>,
    pub data_abertura: NaiveDate,
    pub data_prevista_conclusao: Option<NaiveDate>,
    pub data_conclusao: Option<NaiveDate>,
    pub km_abertura: Option<i64>,
    pub custo_previsto: Option<Decimal>,
    pub custo_real: Option<Decimal>,
    pub numero_os_externo: Option<String>,
    pub documento_sei: Option<String>,
    pub incident_id: Option<Uuid>,
    pub notas: Option<String>,
    pub concluido_por: Option<Uuid>,
    pub cancelado_por: Option<Uuid>,
    pub cancelado_em: Option<DateTime<Utc>>,
    pub motivo_cancelamento: Option<String>,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Abre uma nova OS — veículo → MANUTENCAO (RF-MNT-01).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaintenanceOrderPayload {
    pub tipo: MaintenanceOrderType,
    pub titulo: String,
    pub descricao: Option<String>,
    pub fornecedor_id: Option<Uuid>,
    pub data_abertura: Option<NaiveDate>,
    pub data_prevista_conclusao: Option<NaiveDate>,
    pub km_abertura: Option<i64>,
    pub custo_previsto: Option<Decimal>,
    pub numero_os_externo: Option<String>,
    pub documento_sei: Option<String>,
    pub incident_id: Option<Uuid>,
    pub notas: Option<String>,
    /// Versão atual do veículo para OCC (muda operational_status → MANUTENCAO).
    pub vehicle_version: i32,
}

/// Avança o status da OS (EM_EXECUCAO, CONCLUIDA, CANCELADA).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdvanceMaintenanceOrderPayload {
    pub new_status: MaintenanceOrderStatus,
    /// Custo real — obrigatório ao concluir.
    pub custo_real: Option<Decimal>,
    pub data_conclusao: Option<NaiveDate>,
    pub notas: Option<String>,
    pub motivo_cancelamento: Option<String>,
    pub version: i32,
}

/// Item de serviço dentro de uma OS (RF-MNT-03).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MaintenanceOrderItemDto {
    pub id: Uuid,
    pub order_id: Uuid,
    pub service_id: Option<Uuid>,
    pub descricao: String,
    pub quantidade: Decimal,
    pub custo_unitario: Option<Decimal>,
    pub custo_total: Option<Decimal>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Adiciona um item de serviço à OS.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaintenanceOrderItemPayload {
    /// ID do serviço no catálogo (fleet_maintenance_services) — opcional.
    pub service_id: Option<Uuid>,
    pub descricao: String,
    pub quantidade: Option<Decimal>,
    pub custo_unitario: Option<Decimal>,
}

/// Resumo de custo de manutenção por veículo (RF-MNT-04).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MaintenanceCostSummaryDto {
    pub vehicle_id: Uuid,
    pub total_os: i64,
    pub os_concluidas: i64,
    pub custo_total_real: Option<Decimal>,
    pub custo_total_previsto: Option<Decimal>,
}
