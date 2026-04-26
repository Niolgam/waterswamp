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
        destination: &str,
        purpose: &str,
        passengers: i32,
        planned_departure: DateTime<Utc>,
        planned_return: Option<DateTime<Utc>>,
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
        .bind(destination)
        .bind(purpose)
        .bind(passengers)
        .bind(planned_departure)
        .bind(planned_return)
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
        approved_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status       = 'APROVADA',
                aprovado_por = $2,
                aprovado_em  = NOW(),
                version      = version + 1,
                updated_by   = $2,
                updated_at   = NOW()
            WHERE id = $1 AND version = $3 AND status = 'SOLICITADA'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(approved_by)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn reject(
        &self,
        id: Uuid,
        rejection_reason: &str,
        rejected_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status          = 'REJEITADA',
                motivo_rejeicao = $2,
                aprovado_por    = $3,
                aprovado_em     = NOW(),
                version         = version + 1,
                updated_by      = $3,
                updated_at      = NOW()
            WHERE id = $1 AND version = $4 AND status = 'SOLICITADA'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(rejection_reason)
        .bind(rejected_by)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn allocate(
        &self,
        trip_id: Uuid,
        vehicle_id: Uuid,
        driver_id: Uuid,
        allocated_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        // Pessimistic lock: fail immediately if another transaction holds the lock.
        let lock_ok = sqlx::query(
            "SELECT id FROM vehicles WHERE id = $1 FOR UPDATE NOWAIT",
        )
        .bind(vehicle_id)
        .execute(&mut *tx)
        .await;

        if lock_ok.is_err() {
            let _ = tx.rollback().await;
            return Err(RepositoryError::OptimisticLockConflict(
                format!("vehicle:{} is locked by a concurrent allocation", vehicle_id),
            ));
        }

        let update_result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status       = 'ALOCADA',
                driver_id    = $3,
                allocated_at = NOW(),
                allocated_by = $4,
                version      = version + 1,
                updated_by   = $4,
                updated_at   = NOW()
            WHERE id = $1 AND version = $5 AND status = 'APROVADA'
            RETURNING *
            "#,
        )
        .bind(trip_id)
        .bind(vehicle_id)
        .bind(driver_id)
        .bind(allocated_by)
        .bind(version)
        .fetch_optional(&mut *tx)
        .await;

        // Rollback automatically on drop if we return early.
        let result = match update_result {
            Ok(row) => row,
            Err(e) => {
                drop(tx); // triggers rollback
                return Err(map_db_error(e));
            }
        };

        tx.commit().await.map_err(map_db_error)?;

        result.ok_or_else(|| {
            RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", trip_id))
        })
    }

    async fn checkout(
        &self,
        id: Uuid,
        odometer_departure: i64,
        checkout_odometer_id: Option<Uuid>,
        checkout_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status               = 'EM_CURSO',
                checkout_km          = $2,
                checkout_odometer_id = $3,
                checkout_por         = $4,
                checkout_em          = NOW(),
                version              = version + 1,
                updated_by           = $4,
                updated_at           = NOW()
            WHERE id = $1 AND version = $5 AND status = 'ALOCADA'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(odometer_departure)
        .bind(checkout_odometer_id)
        .bind(checkout_by)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn checkin(
        &self,
        id: Uuid,
        odometer_return: i64,
        checkin_odometer_id: Option<Uuid>,
        checkin_by: Uuid,
        notes: Option<&str>,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status              = 'AGUARDANDO_PC',
                checkin_km          = $2,
                checkin_odometer_id = $3,
                checkin_por         = $4,
                checkin_em          = NOW(),
                waiting_pc_at       = NOW(),
                notes               = COALESCE($5, notes),
                version             = version + 1,
                updated_by          = $4,
                updated_at          = NOW()
            WHERE id = $1 AND version = $6 AND status = 'EM_CURSO'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(odometer_return)
        .bind(checkin_odometer_id)
        .bind(checkin_by)
        .bind(notes)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn finalize(
        &self,
        id: Uuid,
        finalized_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status     = 'CONCLUIDA',
                version    = version + 1,
                updated_by = $2,
                updated_at = NOW()
            WHERE id = $1 AND version = $3 AND status = 'AGUARDANDO_PC'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(finalized_by)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn set_conflict(
        &self,
        id: Uuid,
        reason: &str,
        conflict_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError> {
        let result = sqlx::query_as::<_, VehicleTripDto>(
            r#"
            UPDATE vehicle_trips
            SET status          = 'CONFLITO_MANUAL',
                conflict_reason = $2,
                conflict_at     = NOW(),
                conflict_by     = $3,
                version         = version + 1,
                updated_by      = $3,
                updated_at      = NOW()
            WHERE id = $1 AND version = $4
              AND status NOT IN ('CONCLUIDA', 'REJEITADA', 'CANCELADA', 'CONFLITO_MANUAL')
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(conflict_by)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_trip:{}", id)))
    }

    async fn cancel(
        &self,
        id: Uuid,
        reason: &str,
        cancelled_by: Uuid,
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
            WHERE id = $1 AND version = $4
              AND status IN ('SOLICITADA', 'APROVADA', 'ALOCADA')
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(reason)
        .bind(cancelled_by)
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
