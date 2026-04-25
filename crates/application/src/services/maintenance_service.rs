use crate::errors::ServiceError;
use chrono::Local;
use domain::{
    models::maintenance::*,
    models::vehicle::{AllocationStatus, OperationalStatus, VehicleStatus},
    ports::maintenance::MaintenanceOrderRepositoryPort,
    ports::vehicle::{VehicleRepositoryPort, VehicleStatusHistoryRepositoryPort},
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct MaintenanceService {
    order_repo: Arc<dyn MaintenanceOrderRepositoryPort>,
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl MaintenanceService {
    pub fn new(
        order_repo: Arc<dyn MaintenanceOrderRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self { order_repo, vehicle_repo, status_history_repo }
    }

    // ── RF-MNT-01: Open work order ─────────────────────────────────────────

    pub async fn open_order(
        &self,
        vehicle_id: Uuid,
        payload: CreateMaintenanceOrderPayload,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.allocation_status != AllocationStatus::Livre {
            return Err(ServiceError::Conflict(
                "Veículo em uso — encerre a viagem antes de abrir OS (RN-FSM-01)".to_string(),
            ));
        }

        // Transition operational_status → MANUTENCAO (OCC)
        let _ = self.vehicle_repo
            .change_operational_status(
                vehicle_id,
                OperationalStatus::Manutencao,
                payload.vehicle_version,
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        let _ = self.status_history_repo
            .create(
                vehicle_id,
                Some(vehicle.status),
                VehicleStatus::InMaintenance,
                Some(&format!("OS aberta (RF-MNT-01): {}", payload.title)),
                created_by,
            )
            .await;

        let opened_date = payload.opened_date
            .unwrap_or_else(|| Local::now().date_naive());

        self.order_repo
            .create(
                vehicle_id,
                payload.order_type,
                &payload.title,
                payload.description.as_deref(),
                payload.supplier_id,
                opened_date,
                payload.expected_completion_date,
                payload.odometer_at_opening,
                payload.estimated_cost,
                payload.external_order_number.as_deref(),
                payload.documento_sei.as_deref(),
                payload.incident_id,
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-MNT-02: Advance work order (IN_PROGRESS / COMPLETED / CANCELLED) ─

    pub async fn advance_order(
        &self,
        order_id: Uuid,
        payload: AdvanceMaintenanceOrderPayload,
        updated_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, ServiceError> {
        let order = self.order_repo
            .find_by_id(order_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))?;

        let valid = match (&order.status, &payload.new_status) {
            (MaintenanceOrderStatus::Open, MaintenanceOrderStatus::InProgress) => true,
            (MaintenanceOrderStatus::Open | MaintenanceOrderStatus::InProgress,
             MaintenanceOrderStatus::Completed) => true,
            (MaintenanceOrderStatus::Open | MaintenanceOrderStatus::InProgress,
             MaintenanceOrderStatus::Cancelled) => true,
            _ => false,
        };
        if !valid {
            return Err(ServiceError::BadRequest(format!(
                "Transição inválida: {:?} → {:?}",
                order.status, payload.new_status
            )));
        }

        if payload.new_status == MaintenanceOrderStatus::Cancelled
            && payload.cancellation_reason.is_none()
        {
            return Err(ServiceError::BadRequest(
                "Motivo de cancelamento obrigatório".to_string(),
            ));
        }

        if payload.new_status == MaintenanceOrderStatus::Completed
            && payload.actual_cost.is_none()
        {
            return Err(ServiceError::BadRequest(
                "actual_cost obrigatório ao concluir a OS".to_string(),
            ));
        }

        let completed_by = if payload.new_status == MaintenanceOrderStatus::Completed {
            updated_by
        } else {
            None
        };
        let cancelled_by = if payload.new_status == MaintenanceOrderStatus::Cancelled {
            updated_by
        } else {
            None
        };

        let updated = self.order_repo
            .advance_status(
                order_id,
                payload.new_status.clone(),
                payload.actual_cost,
                payload.completion_date,
                payload.notes.as_deref(),
                payload.cancellation_reason.as_deref(),
                completed_by,
                cancelled_by,
                payload.version,
            )
            .await
            .map_err(ServiceError::from)?;

        // On completion or cancellation: vehicle → ATIVO
        if matches!(
            payload.new_status,
            MaintenanceOrderStatus::Completed | MaintenanceOrderStatus::Cancelled
        ) {
            if let Ok(Some(vehicle)) = self.vehicle_repo.find_by_id(order.vehicle_id).await {
                let _ = self.vehicle_repo
                    .change_operational_status(
                        order.vehicle_id,
                        OperationalStatus::Ativo,
                        vehicle.version,
                        updated_by,
                    )
                    .await;

                let _ = self.status_history_repo
                    .create(
                        order.vehicle_id,
                        Some(VehicleStatus::InMaintenance),
                        VehicleStatus::Active,
                        Some("OS encerrada — veículo retornou ao serviço ativo"),
                        updated_by,
                    )
                    .await;
            }
        }

        Ok(updated)
    }

    // ── RF-MNT-03: Service items ─────────────────────────────────────────────

    pub async fn add_item(
        &self,
        order_id: Uuid,
        payload: CreateMaintenanceOrderItemPayload,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderItemDto, ServiceError> {
        let order = self.order_repo
            .find_by_id(order_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))?;

        if matches!(
            order.status,
            MaintenanceOrderStatus::Completed | MaintenanceOrderStatus::Cancelled
        ) {
            return Err(ServiceError::BadRequest(
                "Não é possível adicionar itens a uma OS finalizada".to_string(),
            ));
        }

        self.order_repo
            .add_item(
                order_id,
                payload.service_id,
                &payload.description,
                payload.quantity.unwrap_or(Decimal::ONE),
                payload.unit_cost,
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_items(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<MaintenanceOrderItemDto>, ServiceError> {
        self.order_repo
            .find_by_id(order_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))?;

        self.order_repo
            .list_items(order_id)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-MNT-04: List and cost ─────────────────────────────────────────────

    pub async fn get_order(&self, id: Uuid) -> Result<MaintenanceOrderDto, ServiceError> {
        self.order_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))
    }

    pub async fn list_orders(
        &self,
        vehicle_id: Option<Uuid>,
        status: Option<MaintenanceOrderStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MaintenanceOrderDto>, i64), ServiceError> {
        self.order_repo
            .list(vehicle_id, status, limit, offset)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn cost_summary(
        &self,
        vehicle_id: Uuid,
    ) -> Result<MaintenanceCostSummaryDto, ServiceError> {
        self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.order_repo
            .cost_summary(vehicle_id)
            .await
            .map_err(ServiceError::from)
    }
}
