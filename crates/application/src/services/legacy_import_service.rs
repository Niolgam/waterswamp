use std::sync::Arc;

use domain::{
    models::legacy_import::*,
    ports::legacy_import::LegacyImportRepositoryPort,
};
use uuid::Uuid;

use crate::errors::ServiceError;

pub struct LegacyImportService {
    repo: Arc<dyn LegacyImportRepositoryPort>,
}

impl LegacyImportService {
    pub fn new(repo: Arc<dyn LegacyImportRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn import_suppliers(
        &self,
        payload: ImportSuppliersPayload,
        submitted_by: Option<Uuid>,
    ) -> Result<ImportJobDto, ServiceError> {
        if payload.records.is_empty() {
            return Err(ServiceError::BadRequest("No records to import".into()));
        }

        let total = payload.records.len() as i32;
        let job = self
            .repo
            .create_job(ImportEntityType::Supplier, submitted_by, total)
            .await
            .map_err(ServiceError::from)?;

        self.repo.start_job(job.id).await.map_err(ServiceError::from)?;

        let mut success = 0i32;
        let mut failed = 0i32;
        let mut errors: Vec<serde_json::Value> = vec![];

        for (i, record) in payload.records.iter().enumerate() {
            match self.repo.upsert_supplier(record).await {
                Ok(_) => success += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({
                        "index": i,
                        "document_number": record.document_number,
                        "error": e.to_string()
                    }));
                }
            }
        }

        let error_log = if errors.is_empty() {
            None
        } else {
            Some(serde_json::Value::Array(errors))
        };

        self.repo
            .update_progress(job.id, total, success, failed, error_log)
            .await
            .map_err(ServiceError::from)?;

        let final_status = if failed == 0 {
            ImportJobStatus::Completed
        } else if success == 0 {
            ImportJobStatus::Failed
        } else {
            ImportJobStatus::Partial
        };

        self.repo
            .complete_job(job.id, final_status)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn import_catalog_items(
        &self,
        payload: ImportCatalogItemsPayload,
        submitted_by: Option<Uuid>,
    ) -> Result<ImportJobDto, ServiceError> {
        if payload.records.is_empty() {
            return Err(ServiceError::BadRequest("No records to import".into()));
        }

        let total = payload.records.len() as i32;
        let job = self
            .repo
            .create_job(ImportEntityType::CatalogItem, submitted_by, total)
            .await
            .map_err(ServiceError::from)?;

        self.repo.start_job(job.id).await.map_err(ServiceError::from)?;

        let mut success = 0i32;
        let mut failed = 0i32;
        let mut errors: Vec<serde_json::Value> = vec![];

        for (i, record) in payload.records.iter().enumerate() {
            match self.repo.upsert_catalog_item(record).await {
                Ok(_) => success += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({
                        "index": i,
                        "code": record.code,
                        "error": e.to_string()
                    }));
                }
            }
        }

        let error_log = if errors.is_empty() {
            None
        } else {
            Some(serde_json::Value::Array(errors))
        };

        self.repo
            .update_progress(job.id, total, success, failed, error_log)
            .await
            .map_err(ServiceError::from)?;

        let final_status = if failed == 0 {
            ImportJobStatus::Completed
        } else if success == 0 {
            ImportJobStatus::Failed
        } else {
            ImportJobStatus::Partial
        };

        self.repo
            .complete_job(job.id, final_status)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn import_initial_stock(
        &self,
        payload: ImportInitialStockPayload,
        submitted_by: Option<Uuid>,
    ) -> Result<ImportJobDto, ServiceError> {
        if payload.records.is_empty() {
            return Err(ServiceError::BadRequest("No records to import".into()));
        }

        let total = payload.records.len() as i32;
        let job = self
            .repo
            .create_job(ImportEntityType::InitialStock, submitted_by, total)
            .await
            .map_err(ServiceError::from)?;

        self.repo.start_job(job.id).await.map_err(ServiceError::from)?;

        let mut success = 0i32;
        let mut failed = 0i32;
        let mut errors: Vec<serde_json::Value> = vec![];

        for (i, record) in payload.records.iter().enumerate() {
            if record.quantity < rust_decimal::Decimal::ZERO {
                failed += 1;
                errors.push(serde_json::json!({
                    "index": i,
                    "warehouse_code": record.warehouse_code,
                    "catalog_item_code": record.catalog_item_code,
                    "error": "quantity must be >= 0"
                }));
                continue;
            }
            match self.repo.upsert_initial_stock(record).await {
                Ok(_) => success += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({
                        "index": i,
                        "warehouse_code": record.warehouse_code,
                        "catalog_item_code": record.catalog_item_code,
                        "error": e.to_string()
                    }));
                }
            }
        }

        let error_log = if errors.is_empty() {
            None
        } else {
            Some(serde_json::Value::Array(errors))
        };

        self.repo
            .update_progress(job.id, total, success, failed, error_log)
            .await
            .map_err(ServiceError::from)?;

        let final_status = if failed == 0 {
            ImportJobStatus::Completed
        } else if success == 0 {
            ImportJobStatus::Failed
        } else {
            ImportJobStatus::Partial
        };

        self.repo
            .complete_job(job.id, final_status)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_job(&self, id: Uuid) -> Result<ImportJobDto, ServiceError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound(format!("Import job {} not found", id)))
    }

    pub async fn list_jobs(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ImportJobDto>, i64), ServiceError> {
        self.repo
            .list(limit, offset)
            .await
            .map_err(ServiceError::from)
    }
}
