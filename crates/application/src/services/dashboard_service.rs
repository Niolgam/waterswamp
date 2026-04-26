use std::sync::Arc;

use domain::{
    models::dashboard::*,
    ports::dashboard::DashboardRepositoryPort,
};
use uuid::Uuid;

use crate::errors::ServiceError;

pub struct DashboardService {
    repo: Arc<dyn DashboardRepositoryPort>,
}

impl DashboardService {
    pub fn new(repo: Arc<dyn DashboardRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn refresh_all(&self) -> Result<DashboardRefreshResult, ServiceError> {
        self.repo
            .refresh_stock_summary()
            .await
            .map_err(ServiceError::from)?;
        self.repo
            .refresh_daily_movements()
            .await
            .map_err(ServiceError::from)?;
        self.repo
            .refresh_supplier_performance()
            .await
            .map_err(ServiceError::from)?;

        Ok(DashboardRefreshResult {
            refreshed_views: vec![
                "mv_warehouse_stock_summary".into(),
                "mv_daily_movements".into(),
                "mv_supplier_performance".into(),
            ],
        })
    }

    pub async fn get_stock_summary(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<WarehouseStockSummaryRow>, ServiceError> {
        self.repo
            .get_stock_summary(warehouse_id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_daily_movements(
        &self,
        warehouse_id: Option<Uuid>,
        days: Option<i32>,
    ) -> Result<Vec<DailyMovementStat>, ServiceError> {
        let days = days.unwrap_or(30).max(1).min(365);
        self.repo
            .get_daily_movements(warehouse_id, days)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_supplier_performance(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<SupplierPerformanceStat>, ServiceError> {
        let limit = limit.unwrap_or(20).max(1).min(100);
        self.repo
            .get_supplier_performance(limit)
            .await
            .map_err(ServiceError::from)
    }
}
