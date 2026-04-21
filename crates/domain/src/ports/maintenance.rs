use crate::errors::RepositoryError;
use crate::models::maintenance::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait MaintenanceOrderRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_id: Uuid,
        order_type: MaintenanceOrderType,
        title: &str,
        description: Option<&str>,
        supplier_id: Option<Uuid>,
        opened_date: NaiveDate,
        expected_completion_date: Option<NaiveDate>,
        odometer_at_opening: Option<i64>,
        estimated_cost: Option<Decimal>,
        external_order_number: Option<&str>,
        documento_sei: Option<&str>,
        incident_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaintenanceOrderDto>, RepositoryError>;

    async fn advance_status(
        &self,
        id: Uuid,
        new_status: MaintenanceOrderStatus,
        actual_cost: Option<Decimal>,
        completion_date: Option<NaiveDate>,
        notes: Option<&str>,
        cancellation_reason: Option<&str>,
        completed_by: Option<Uuid>,
        cancelled_by: Option<Uuid>,
        version: i32,
    ) -> Result<MaintenanceOrderDto, RepositoryError>;

    async fn list(
        &self,
        vehicle_id: Option<Uuid>,
        status: Option<MaintenanceOrderStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MaintenanceOrderDto>, i64), RepositoryError>;

    async fn cost_summary(
        &self,
        vehicle_id: Uuid,
    ) -> Result<MaintenanceCostSummaryDto, RepositoryError>;

    // Items
    async fn add_item(
        &self,
        order_id: Uuid,
        service_id: Option<Uuid>,
        description: &str,
        quantity: Decimal,
        unit_cost: Option<Decimal>,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderItemDto, RepositoryError>;

    async fn list_items(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<MaintenanceOrderItemDto>, RepositoryError>;
}
