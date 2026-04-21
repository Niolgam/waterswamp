use crate::errors::RepositoryError;
use crate::models::report::{FuelConsumptionDto, FleetSummaryDto, VehicleDashboardDto};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait FleetReportRepositoryPort: Send + Sync {
    /// RF-REL-01: Consumo de combustível por veículo num intervalo de datas.
    async fn fuel_consumption(
        &self,
        vehicle_id: Option<Uuid>,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<Vec<FuelConsumptionDto>, RepositoryError>;

    /// RF-REL-02: Dashboard consolidado de um único veículo.
    async fn vehicle_dashboard(
        &self,
        vehicle_id: Uuid,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<VehicleDashboardDto, RepositoryError>;

    /// RF-REL-03: Resumo consolidado de toda a frota no mês corrente (ou intervalo).
    async fn fleet_summary(
        &self,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<FleetSummaryDto, RepositoryError>;
}
