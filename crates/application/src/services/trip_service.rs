use crate::errors::ServiceError;
use chrono::Utc;
use domain::{
    models::trip::*,
    models::vehicle::{AllocationStatus, OperationalStatus},
    models::odometer::{FonteLeitura, StatusLeitura},
    ports::driver::DriverRepositoryPort,
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
    driver_repo: Arc<dyn DriverRepositoryPort>,
    odometer_repo: Arc<dyn OdometerReadingRepositoryPort>,
    #[allow(dead_code)]
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl TripService {
    pub fn new(
        trip_repo: Arc<dyn VehicleTripRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        driver_repo: Arc<dyn DriverRepositoryPort>,
        odometer_repo: Arc<dyn OdometerReadingRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self { trip_repo, vehicle_repo, driver_repo, odometer_repo, status_history_repo }
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

    // ── RF-USO-01: Review (approve / reject) ────────────────────────────────

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

        if trip.status != TripStatus::Requested {
            return Err(ServiceError::BadRequest(
                "Apenas viagens SOLICITADA podem ser revisadas".to_string(),
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

    // ── RF-VIG-04: Allocate trip (APROVADA → ALOCADA) ───────────────────────

    pub async fn allocate_trip(
        &self,
        trip_id: Uuid,
        payload: AllocateTripPayload,
        allocator_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        let trip = self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        if trip.status != TripStatus::Approved {
            return Err(ServiceError::BadRequest(
                "Apenas viagens APROVADA podem ser alocadas".to_string(),
            ));
        }

        let vehicle = self.vehicle_repo
            .find_by_id(trip.vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.allocation_status != AllocationStatus::Livre {
            return Err(ServiceError::Conflict(
                "Veículo não está disponível (allocation_status ≠ LIVRE)".to_string(),
            ));
        }

        if vehicle.operational_status != OperationalStatus::Ativo {
            return Err(ServiceError::Conflict(
                "Veículo inoperante — não pode ser alocado".to_string(),
            ));
        }

        // Allocate trip with pessimistic vehicle lock (FOR UPDATE NOWAIT).
        let allocated = self.trip_repo
            .allocate(trip_id, trip.vehicle_id, payload.driver_id, allocator_id, payload.version)
            .await
            .map_err(ServiceError::from)?;

        // Mark vehicle as reserved.
        let _ = self.vehicle_repo
            .change_allocation_status(
                trip.vehicle_id,
                AllocationStatus::Reservado,
                vehicle.version,
                Some(allocator_id),
            )
            .await
            .map_err(ServiceError::from)?;

        Ok(allocated)
    }

    // ── RF-USO-02: Checkout — vehicle departure (ALOCADA → EM_CURSO) ────────

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

        if trip.status != TripStatus::Allocated {
            return Err(ServiceError::BadRequest(
                "Check-out só é possível em viagens ALOCADA".to_string(),
            ));
        }

        // ── CA-04: Validate driver CNH (RN-FSM-02) ──────────────────────────
        let driver_id = trip.driver_id.ok_or_else(|| {
            ServiceError::BadRequest("Condutor não designado na viagem".to_string())
        })?;

        let driver = self.driver_repo
            .find_by_id(driver_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Condutor não encontrado".to_string()))?;

        if !driver.is_active {
            return Err(ServiceError::BadRequest(
                "Condutor está inativo — check-out bloqueado".to_string(),
            ));
        }

        // Block if CNH expires before planned return date.
        if let Some(planned_return) = trip.planned_return {
            let return_date = planned_return.date_naive();
            if driver.cnh_expiration < return_date {
                return Err(ServiceError::BadRequest(format!(
                    "CNH do condutor vence em {} — antes do retorno previsto em {}. Check-out bloqueado.",
                    driver.cnh_expiration, return_date
                )));
            }
        }

        // ── CA-03: Register odometer — source CheckoutCondutor (Peso 3) ─────
        let odometer = self.odometer_repo
            .create(
                trip.vehicle_id,
                Decimal::from(payload.odometer_departure),
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
            .checkout(
                trip_id,
                payload.odometer_departure,
                Some(odometer.id),
                user_id,
                payload.version,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-USO-03: Checkin — vehicle return (EM_CURSO → AGUARDANDO_PC) ──────

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

        if trip.status != TripStatus::InProgress {
            return Err(ServiceError::BadRequest(
                "Check-in só é possível em viagens EM_CURSO".to_string(),
            ));
        }

        if let Some(km_departure) = trip.checkout_km {
            if payload.odometer_return < km_departure {
                return Err(ServiceError::BadRequest(format!(
                    "odometer_return ({}) deve ser >= odometer_departure ({})",
                    payload.odometer_return, km_departure
                )));
            }
        }

        // ── CA-03: Register odometer — source CheckinCondutor (Peso 2) ──────
        let odometer = self.odometer_repo
            .create(
                trip.vehicle_id,
                Decimal::from(payload.odometer_return),
                FonteLeitura::CheckinCondutor,
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
            .checkin(
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

    // ── RF-USO: Finalize (AGUARDANDO_PC → CONCLUIDA) ────────────────────────

    pub async fn finalize_trip(
        &self,
        trip_id: Uuid,
        payload: FinalizeTripPayload,
        user_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        let trip = self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        if trip.status != TripStatus::AwaitingAccounting {
            return Err(ServiceError::BadRequest(
                "Finalização só é possível em viagens AGUARDANDO_PC".to_string(),
            ));
        }

        self.trip_repo
            .finalize(trip_id, user_id, payload.version)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-ADM-06: Set manual conflict ──────────────────────────────────────

    pub async fn set_conflict(
        &self,
        trip_id: Uuid,
        payload: SetConflictPayload,
        user_id: Uuid,
    ) -> Result<VehicleTripDto, ServiceError> {
        self.trip_repo
            .find_by_id(trip_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Viagem não encontrada".to_string()))?;

        self.trip_repo
            .set_conflict(trip_id, &payload.conflict_reason, user_id, payload.version)
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

        if !matches!(trip.status, TripStatus::Requested | TripStatus::Approved | TripStatus::Allocated) {
            return Err(ServiceError::BadRequest(
                "Cancelamento só é possível em viagens SOLICITADA, APROVADA ou ALOCADA".to_string(),
            ));
        }

        // If the vehicle was reserved, release it back to LIVRE.
        if trip.status == TripStatus::Allocated {
            let vehicle = self.vehicle_repo
                .find_by_id(trip.vehicle_id)
                .await
                .map_err(ServiceError::from)?;

            if let Some(v) = vehicle {
                if v.allocation_status == AllocationStatus::Reservado {
                    let _ = self.vehicle_repo
                        .change_allocation_status(
                            trip.vehicle_id,
                            AllocationStatus::Livre,
                            v.version,
                            Some(user_id),
                        )
                        .await
                        .map_err(ServiceError::from)?;
                }
            }
        }

        self.trip_repo
            .cancel(trip_id, &payload.cancellation_reason, user_id, payload.version)
            .await
            .map_err(ServiceError::from)
    }

    // ── Fetch and list ──────────────────────────────────────────────────────

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
