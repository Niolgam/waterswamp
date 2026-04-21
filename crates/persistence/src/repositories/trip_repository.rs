use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::trip::*,
    ports::trip::VehicleTripRepositoryPort,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct VehicleTripRepository {
    pool: PgPool,
}

impl VehicleTripRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleTripRepositoryPort for VehicleTripRepository {
    async fn create(
        &self,
        vehicle_id: Uuid,
        driver_id: Option<Uuid>,
        requester_id: Option<Uuid>,
        destino: &str,
        finalidade: &str,
        passageiros: i32,
        data_saida_prevista: DateTime<Utc>,
        data_retorno_prevista: Option<DateTime<Utc>>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleTripDto, RepositoryError> {
        sqlx::query_as::<_, VehicleTripDto>(
            r#"
            INSERT INTO vehicle_trips
                (vehicle_id, driver_id, requester_id, destino, finalidade,
                 passageiros, data_saida_prevista, data_retorno_prevista,
                 notes, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
            RETURNING *
            "#,
        )
        .bind(vehicle_id)
        .bind(driver_id)
        .bind(requester_id)
        .bind(destino)
        .bind(finalidade)
        .bind(passageiros)
        .bind(data_saida_prevista)
        .bind(data_retorno_prevista)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleTripDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleTripDto>("SELECT * FROM vehicle_trips WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn approve(
        &self,
        id: Uuid,
        aprovado_por: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status      = 'APROVADA',
                aprovado_por = $2,
                aprovado_em  = NOW(),
                version      = version + 1,
                updated_by   = $2,
                updated_at   = NOW()
            WHERE id = $1 AND version = $3 AND status = 'PENDENTE'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(aprovado_por)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn reject(
        &self,
        id: Uuid,
        motivo_rejeicao: &str,
        rejeitado_por: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status         = 'REJEITADA',
                motivo_rejeicao = $2,
                aprovado_por    = $3,
                aprovado_em     = NOW(),
                version         = version + 1,
                updated_by      = $3,
                updated_at      = NOW()
            WHERE id = $1 AND version = $4 AND status = 'PENDENTE'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(motivo_rejeicao)
        .bind(rejeitado_por)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn checkin(
        &self,
        id: Uuid,
        driver_id: Uuid,
        km_saida: i64,
        checkin_odometer_id: Option<Uuid>,
        checkin_por: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status              = 'CHECKIN',
                driver_id           = $2,
                checkin_km          = $3,
                checkin_odometer_id = $4,
                checkin_por         = $5,
                checkin_em          = NOW(),
                version             = version + 1,
                updated_by          = $5,
                updated_at          = NOW()
            WHERE id = $1 AND version = $6 AND status = 'APROVADA'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(driver_id)
        .bind(km_saida)
        .bind(checkin_odometer_id)
        .bind(checkin_por)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn checkout(
        &self,
        id: Uuid,
        km_retorno: i64,
        checkout_odometer_id: Option<Uuid>,
        checkout_por: Uuid,
        notes: Option<&str>,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status               = 'CONCLUIDA',
                checkout_km          = $2,
                checkout_odometer_id = $3,
                checkout_por         = $4,
                checkout_em          = NOW(),
                notes                = COALESCE($5, notes),
                version              = version + 1,
                updated_by           = $4,
                updated_at           = NOW()
            WHERE id = $1 AND version = $6 AND status = 'CHECKIN'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(km_retorno)
        .bind(checkout_odometer_id)
        .bind(checkout_por)
        .bind(notes)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn cancel(
        &self,
        id: Uuid,
        motivo: &str,
        cancelado_por: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status              = 'CANCELADA',
                motivo_cancelamento = $2,
                cancelado_por       = $3,
                cancelado_em        = NOW(),
                version             = version + 1,
                updated_by          = $3,
                updated_at          = NOW()
            WHERE id = $1 AND version = $4 AND status IN ('PENDENTE', 'APROVADA')
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(motivo)
        .bind(cancelado_por)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn list(
        &self,
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        status: Option<TripStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<VehicleTripDto>, i64), RepositoryError> {
        let rows = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            SELECT * FROM vehicle_trips
            WHERE ($1::UUID IS NULL OR vehicle_id = $1)
              AND ($2::UUID IS NULL OR driver_id = $2)
              AND ($3::trip_status_enum IS NULL OR status = $3)
            ORDER BY data_saida_prevista DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(vehicle_id)
        .bind(driver_id)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM vehicle_trips
            WHERE ($1::UUID IS NULL OR vehicle_id = $1)
              AND ($2::UUID IS NULL OR driver_id = $2)
              AND ($3::trip_status_enum IS NULL OR status = $3)
            "#,
        )
        .bind(vehicle_id)
        .bind(driver_id)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((rows, total))
    }
}
