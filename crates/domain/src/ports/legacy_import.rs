use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::RepositoryError,
    models::legacy_import::*,
};

#[async_trait]
pub trait LegacyImportRepositoryPort: Send + Sync {
    async fn create_job(
        &self,
        entity_type: ImportEntityType,
        submitted_by: Option<Uuid>,
        total_records: i32,
    ) -> Result<ImportJobDto, RepositoryError>;

    async fn start_job(&self, id: Uuid) -> Result<(), RepositoryError>;

    async fn update_progress(
        &self,
        id: Uuid,
        processed: i32,
        success: i32,
        failed: i32,
        error_log: Option<serde_json::Value>,
    ) -> Result<(), RepositoryError>;

    async fn complete_job(
        &self,
        id: Uuid,
        status: ImportJobStatus,
    ) -> Result<ImportJobDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ImportJobDto>, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ImportJobDto>, i64), RepositoryError>;

    // ── per-entity upsert helpers ─────────────────────────────────────────────

    async fn upsert_supplier(
        &self,
        record: &SupplierImportRecord,
    ) -> Result<(), RepositoryError>;

    async fn upsert_catalog_item(
        &self,
        record: &CatalogItemImportRecord,
    ) -> Result<(), RepositoryError>;

    /// Direct upsert into warehouse_stocks (legacy seed — not a stock movement).
    async fn upsert_initial_stock(
        &self,
        record: &InitialStockImportRecord,
    ) -> Result<(), RepositoryError>;
}
