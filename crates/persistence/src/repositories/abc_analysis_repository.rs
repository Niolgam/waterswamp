use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::abc_analysis::*,
    ports::abc_analysis::AbcAnalysisRepositoryPort,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct AbcAnalysisRepository {
    pool: PgPool,
}

impl AbcAnalysisRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AbcAnalysisRepositoryPort for AbcAnalysisRepository {
    async fn get_stock_values(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<(Uuid, Decimal)>, RepositoryError> {
        let rows = if let Some(wh_id) = warehouse_id {
            sqlx::query(
                r#"SELECT catalog_item_id, SUM(quantity * average_unit_value) AS total_value
                   FROM warehouse_stocks
                   WHERE warehouse_id = $1 AND quantity > 0
                   GROUP BY catalog_item_id
                   ORDER BY total_value DESC"#,
            )
            .bind(wh_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
        } else {
            sqlx::query(
                r#"SELECT catalog_item_id, SUM(quantity * average_unit_value) AS total_value
                   FROM warehouse_stocks
                   WHERE quantity > 0
                   GROUP BY catalog_item_id
                   ORDER BY total_value DESC"#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
        };

        let result = rows
            .into_iter()
            .map(|r| {
                let id: Uuid = r.get("catalog_item_id");
                let val: Decimal = r.get("total_value");
                (id, val)
            })
            .collect();

        Ok(result)
    }

    async fn save_results(
        &self,
        run_at: DateTime<Utc>,
        warehouse_id: Option<Uuid>,
        results: Vec<AbcAnalysisResultDto>,
    ) -> Result<(), RepositoryError> {
        if results.is_empty() {
            return Ok(());
        }

        for r in &results {
            sqlx::query(
                r#"INSERT INTO abc_analysis_results
                   (run_at, warehouse_id, catalog_item_id, classification,
                    total_value, cumulative_percentage, rank_position)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            )
            .bind(run_at)
            .bind(warehouse_id)
            .bind(r.catalog_item_id)
            .bind(&r.classification)
            .bind(r.total_value)
            .bind(r.cumulative_percentage)
            .bind(r.rank_position)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        }

        Ok(())
    }

    async fn get_latest_results(
        &self,
        warehouse_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AbcAnalysisResultDto>, i64), RepositoryError> {
        let latest_run = self.get_latest_run_at(warehouse_id).await?;
        let Some(run_at) = latest_run else {
            return Ok((vec![], 0));
        };

        let (count_sql, list_sql) = if warehouse_id.is_some() {
            (
                "SELECT COUNT(*) FROM abc_analysis_results WHERE run_at = $1 AND warehouse_id = $2".to_string(),
                format!(
                    "SELECT * FROM abc_analysis_results WHERE run_at = $1 AND warehouse_id = $2 ORDER BY rank_position ASC LIMIT ${} OFFSET ${}",
                    3, 4
                ),
            )
        } else {
            (
                "SELECT COUNT(*) FROM abc_analysis_results WHERE run_at = $1 AND warehouse_id IS NULL".to_string(),
                format!(
                    "SELECT * FROM abc_analysis_results WHERE run_at = $1 AND warehouse_id IS NULL ORDER BY rank_position ASC LIMIT ${} OFFSET ${}",
                    2, 3
                ),
            )
        };

        let total: i64;
        let rows: Vec<AbcAnalysisResultDto>;

        if let Some(wh_id) = warehouse_id {
            total = sqlx::query(&count_sql)
                .bind(run_at)
                .bind(wh_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?
                .get(0);

            rows = sqlx::query_as::<_, AbcAnalysisResultDto>(&list_sql)
                .bind(run_at)
                .bind(wh_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_error)?;
        } else {
            total = sqlx::query(&count_sql)
                .bind(run_at)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?
                .get(0);

            rows = sqlx::query_as::<_, AbcAnalysisResultDto>(&list_sql)
                .bind(run_at)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_error)?;
        }

        Ok((rows, total))
    }

    async fn get_latest_run_at(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Option<DateTime<Utc>>, RepositoryError> {
        let row = if let Some(wh_id) = warehouse_id {
            sqlx::query(
                "SELECT MAX(run_at) FROM abc_analysis_results WHERE warehouse_id = $1",
            )
            .bind(wh_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
        } else {
            sqlx::query(
                "SELECT MAX(run_at) FROM abc_analysis_results WHERE warehouse_id IS NULL",
            )
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
        };

        Ok(row.get(0))
    }
}
