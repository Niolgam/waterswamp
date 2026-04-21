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
        tipo: MaintenanceOrderType,
        titulo: &str,
        descricao: Option<&str>,
        fornecedor_id: Option<Uuid>,
        data_abertura: NaiveDate,
        data_prevista_conclusao: Option<NaiveDate>,
        km_abertura: Option<i64>,
        custo_previsto: Option<Decimal>,
        numero_os_externo: Option<&str>,
        documento_sei: Option<&str>,
        incident_id: Option<Uuid>,
        notas: Option<&str>,
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
        .bind(tipo)
        .bind(titulo)
        .bind(descricao)
        .bind(fornecedor_id)
        .bind(data_abertura)
        .bind(data_prevista_conclusao)
        .bind(km_abertura)
        .bind(custo_previsto)
        .bind(numero_os_externo)
        .bind(documento_sei)
        .bind(incident_id)
        .bind(notas)
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
        custo_real: Option<Decimal>,
        data_conclusao: Option<NaiveDate>,
        notas: Option<&str>,
        motivo_cancelamento: Option<&str>,
        concluido_por: Option<Uuid>,
        cancelado_por: Option<Uuid>,
        version: i32,
    ) -> Result<MaintenanceOrderDto, RepositoryError> {
        let is_concluida = new_status == MaintenanceOrderStatus::Concluida;
        let is_cancelada = new_status == MaintenanceOrderStatus::Cancelada;

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
        .bind(custo_real)
        .bind(data_conclusao)
        .bind(notas)
        .bind(motivo_cancelamento)
        .bind(concluido_por.or(cancelado_por))
        .bind(is_concluida)
        .bind(is_cancelada)
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
        descricao: &str,
        quantidade: Decimal,
        custo_unitario: Option<Decimal>,
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
        .bind(descricao)
        .bind(quantidade)
        .bind(custo_unitario)
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
