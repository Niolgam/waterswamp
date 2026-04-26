use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::RepositoryError,
    models::alert::*,
};

#[async_trait]
pub trait StockAlertRepositoryPort: Send + Sync {
    async fn create(&self, input: CreateStockAlertInput) -> Result<StockAlertDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<StockAlertDto>, RepositoryError>;

    async fn list(
        &self,
        warehouse_id: Option<Uuid>,
        status: Option<StockAlertStatus>,
        alert_type: Option<StockAlertType>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<StockAlertDto>, i64), RepositoryError>;

    async fn acknowledge(
        &self,
        id: Uuid,
        acknowledged_by: Uuid,
    ) -> Result<StockAlertDto, RepositoryError>;

    async fn resolve(
        &self,
        id: Uuid,
        resolved_by: Uuid,
    ) -> Result<StockAlertDto, RepositoryError>;

    async fn mark_sla_breached(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Returns all OPEN/ACKNOWLEDGED alerts whose sla_deadline has passed.
    async fn find_overdue_sla(&self) -> Result<Vec<StockAlertDto>, RepositoryError>;
}
