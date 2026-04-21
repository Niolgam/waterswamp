use crate::errors::ServiceError;
use chrono::Utc;
use domain::{
    models::trip::*,
    models::vehicle::{AllocationStatus, OperationalStatus},
    models::odometer::{FonteLeitura, StatusLeitura},
    ports::trip::VehicleTripRepositoryPort,
    ports::vehicle::{VehicleRepositoryPort, VehicleStatusHistoryRepositoryPort},
    ports::odometer::OdometerReadingRepositoryPort,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct TripService {
    trip_repo: Arc<dyn VehicleTripRepositoryPort>,
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    odometer_repo: Arc<dyn OdometerReadingRepositoryPort>,
    #[allow(dead_code)]
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl TripService {
    pub fn new(
        trip_repo: Arc<dyn VehicleTripRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        odometer_repo: Arc<dyn OdometerReadingRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self { trip_repo, vehicle_repo, odometer_repo, status_history_repo }
    }

    // ── RF-USO-01: Solicitar viagem ─────────────────────────────────────────

    pub async fn request_trip(
        &self,
        payload: CreateTripPayload,
        requester_id: Option<Uuid>,
    ) -> Result<VehicleTripDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(payload.vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.allocation_status != AllocationStatus::Livre {
            return Err(ServiceError::Conflict(
                "Veículo indisponível para programação (allocation_status ≠ LIVRE)".to_string(),
            ));
        }

        if vehicle.operational_status != OperationalStatus::Ativo {
            return Err(ServiceError::Conflict(
                "Veículo inoperante — não pode ser programado".to_string(),
            ));
        }

        self.trip_repo
            .create(
                payload.vehicle_id,
                payload.driver_id,
                requester_id,
                &payload.destino,
                &payload.finalidade,
                payload.passageiros.unwrap_or(0),
                payload.data_saida_prevista,
                payload.data_retorno_prevista,
                payload.notes.as_deref(),
                requester_id,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-01: Aprovar / Rejeitar ───────────────────────────────────────

    pub async fn review_trip(
        &self,
        trip_id: Uuid,
        payload: ReviewTripPayload,
        reviewer_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        let trip = self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        if trip.status != TripStatus::Pendente {
            return Err(ServiceError::BadRequest(
                "Apenas viagens PENDENTE podem ser revisadas".to_string(),
            ));
        }

        if payload.approved {
            self.trip_repo
                .approve(trip_id, reviewer_id, payload.version)
                .await
                .map_err(ServiceError::from)
        } else {
            let motivo = payload.motivo_rejeicao.ok_or_else(|| {
                ServiceError::BadRequest("Motivo de rejeição obrigatório".to_string())
            })?;
            self.trip_repo
                .reject(trip_id, &motivo, reviewer_id, payload.version)
                .await
                .map_err(ServiceError::from)
        }
    }

    // ── RF-USO-02: Checkin ──────────────────────────────────────────────────

    pub async fn checkin(
        &self,
        trip_id: Uuid,
        payload: CheckinPayload,
        user_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        let trip = self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        if trip.status != TripStatus::Aprovada {
            return Err(ServiceError::BadRequest(
                "Checkin só é possível em viagens APROVADAS".to_string(),
            ));
        }

        // Registra leitura de hodômetro (fonte: CHECKIN_GESTOR, sempre VALIDADO)
        let odometer = self.odometer_repo
            .create(
                trip.vehicle_id,
                Decimal::from(payload.km_saida),
                FonteLeitura::CheckinGestor,
                Some(trip_id),
                Utc::now(),
                StatusLeitura::Validado,
                None,
                Uuid::new_v4(),
                Some(user_id),
            )
            .await
            .map_err(ServiceError::from)?;

        // allocation_status → EM_USO (OCC sobre veículo)
        let _ = self.vehicle_repo
            .change_allocation_status(
                trip.vehicle_id,
                AllocationStatus::EmUso,
                payload.vehicle_version,
                Some(user_id),
            )
            .await
            .map_err(ServiceError::from)?;

        self.trip_repo
            .checkin(
                trip_id,
                payload.driver_id,
                payload.km_saida,
                Some(odometer.id),
                user_id,
                payload.version,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-03: Checkout ─────────────────────────────────────────────────

    pub async fn checkout(
        &self,
        trip_id: Uuid,
        payload: CheckoutPayload,
        user_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        let trip = self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        if trip.status != TripStatus::Checkin {
            return Err(ServiceError::BadRequest(
                "Checkout só é possível em viagens em CHECKIN".to_string(),
            ));
        }

        if let Some(km_saida) = trip.checkin_km {
            if payload.km_retorno < km_saida {
                return Err(ServiceError::BadRequest(format!(
                    "km_retorno ({}) deve ser >= km_saida ({})",
                    payload.km_retorno, km_saida
                )));
            }
        }

        // Registra leitura de hodômetro (fonte: CHECKOUT_CONDUTOR, sempre VALIDADO)
        let odometer = self.odometer_repo
            .create(
                trip.vehicle_id,
                Decimal::from(payload.km_retorno),
                FonteLeitura::CheckoutCondutor,
                Some(trip_id),
                Utc::now(),
                StatusLeitura::Validado,
                None,
                Uuid::new_v4(),
                Some(user_id),
            )
            .await
            .map_err(ServiceError::from)?;

        // allocation_status → LIVRE (OCC sobre veículo)
        let _ = self.vehicle_repo
            .change_allocation_status(
                trip.vehicle_id,
                AllocationStatus::Livre,
                payload.vehicle_version,
                Some(user_id),
            )
            .await
            .map_err(ServiceError::from)?;

        self.trip_repo
            .checkout(
                trip_id,
                payload.km_retorno,
                Some(odometer.id),
                user_id,
                payload.notes.as_deref(),
                payload.version,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-04: Cancelar ─────────────────────────────────────────────────

    pub async fn cancel_trip(
        &self,
        trip_id: Uuid,
        payload: CancelTripPayload,
        user_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        let trip = self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        if !matches!(trip.status, TripStatus::Pendente | TripStatus::Aprovada) {
            return Err(ServiceError::BadRequest(
                "Cancelamento só é possível em viagens PENDENTE ou APROVADA".to_string(),
            ));
        }

        self.trip_repo
            .cancel(trip_id, &payload.motivo_cancelamento, user_id, payload.version)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-04: Busca e listagem ─────────────────────────────────────────

    pub async fn get_trip(&self, id: Uuid) -> Result<VehicleTripDto, ServiceError> {
        self.trip_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))
    }

    pub async fn list_trips(
        &self,
        filters: TripListFilters,
    ) -> Result<(Vec<VehicleTripDto>, i64), ServiceError> {
        self.trip_repo
            .list(
                filters.vehicle_id,
                filters.driver_id,
                filters.status,
                filters.limit.unwrap_or(50),
                filters.offset.unwrap_or(0),
            )
            .await
            .map_err(ServiceError::from)
    }
}
