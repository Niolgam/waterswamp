use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::RepositoryError,
    models::dashboard::*,
};

#[async_trait]
pub trait DashboardRepositoryPort: Send + Sync {
    async fn refresh_stock_summary(&self) -> Result<(), RepositoryError>;
    async fn refresh_daily_movements(&self) -> Result<(), RepositoryError>;
    async fn refresh_supplier_performance(&self) -> Result<(), RepositoryError>;

    async fn get_stock_summary(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<WarehouseStockSummaryRow>, RepositoryError>;

    async fn get_daily_movements(
        &self,
        warehouse_id: Option<Uuid>,
        days: i32,
    ) -> Result<Vec<DailyMovementStat>, RepositoryError>;

    async fn get_supplier_performance(
        &self,
        limit: i64,
    ) -> Result<Vec<SupplierPerformanceStat>, RepositoryError>;
}
