use async_trait::async_trait;
use chrono::Utc;
use domain::{
    errors::RepositoryError,
    models::alert::*,
    ports::alert::StockAlertRepositoryPort,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct StockAlertRepository {
    pool: PgPool,
}

impl StockAlertRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockAlertRepositoryPort for StockAlertRepository {
    async fn create(&self, input: CreateStockAlertInput) -> Result<StockAlertDto, RepositoryError> {
        let sla_deadline = input.sla_hours.map(|h| Utc::now() + chrono::Duration::hours(h));

        sqlx::query_as::<_, StockAlertDto>(
            r#"INSERT INTO stock_alerts
               (alert_type, warehouse_id, catalog_item_id, batch_number, requisition_id,
                title, description, severity, sla_deadline, metadata)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#,
        )
        .bind(&input.alert_type)
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .bind(input.batch_number.as_deref())
        .bind(input.requisition_id)
        .bind(&input.title)
        .bind(input.description.as_deref())
        .bind(&input.severity)
        .bind(sla_deadline)
        .bind(input.metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<StockAlertDto>, RepositoryError> {
        sqlx::query_as::<_, StockAlertDto>("SELECT * FROM stock_alerts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn list(
        &self,
        warehouse_id: Option<Uuid>,
        status: Option<StockAlertStatus>,
        alert_type: Option<StockAlertType>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<StockAlertDto>, i64), RepositoryError> {
        let mut conditions = vec!["1=1".to_string()];
        let mut param = 1usize;

        if warehouse_id.is_some() {
            conditions.push(format!("warehouse_id = ${}", param));
            param += 1;
        }
        if status.is_some() {
            conditions.push(format!("status = ${}", param));
            param += 1;
        }
        if alert_type.is_some() {
            conditions.push(format!("alert_type = ${}", param));
            param += 1;
        }

        let where_clause = conditions.join(" AND ");
        let count_sql = format!("SELECT COUNT(*) FROM stock_alerts WHERE {}", where_clause);
        let list_sql = format!(
            "SELECT * FROM stock_alerts WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param, param + 1
        );

        let mut count_q = sqlx::query(&count_sql);
        let mut list_q = sqlx::query_as::<_, StockAlertDto>(&list_sql);

        if let Some(v) = warehouse_id {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }
        if let Some(ref v) = status {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }
        if let Some(ref v) = alert_type {
            count_q = count_q.bind(v);
            list_q = list_q.bind(v);
        }

        count_q = count_q.bind(limit).bind(offset);
        list_q = list_q.bind(limit).bind(offset);

        let total: i64 = count_q
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get(0);

        let rows = list_q.fetch_all(&self.pool).await.map_err(map_db_error)?;

        Ok((rows, total))
    }

    async fn acknowledge(
        &self,
        id: Uuid,
        acknowledged_by: Uuid,
    ) -> Result<StockAlertDto, RepositoryError> {
        sqlx::query_as::<_, StockAlertDto>(
            r#"UPDATE stock_alerts SET
               status = 'ACKNOWLEDGED',
               acknowledged_at = NOW(),
               acknowledged_by = $2,
               updated_at = NOW()
               WHERE id = $1 AND status = 'OPEN'
               RETURNING *"#,
        )
        .bind(id)
        .bind(acknowledged_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn resolve(
        &self,
        id: Uuid,
        resolved_by: Uuid,
    ) -> Result<StockAlertDto, RepositoryError> {
        sqlx::query_as::<_, StockAlertDto>(
            r#"UPDATE stock_alerts SET
               status = 'RESOLVED',
               resolved_at = NOW(),
               resolved_by = $2,
               updated_at = NOW()
               WHERE id = $1 AND status IN ('OPEN', 'ACKNOWLEDGED', 'SLA_BREACHED')
               RETURNING *"#,
        )
        .bind(id)
        .bind(resolved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn mark_sla_breached(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"UPDATE stock_alerts SET
               status = 'SLA_BREACHED',
               sla_breached_at = NOW(),
               updated_at = NOW()
               WHERE id = $1 AND status IN ('OPEN', 'ACKNOWLEDGED')"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn find_overdue_sla(&self) -> Result<Vec<StockAlertDto>, RepositoryError> {
        sqlx::query_as::<_, StockAlertDto>(
            r#"SELECT * FROM stock_alerts
               WHERE status IN ('OPEN', 'ACKNOWLEDGED')
                 AND sla_deadline IS NOT NULL
                 AND sla_deadline < NOW()
               ORDER BY sla_deadline ASC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
