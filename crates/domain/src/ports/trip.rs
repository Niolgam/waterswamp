use crate::errors::RepositoryError;
use crate::models::trip::*;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[async_trait]
pub trait VehicleTripRepositoryPort: Send + Sync {
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
    ) -> Result<VehicleTripDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleTripDto>, RepositoryError>;

    async fn approve(
        &self,
        id: Uuid,
        approved_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    async fn reject(
        &self,
        id: Uuid,
        rejection_reason: &str,
        rejected_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    /// Transitions APROVADA → ALOCADA.
    ///
    /// Acquires a pessimistic row-level lock on the vehicle row
    /// (`SELECT … FOR UPDATE NOWAIT`) inside a transaction to prevent
    /// concurrent double-booking. Returns `OptimisticLockConflict` if
    /// the lock cannot be acquired immediately or if `version` is stale.
    async fn allocate(
        &self,
        trip_id: Uuid,
        vehicle_id: Uuid,
        driver_id: Uuid,
        allocated_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    /// Check-out: departure — ALOCADA → EM_CURSO (DRS: checkout = saída).
    async fn checkout(
        &self,
        id: Uuid,
        odometer_departure: i64,
        checkout_odometer_id: Option<Uuid>,
        checkout_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    /// Check-in: return — EM_CURSO → AGUARDANDO_PC (DRS: checkin = retorno).
    async fn checkin(
        &self,
        id: Uuid,
        odometer_return: i64,
        checkin_odometer_id: Option<Uuid>,
        checkin_by: Uuid,
        notes: Option<&str>,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    /// Finalize accountability — AGUARDANDO_PC → CONCLUIDA.
    async fn finalize(
        &self,
        id: Uuid,
        finalized_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    /// Flag irrecoverable state conflict — any non-terminal → CONFLITO_MANUAL.
    async fn set_conflict(
        &self,
        id: Uuid,
        reason: &str,
        conflict_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    async fn cancel(
        &self,
        id: Uuid,
        reason: &str,
        cancelled_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    async fn list(
        &self,
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        status: Option<TripStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<VehicleTripDto>, i64), RepositoryError>;
}
