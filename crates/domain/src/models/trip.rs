use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================
// RF-USO — Programação e Uso de Veículos
//
// FSM (DRS 4.1.2):
//   SOLICITADA → APROVADA → ALOCADA → EM_CURSO → AGUARDANDO_PC → CONCLUIDA
//                         ↘ CANCELADA (terminal)
//              ↘ REJEITADA (terminal)
//   Any non-terminal → CONFLITO_MANUAL (gestor action)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "trip_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TripStatus {
    #[sqlx(rename = "SOLICITADA")]
    Requested,
    #[sqlx(rename = "APROVADA")]
    Approved,
    #[sqlx(rename = "ALOCADA")]
    Allocated,
    #[sqlx(rename = "EM_CURSO")]
    InProgress,
    #[sqlx(rename = "AGUARDANDO_PC")]
    AwaitingAccounting,
    #[sqlx(rename = "CONCLUIDA")]
    Completed,
    #[sqlx(rename = "CANCELADA")]
    Cancelled,
    #[sqlx(rename = "REJEITADA")]
    Rejected,
    #[sqlx(rename = "CONFLITO_MANUAL")]
    ManualConflict,
}

/// Viagem/uso de veículo (RF-USO-01/02/03).
///
/// Terminology (DRS): check-out = vehicle departure (saída);
///                    check-in  = vehicle return (retorno).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleTripDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub requester_id: Option<Uuid>,
    #[sqlx(rename = "destino")]
    pub destination: String,
    #[sqlx(rename = "finalidade")]
    pub purpose: String,
    #[sqlx(rename = "passageiros")]
    pub passengers: i32,
    #[sqlx(rename = "data_saida_prevista")]
    pub planned_departure: DateTime<Utc>,
    #[sqlx(rename = "data_retorno_prevista")]
    pub planned_return: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub status: TripStatus,
    // Aprovação / Rejeição
    #[sqlx(rename = "aprovado_por")]
    pub approved_by: Option<Uuid>,
    #[sqlx(rename = "aprovado_em")]
    pub approved_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "motivo_rejeicao")]
    pub rejection_reason: Option<String>,
    // Alocação (APROVADA → ALOCADA)
    pub allocated_at: Option<DateTime<Utc>>,
    pub allocated_by: Option<Uuid>,
    // Check-out: saída do veículo (ALOCADA → EM_CURSO)
    #[sqlx(rename = "checkout_em")]
    pub checkout_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "checkout_por")]
    pub checkout_by: Option<Uuid>,
    pub checkout_km: Option<i64>,
    pub checkout_odometer_id: Option<Uuid>,
    // Check-in: retorno do veículo (EM_CURSO → AGUARDANDO_PC)
    #[sqlx(rename = "checkin_em")]
    pub checkin_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "checkin_por")]
    pub checkin_by: Option<Uuid>,
    pub checkin_km: Option<i64>,
    pub checkin_odometer_id: Option<Uuid>,
    pub waiting_pc_at: Option<DateTime<Utc>>,
    // km_percorridos = checkin_km − checkout_km (GENERATED STORED)
    #[sqlx(rename = "km_percorridos")]
    pub km_traveled: Option<i64>,
    // Cancelamento
    #[sqlx(rename = "cancelado_por")]
    pub cancelled_by: Option<Uuid>,
    #[sqlx(rename = "cancelado_em")]
    pub cancelled_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "motivo_cancelamento")]
    pub cancellation_reason: Option<String>,
    // Conflito manual
    pub conflict_reason: Option<String>,
    pub conflict_at: Option<DateTime<Utc>>,
    pub conflict_by: Option<Uuid>,
    // OCC + audit
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Solicita uma nova viagem (RF-USO-01).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTripPayload {
    pub vehicle_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub destination: String,
    pub purpose: String,
    pub passengers: Option<i32>,
    pub planned_departure: DateTime<Utc>,
    pub planned_return: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Aprova ou rejeita uma solicitação de viagem (SOLICITADA → APROVADA|REJEITADA).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewTripPayload {
    /// `true` = APROVADA; `false` = REJEITADA.
    pub approved: bool,
    pub rejection_reason: Option<String>,
    pub version: i32,
}

/// Aloca a viagem ao condutor e reserva o veículo (APROVADA → ALOCADA).
///
/// Realiza SELECT … FOR UPDATE NOWAIT no veículo para prevenir double-booking.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AllocateTripPayload {
    /// Condutor designado para a viagem.
    pub driver_id: Uuid,
    pub version: i32,
}

/// Check-out: registra a saída do veículo (ALOCADA → EM_CURSO) — RF-USO-02.
///
/// DRS: check-out = saída (departure).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckoutPayload {
    /// Leitura do hodômetro no momento da saída.
    pub odometer_departure: i64,
    /// Versão do veículo para OCC (allocation_status → EM_USO).
    pub vehicle_version: i32,
    pub version: i32,
}

/// Check-in: registra o retorno do veículo (EM_CURSO → AGUARDANDO_PC) — RF-USO-03.
///
/// DRS: check-in = retorno (return).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckinPayload {
    /// Leitura do hodômetro no retorno.
    pub odometer_return: i64,
    pub notes: Option<String>,
    /// Versão do veículo para OCC (allocation_status → LIVRE).
    pub vehicle_version: i32,
    pub version: i32,
}

/// Finaliza a prestação de contas (AGUARDANDO_PC → CONCLUIDA).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FinalizeTripPayload {
    pub version: i32,
}

/// Marca a viagem como em conflito que exige resolução manual.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetConflictPayload {
    pub conflict_reason: String,
    pub version: i32,
}

/// Cancela uma viagem (SOLICITADA|APROVADA|ALOCADA → CANCELADA).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CancelTripPayload {
    pub cancellation_reason: String,
    pub version: i32,
}

/// Filtros para listagem de viagens.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TripListFilters {
    pub vehicle_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub status: Option<TripStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
