use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::dashboard::*,
    ports::dashboard::DashboardRepositoryPort,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DashboardRepositoryPort for DashboardRepository {
    async fn refresh_stock_summary(&self) -> Result<(), RepositoryError> {
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_warehouse_stock_summary")
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn refresh_daily_movements(&self) -> Result<(), RepositoryError> {
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_movements")
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn refresh_supplier_performance(&self) -> Result<(), RepositoryError> {
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_supplier_performance")
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn get_stock_summary(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<WarehouseStockSummaryRow>, RepositoryError> {
        if let Some(wh_id) = warehouse_id {
            sqlx::query_as::<_, WarehouseStockSummaryRow>(
                "SELECT * FROM mv_warehouse_stock_summary WHERE warehouse_id = $1",
            )
            .bind(wh_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
        } else {
            sqlx::query_as::<_, WarehouseStockSummaryRow>(
                "SELECT * FROM mv_warehouse_stock_summary ORDER BY warehouse_name ASC",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
        }
    }

    async fn get_daily_movements(
        &self,
        warehouse_id: Option<Uuid>,
        days: i32,
    ) -> Result<Vec<DailyMovementStat>, RepositoryError> {
        if let Some(wh_id) = warehouse_id {
            sqlx::query_as::<_, DailyMovementStat>(
                r#"SELECT * FROM mv_daily_movements
                   WHERE warehouse_id = $1
                     AND movement_date >= CURRENT_DATE - ($2 || ' days')::INTERVAL
                   ORDER BY movement_date DESC, movement_type ASC"#,
            )
            .bind(wh_id)
            .bind(days)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
        } else {
            sqlx::query_as::<_, DailyMovementStat>(
                r#"SELECT * FROM mv_daily_movements
                   WHERE movement_date >= CURRENT_DATE - ($1 || ' days')::INTERVAL
                   ORDER BY movement_date DESC, warehouse_id ASC, movement_type ASC"#,
            )
            .bind(days)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
        }
    }

    async fn get_supplier_performance(
        &self,
        limit: i64,
    ) -> Result<Vec<SupplierPerformanceStat>, RepositoryError> {
        sqlx::query_as::<_, SupplierPerformanceStat>(
            r#"SELECT * FROM mv_supplier_performance
               ORDER BY total_invoices DESC, quality_score ASC
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
