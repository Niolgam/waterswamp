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
    #[sqlx(rename = "ABERTA")]
    Open,
    #[sqlx(rename = "EM_EXECUCAO")]
    InProgress,
    #[sqlx(rename = "CONCLUIDA")]
    Completed,
    #[sqlx(rename = "CANCELADA")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "maintenance_order_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaintenanceOrderType {
    #[sqlx(rename = "PREVENTIVA")]
    Preventive,
    #[sqlx(rename = "CORRETIVA")]
    Corrective,
    Recall,
    #[sqlx(rename = "SINISTRO")]
    Incident,
}

/// Ordem de Serviço de manutenção veicular (RF-MNT-01/02).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MaintenanceOrderDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    #[sqlx(rename = "tipo")]
    pub order_type: MaintenanceOrderType,
    pub status: MaintenanceOrderStatus,
    #[sqlx(rename = "titulo")]
    pub title: String,
    #[sqlx(rename = "descricao")]
    pub description: Option<String>,
    #[sqlx(rename = "fornecedor_id")]
    pub supplier_id: Option<Uuid>,
    #[sqlx(rename = "data_abertura")]
    pub opened_date: NaiveDate,
    #[sqlx(rename = "data_prevista_conclusao")]
    pub expected_completion_date: Option<NaiveDate>,
    #[sqlx(rename = "data_conclusao")]
    pub completion_date: Option<NaiveDate>,
    #[sqlx(rename = "km_abertura")]
    pub odometer_at_opening: Option<i64>,
    #[sqlx(rename = "custo_previsto")]
    pub estimated_cost: Option<Decimal>,
    #[sqlx(rename = "custo_real")]
    pub actual_cost: Option<Decimal>,
    #[sqlx(rename = "numero_os_externo")]
    pub external_order_number: Option<String>,
    pub documento_sei: Option<String>,
    pub incident_id: Option<Uuid>,
    #[sqlx(rename = "notas")]
    pub notes: Option<String>,
    #[sqlx(rename = "concluido_por")]
    pub completed_by: Option<Uuid>,
    #[sqlx(rename = "cancelado_por")]
    pub cancelled_by: Option<Uuid>,
    #[sqlx(rename = "cancelado_em")]
    pub cancelled_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "motivo_cancelamento")]
    pub cancellation_reason: Option<String>,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Abre uma nova OS — veículo → MANUTENCAO (RF-MNT-01).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaintenanceOrderPayload {
    pub order_type: MaintenanceOrderType,
    pub title: String,
    pub description: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub opened_date: Option<NaiveDate>,
    pub expected_completion_date: Option<NaiveDate>,
    pub odometer_at_opening: Option<i64>,
    pub estimated_cost: Option<Decimal>,
    pub external_order_number: Option<String>,
    pub documento_sei: Option<String>,
    pub incident_id: Option<Uuid>,
    pub notes: Option<String>,
    /// Versão atual do veículo para OCC (muda operational_status → MANUTENCAO).
    pub vehicle_version: i32,
}

/// Avança o status da OS (EM_EXECUCAO, CONCLUIDA, CANCELADA).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdvanceMaintenanceOrderPayload {
    pub new_status: MaintenanceOrderStatus,
    /// Custo real — obrigatório ao concluir.
    pub actual_cost: Option<Decimal>,
    pub completion_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub cancellation_reason: Option<String>,
    pub version: i32,
}

/// Item de serviço dentro de uma OS (RF-MNT-03).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MaintenanceOrderItemDto {
    pub id: Uuid,
    pub order_id: Uuid,
    pub service_id: Option<Uuid>,
    #[sqlx(rename = "descricao")]
    pub description: String,
    #[sqlx(rename = "quantidade")]
    pub quantity: Decimal,
    #[sqlx(rename = "custo_unitario")]
    pub unit_cost: Option<Decimal>,
    #[sqlx(rename = "custo_total")]
    pub total_cost: Option<Decimal>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Adiciona um item de serviço à OS.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaintenanceOrderItemPayload {
    /// ID do serviço no catálogo (fleet_maintenance_services) — opcional.
    pub service_id: Option<Uuid>,
    pub description: String,
    pub quantity: Option<Decimal>,
    pub unit_cost: Option<Decimal>,
}

/// Resumo de custo de manutenção por veículo (RF-MNT-04).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct MaintenanceCostSummaryDto {
    pub vehicle_id: Uuid,
    #[sqlx(rename = "total_os")]
    pub total_orders: i64,
    #[sqlx(rename = "os_concluidas")]
    pub completed_orders: i64,
    #[sqlx(rename = "custo_total_real")]
    pub total_actual_cost: Option<Decimal>,
    #[sqlx(rename = "custo_total_previsto")]
    pub total_estimated_cost: Option<Decimal>,
}
