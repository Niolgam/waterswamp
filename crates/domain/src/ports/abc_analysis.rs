use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    errors::RepositoryError,
    models::abc_analysis::*,
};

#[async_trait]
pub trait AbcAnalysisRepositoryPort: Send + Sync {
    /// Returns (catalog_item_id, total_stock_value) sorted descending for ABC computation.
    async fn get_stock_values(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<(Uuid, Decimal)>, RepositoryError>;

    async fn save_results(
        &self,
        run_at: DateTime<Utc>,
        warehouse_id: Option<Uuid>,
        results: Vec<AbcAnalysisResultDto>,
    ) -> Result<(), RepositoryError>;

    async fn get_latest_results(
        &self,
        warehouse_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AbcAnalysisResultDto>, i64), RepositoryError>;

    async fn get_latest_run_at(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Option<DateTime<Utc>>, RepositoryError>;
}
