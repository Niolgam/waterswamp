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

    // ── RF-USO-01: Request trip ─────────────────────────────────────────────

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
                &payload.destination,
                &payload.purpose,
                payload.passengers.unwrap_or(0),
                payload.planned_departure,
                payload.planned_return,
                payload.notes.as_deref(),
                requester_id,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-01: Approve / Reject ─────────────────────────────────────────

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

        if trip.status != TripStatus::Pending {
            return Err(ServiceError::BadRequest(
                "Apenas viagens PENDING podem ser revisadas".to_string(),
            ));
        }

        if payload.approved {
            self.trip_repo
                .approve(trip_id, reviewer_id, payload.version)
                .await
                .map_err(ServiceError::from)
        } else {
            let reason = payload.rejection_reason.ok_or_else(|| {
                ServiceError::BadRequest("Motivo de rejeição obrigatório".to_string())
            })?;
            self.trip_repo
                .reject(trip_id, &reason, reviewer_id, payload.version)
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

        if trip.status != TripStatus::Approved {
            return Err(ServiceError::BadRequest(
                "Checkin só é possível em viagens APPROVED".to_string(),
            ));
        }

        // Register odometer reading (source: CHECKIN_GESTOR, always VALIDATED)
        let odometer = self.odometer_repo
            .create(
                trip.vehicle_id,
                Decimal::from(payload.odometer_departure),
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

        // allocation_status → EM_USO (OCC on vehicle)
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
                payload.odometer_departure,
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

        if let Some(km_departure) = trip.checkin_km {
            if payload.odometer_return < km_departure {
                return Err(ServiceError::BadRequest(format!(
                    "odometer_return ({}) deve ser >= odometer_departure ({})",
                    payload.odometer_return, km_departure
                )));
            }
        }

        // Register odometer reading (source: CHECKOUT_CONDUTOR, always VALIDATED)
        let odometer = self.odometer_repo
            .create(
                trip.vehicle_id,
                Decimal::from(payload.odometer_return),
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

        // allocation_status → LIVRE (OCC on vehicle)
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
                payload.odometer_return,
                Some(odometer.id),
                user_id,
                payload.notes.as_deref(),
                payload.version,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-04: Cancel ───────────────────────────────────────────────────

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

        if !matches!(trip.status, TripStatus::Pending | TripStatus::Approved) {
            return Err(ServiceError::BadRequest(
                "Cancelamento só é possível em viagens PENDING ou APPROVED".to_string(),
            ));
        }

        self.trip_repo
            .cancel(trip_id, &payload.cancellation_reason, user_id, payload.version)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-04: Fetch and list ───────────────────────────────────────────

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
