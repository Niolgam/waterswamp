use crate::errors::RepositoryError;
use crate::models::batch::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait WarehouseBatchStockRepositoryPort: Send + Sync {
    /// Upsert batch stock entry after a movement.
    async fn upsert(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        batch_number: &str,
        expiration_date: Option<NaiveDate>,
        quantity_delta: Decimal, // positive = entry, negative = exit
        unit_cost: Decimal,
    ) -> Result<WarehouseBatchStockDto, RepositoryError>;

    /// List batches for an item in FEFO order (earliest expiration first, quarantined last).
    async fn list_fefo(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
    ) -> Result<Vec<WarehouseBatchStockDto>, RepositoryError>;

    /// Get a specific batch stock entry.
    async fn find(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        batch_number: &str,
    ) -> Result<Option<WarehouseBatchStockDto>, RepositoryError>;

    /// Set quarantine status on a batch.
    async fn set_quarantine(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        batch_number: &str,
        quarantined: bool,
        reason: Option<&str>,
        quarantined_by: Option<Uuid>,
    ) -> Result<(), RepositoryError>;

    /// List all batches nearing expiry (expiration_date <= today + days).
    async fn list_near_expiry(
        &self,
        warehouse_id: Option<Uuid>,
        days_ahead: i32,
    ) -> Result<Vec<WarehouseBatchStockDto>, RepositoryError>;
}

#[async_trait]
pub trait BatchQualityOccurrenceRepositoryPort: Send + Sync {
    async fn create(
        &self,
        payload: &CreateBatchQualityOccurrencePayload,
        reported_by: Uuid,
        quarantine_triggered: bool,
    ) -> Result<BatchQualityOccurrenceDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BatchQualityOccurrenceDto>, RepositoryError>;

    async fn list(
        &self,
        warehouse_id: Option<Uuid>,
        catalog_item_id: Option<Uuid>,
        batch_number: Option<&str>,
        status: Option<BatchOccurrenceStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<BatchQualityOccurrenceDto>, i64), RepositoryError>;

    async fn resolve(
        &self,
        id: Uuid,
        payload: &ResolveOccurrencePayload,
        resolved_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, RepositoryError>;

    async fn close(
        &self,
        id: Uuid,
        payload: &CloseOccurrencePayload,
        closed_by: Uuid,
    ) -> Result<BatchQualityOccurrenceDto, RepositoryError>;
}
