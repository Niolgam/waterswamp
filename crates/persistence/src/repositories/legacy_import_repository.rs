use async_trait::async_trait;
use chrono::NaiveDate;
use domain::{
    errors::RepositoryError,
    models::legacy_import::*,
    ports::legacy_import::LegacyImportRepositoryPort,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct LegacyImportRepository {
    pool: PgPool,
}

impl LegacyImportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LegacyImportRepositoryPort for LegacyImportRepository {
    async fn create_job(
        &self,
        entity_type: ImportEntityType,
        submitted_by: Option<Uuid>,
        total_records: i32,
    ) -> Result<ImportJobDto, RepositoryError> {
        sqlx::query_as::<_, ImportJobDto>(
            r#"INSERT INTO legacy_import_jobs
               (entity_type, submitted_by, total_records)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(&entity_type)
        .bind(submitted_by)
        .bind(total_records)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn start_job(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE legacy_import_jobs SET status = 'RUNNING', started_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn update_progress(
        &self,
        id: Uuid,
        processed: i32,
        success: i32,
        failed: i32,
        error_log: Option<serde_json::Value>,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"UPDATE legacy_import_jobs SET
               processed_records = $2,
               success_records = $3,
               failed_records = $4,
               error_log = $5,
               updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(processed)
        .bind(success)
        .bind(failed)
        .bind(error_log)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn complete_job(
        &self,
        id: Uuid,
        status: ImportJobStatus,
    ) -> Result<ImportJobDto, RepositoryError> {
        sqlx::query_as::<_, ImportJobDto>(
            r#"UPDATE legacy_import_jobs SET
               status = $2,
               completed_at = NOW(),
               updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ImportJobDto>, RepositoryError> {
        sqlx::query_as::<_, ImportJobDto>(
            "SELECT * FROM legacy_import_jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ImportJobDto>, i64), RepositoryError> {
        let total: i64 = sqlx::query("SELECT COUNT(*) FROM legacy_import_jobs")
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get(0);

        let rows = sqlx::query_as::<_, ImportJobDto>(
            "SELECT * FROM legacy_import_jobs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((rows, total))
    }

    async fn upsert_supplier(
        &self,
        record: &SupplierImportRecord,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"INSERT INTO suppliers (document_number, legal_name, trade_name, email, phone, address)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (document_number) DO UPDATE SET
                 legal_name  = EXCLUDED.legal_name,
                 trade_name  = COALESCE(EXCLUDED.trade_name, suppliers.trade_name),
                 email       = COALESCE(EXCLUDED.email, suppliers.email),
                 phone       = COALESCE(EXCLUDED.phone, suppliers.phone),
                 address     = COALESCE(EXCLUDED.address, suppliers.address),
                 updated_at  = NOW()"#,
        )
        .bind(&record.document_number)
        .bind(&record.legal_name)
        .bind(record.trade_name.as_deref())
        .bind(record.email.as_deref())
        .bind(record.phone.as_deref())
        .bind(record.address.as_deref())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn upsert_catalog_item(
        &self,
        record: &CatalogItemImportRecord,
    ) -> Result<(), RepositoryError> {
        // Resolve unit_of_measure_id by abbreviation (best-effort; skip if not found)
        let unit_row = sqlx::query(
            "SELECT id FROM units_of_measure WHERE UPPER(abbreviation) = UPPER($1) LIMIT 1",
        )
        .bind(&record.unit_abbreviation)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        let unit_id: Option<Uuid> = unit_row.map(|r| r.get("id"));
        let Some(unit_id) = unit_id else {
            // Unit not found — cannot import without a valid UoM FK
            return Err(RepositoryError::InvalidData(format!(
                "Unit of measure '{}' not found",
                record.unit_abbreviation
            )));
        };

        // Resolve or create a default PDM for this item (use code as pdm identifier)
        // Look for an existing PDM keyed by the item code in its description
        let pdm_row = sqlx::query(
            r#"SELECT id FROM catmat_pdms
               WHERE description = $1
               LIMIT 1"#,
        )
        .bind(format!("LEGADO:{}", record.code))
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        let pdm_id: Uuid = if let Some(row) = pdm_row {
            row.get("id")
        } else {
            // Create a placeholder PDM — requires catmat_class_id; use a sentinel class
            // (This import is best-effort: real cataloguing should be done manually)
            let class_row = sqlx::query("SELECT id FROM catmat_classes LIMIT 1")
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_error)?;

            let Some(class_row) = class_row else {
                return Err(RepositoryError::InvalidData(
                    "No catmat_class exists to create a placeholder PDM for legacy import".into(),
                ));
            };
            let class_id: Uuid = class_row.get("id");

            sqlx::query(
                r#"INSERT INTO catmat_pdms (catmat_class_id, description)
                   VALUES ($1, $2)
                   RETURNING id"#,
            )
            .bind(class_id)
            .bind(format!("LEGADO:{}", record.code))
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get("id")
        };

        sqlx::query(
            r#"INSERT INTO catmat_items (pdm_id, unit_of_measure_id, code, description)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (code) DO UPDATE SET
                 description        = EXCLUDED.description,
                 unit_of_measure_id = EXCLUDED.unit_of_measure_id,
                 updated_at         = NOW()"#,
        )
        .bind(pdm_id)
        .bind(unit_id)
        .bind(&record.code)
        .bind(&record.description)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    async fn upsert_initial_stock(
        &self,
        record: &InitialStockImportRecord,
    ) -> Result<(), RepositoryError> {
        let warehouse_row = sqlx::query(
            "SELECT id FROM warehouses WHERE code = $1 AND is_active = TRUE LIMIT 1",
        )
        .bind(&record.warehouse_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        let Some(wh_row) = warehouse_row else {
            return Err(RepositoryError::InvalidData(format!(
                "Warehouse '{}' not found",
                record.warehouse_code
            )));
        };
        let warehouse_id: Uuid = wh_row.get("id");

        let item_row = sqlx::query(
            "SELECT id FROM catmat_items WHERE code = $1 AND is_active = TRUE LIMIT 1",
        )
        .bind(&record.catalog_item_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        let Some(item_row) = item_row else {
            return Err(RepositoryError::InvalidData(format!(
                "Catalog item '{}' not found",
                record.catalog_item_code
            )));
        };
        let catalog_item_id: Uuid = item_row.get("id");

        // Direct upsert on warehouse_stocks — legacy seed, not a tracked movement
        sqlx::query(
            r#"INSERT INTO warehouse_stocks (warehouse_id, catalog_item_id, quantity, average_unit_value)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (warehouse_id, catalog_item_id) DO UPDATE SET
                 quantity            = EXCLUDED.quantity,
                 average_unit_value  = EXCLUDED.average_unit_value,
                 updated_at          = NOW()"#,
        )
        .bind(warehouse_id)
        .bind(catalog_item_id)
        .bind(record.quantity)
        .bind(record.unit_cost)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        // Optionally upsert batch stock if batch_number provided
        if let Some(ref bn) = record.batch_number {
            let exp_date: Option<NaiveDate> = record
                .expiration_date
                .as_deref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

            sqlx::query(
                r#"INSERT INTO warehouse_batch_stocks
                   (warehouse_id, catalog_item_id, batch_number, expiration_date, quantity, unit_cost)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (warehouse_id, catalog_item_id, batch_number) DO UPDATE SET
                     quantity        = EXCLUDED.quantity,
                     unit_cost       = EXCLUDED.unit_cost,
                     expiration_date = COALESCE(EXCLUDED.expiration_date, warehouse_batch_stocks.expiration_date),
                     updated_at      = NOW()"#,
            )
            .bind(warehouse_id)
            .bind(catalog_item_id)
            .bind(bn)
            .bind(exp_date)
            .bind(record.quantity)
            .bind(record.unit_cost)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        }

        Ok(())
    }
}
