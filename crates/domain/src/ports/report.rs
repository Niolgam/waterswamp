use crate::errors::RepositoryError;
use crate::models::report::{FuelConsumptionDto, FleetSummaryDto, VehicleDashboardDto};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait FleetReportRepositoryPort: Send + Sync {
    /// RF-REL-01: Fuel consumption per vehicle over a date range.
    async fn fuel_consumption(
        &self,
        vehicle_id: Option<Uuid>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<FuelConsumptionDto>, RepositoryError>;

    /// RF-REL-02: Consolidated dashboard for a single vehicle.
    async fn vehicle_dashboard(
        &self,
        vehicle_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<VehicleDashboardDto, RepositoryError>;

    /// RF-REL-03: Consolidated summary for the entire fleet over an interval.
    async fn fleet_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<FleetSummaryDto, RepositoryError>;
}
