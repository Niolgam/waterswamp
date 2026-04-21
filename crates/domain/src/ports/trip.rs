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

    async fn checkin(
        &self,
        id: Uuid,
        driver_id: Uuid,
        odometer_departure: i64,
        checkin_odometer_id: Option<Uuid>,
        checkin_by: Uuid,
        version: i32,
    ) -> Result<VehicleTripDto, RepositoryError>;

    async fn checkout(
        &self,
        id: Uuid,
        odometer_return: i64,
        checkout_odometer_id: Option<Uuid>,
        checkout_by: Uuid,
        notes: Option<&str>,
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
