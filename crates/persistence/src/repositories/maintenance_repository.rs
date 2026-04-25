use async_trait::async_trait;
use chrono::NaiveDate;
use domain::{
    errors::RepositoryError,
    models::maintenance::*,
    ports::maintenance::MaintenanceOrderRepositoryPort,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct MaintenanceOrderRepository {
    pool: PgPool,
}

impl MaintenanceOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaintenanceOrderRepositoryPort for MaintenanceOrderRepository {
    async fn create(
        &self,
        vehicle_id: Uuid,
        order_type: MaintenanceOrderType,
        title: &str,
        description: Option<&str>,
        supplier_id: Option<Uuid>,
        opened_date: NaiveDate,
        expected_completion_date: Option<NaiveDate>,
        odometer_at_opening: Option<i64>,
        estimated_cost: Option<Decimal>,
        external_order_number: Option<&str>,
        documento_sei: Option<&str>,
        incident_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, RepositoryError> {
        sqlx::query_as::<_, MaintenanceOrderDto>(
            r#"
            INSERT INTO vehicle_maintenance_orders
                (vehicle_id, tipo, titulo, descricao, fornecedor_id,
                 data_abertura, data_prevista_conclusao, km_abertura, custo_previsto,
                 numero_os_externo, documento_sei, incident_id, notas,
                 created_by, updated_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$14)
            RETURNING *
            "#,
        )
        .bind(vehicle_id)
        .bind(order_type)
        .bind(title)
        .bind(description)
        .bind(supplier_id)
        .bind(opened_date)
        .bind(expected_completion_date)
        .bind(odometer_at_opening)
        .bind(estimated_cost)
        .bind(external_order_number)
        .bind(documento_sei)
        .bind(incident_id)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaintenanceOrderDto>, RepositoryError> {
        sqlx::query_as::<_, MaintenanceOrderDto>(
            "SELECT * FROM vehicle_maintenance_orders WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn advance_status(
        &self,
        id: Uuid,
        new_status: MaintenanceOrderStatus,
        actual_cost: Option<Decimal>,
        completion_date: Option<NaiveDate>,
        notes: Option<&str>,
        cancellation_reason: Option<&str>,
        completed_by: Option<Uuid>,
        cancelled_by: Option<Uuid>,
        version: i32,
    ) -> Result<MaintenanceOrderDto, RepositoryError> {
        let is_completed = new_status == MaintenanceOrderStatus::Completed;
        let is_cancelled = new_status == MaintenanceOrderStatus::Cancelled;

        let result = sqlx::query_as::<_, MaintenanceOrderDto>(
            r#"
            UPDATE vehicle_maintenance_orders
            SET status              = $2,
                custo_real          = COALESCE($3, custo_real),
                data_conclusao      = CASE WHEN $8 THEN COALESCE($4, CURRENT_DATE) ELSE data_conclusao END,
                notas               = COALESCE($5, notas),
                motivo_cancelamento = COALESCE($6, motivo_cancelamento),
                concluido_por       = CASE WHEN $8 THEN $7 ELSE concluido_por END,
                cancelado_por       = CASE WHEN $9 THEN $7 ELSE cancelado_por END,
                cancelado_em        = CASE WHEN $9 THEN NOW() ELSE cancelado_em END,
                version             = version + 1,
                updated_by          = $7,
                updated_at          = NOW()
            WHERE id = $1 AND version = $10
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(new_status)
        .bind(actual_cost)
        .bind(completion_date)
        .bind(notes)
        .bind(cancellation_reason)
        .bind(completed_by.or(cancelled_by))
        .bind(is_completed)
        .bind(is_cancelled)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("maintenance_order:{}", id)))
    }

    async fn list(
        &self,
        vehicle_id: Option<Uuid>,
        status: Option<MaintenanceOrderStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MaintenanceOrderDto>, i64), RepositoryError> {
        let rows = sqlx::query_as::<_, MaintenanceOrderDto>(
            r#"
            SELECT * FROM vehicle_maintenance_orders
            WHERE ($1::UUID IS NULL OR vehicle_id = $1)
              AND ($2::maintenance_order_status_enum IS NULL OR status = $2)
            ORDER BY data_abertura DESC, created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(vehicle_id)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM vehicle_maintenance_orders
            WHERE ($1::UUID IS NULL OR vehicle_id = $1)
              AND ($2::maintenance_order_status_enum IS NULL OR status = $2)
            "#,
        )
        .bind(vehicle_id)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((rows, total))
    }

    async fn cost_summary(
        &self,
        vehicle_id: Uuid,
    ) -> Result<MaintenanceCostSummaryDto, RepositoryError> {
        sqlx::query_as::<_, MaintenanceCostSummaryDto>(
            r#"
            SELECT
                $1::UUID                                AS vehicle_id,
                COUNT(*)                                AS total_os,
                COUNT(*) FILTER (WHERE status = 'CONCLUIDA') AS os_concluidas,
                SUM(custo_real)                         AS custo_total_real,
                SUM(custo_previsto)                     AS custo_total_previsto
            FROM vehicle_maintenance_orders
            WHERE vehicle_id = $1
            "#,
        )
        .bind(vehicle_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn add_item(
        &self,
        order_id: Uuid,
        service_id: Option<Uuid>,
        description: &str,
        quantity: Decimal,
        unit_cost: Option<Decimal>,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderItemDto, RepositoryError> {
        sqlx::query_as::<_, MaintenanceOrderItemDto>(
            r#"
            INSERT INTO vehicle_maintenance_order_items
                (order_id, service_id, descricao, quantidade, custo_unitario, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(order_id)
        .bind(service_id)
        .bind(description)
        .bind(quantity)
        .bind(unit_cost)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_items(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<MaintenanceOrderItemDto>, RepositoryError> {
        sqlx::query_as::<_, MaintenanceOrderItemDto>(
            "SELECT * FROM vehicle_maintenance_order_items WHERE order_id = $1 ORDER BY created_at",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
