use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================
// RF-USO — Programação e Uso de Veículos
// FSM: PENDENTE → APROVADA → CHECKIN → CONCLUIDA
//              → REJEITADA (terminal)
//  PENDENTE | APROVADA → CANCELADA (terminal)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "trip_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TripStatus {
    Pendente,
    Aprovada,
    Rejeitada,
    Checkin,
    Concluida,
    Cancelada,
}

/// Viagem/uso de veículo (RF-USO-01/02/03).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleTripDto {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub requester_id: Option<Uuid>,
    pub destino: String,
    pub finalidade: String,
    pub passageiros: i32,
    pub data_saida_prevista: DateTime<Utc>,
    pub data_retorno_prevista: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub status: TripStatus,
    // Aprovação
    pub aprovado_por: Option<Uuid>,
    pub aprovado_em: Option<DateTime<Utc>>,
    pub motivo_rejeicao: Option<String>,
    // Checkin
    pub checkin_em: Option<DateTime<Utc>>,
    pub checkin_por: Option<Uuid>,
    pub checkin_km: Option<i64>,
    pub checkin_odometer_id: Option<Uuid>,
    // Checkout
    pub checkout_em: Option<DateTime<Utc>>,
    pub checkout_por: Option<Uuid>,
    pub checkout_km: Option<i64>,
    pub checkout_odometer_id: Option<Uuid>,
    pub km_percorridos: Option<i64>,
    // Cancelamento
    pub cancelado_por: Option<Uuid>,
    pub cancelado_em: Option<DateTime<Utc>>,
    pub motivo_cancelamento: Option<String>,
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
    pub destino: String,
    pub finalidade: String,
    pub passageiros: Option<i32>,
    pub data_saida_prevista: DateTime<Utc>,
    pub data_retorno_prevista: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Aprova ou rejeita uma solicitação de viagem.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewTripPayload {
    /// `true` = APROVADA; `false` = REJEITADA.
    pub approved: bool,
    pub motivo_rejeicao: Option<String>,
    pub version: i32,
}

/// Checkin: registra saída do veículo (RF-USO-02).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckinPayload {
    /// Condutor que realiza o checkin (pode diferir do planejado).
    pub driver_id: Uuid,
    /// Leitura do hodômetro no momento da saída.
    pub km_saida: i64,
    /// Versão atual do veículo para OCC (muda allocation_status → EM_USO).
    pub vehicle_version: i32,
    pub version: i32,
}

/// Checkout: registra retorno do veículo (RF-USO-03).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckoutPayload {
    /// Leitura do hodômetro no retorno.
    pub km_retorno: i64,
    pub notes: Option<String>,
    /// Versão atual do veículo para OCC (muda allocation_status → LIVRE).
    pub vehicle_version: i32,
    pub version: i32,
}

/// Cancela uma viagem antes do checkin.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CancelTripPayload {
    pub motivo_cancelamento: String,
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
