use crate::errors::ServiceError;
use chrono::{DateTime, Utc};
use domain::{
    models::report::{FuelConsumptionDto, FleetSummaryDto, VehicleDashboardDto},
    ports::report::FleetReportRepositoryPort,
    ports::vehicle::VehicleRepositoryPort,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct FleetReportService {
    report_repo: Arc<dyn FleetReportRepositoryPort>,
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
}

impl FleetReportService {
    pub fn new(
        report_repo: Arc<dyn FleetReportRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    ) -> Self {
        Self { report_repo, vehicle_repo }
    }

    pub async fn fuel_consumption(
        &self,
        vehicle_id: Option<Uuid>,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<Vec<FuelConsumptionDto>, ServiceError> {
        self.report_repo
            .fuel_consumption(vehicle_id, data_inicio, data_fim)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn vehicle_dashboard(
        &self,
        vehicle_id: Uuid,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<VehicleDashboardDto, ServiceError> {
        self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.report_repo
            .vehicle_dashboard(vehicle_id, data_inicio, data_fim)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn fleet_summary(
        &self,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<FleetSummaryDto, ServiceError> {
        self.report_repo
            .fleet_summary(data_inicio, data_fim)
            .await
            .map_err(ServiceError::from)
    }
}
