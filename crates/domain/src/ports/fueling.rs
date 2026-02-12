use crate::errors::RepositoryError;
use crate::models::fueling::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait FuelingRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FuelingDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<FuelingWithDetailsDto>, RepositoryError>;
    async fn create(
        &self,
        vehicle_id: Uuid,
        driver_id: Uuid,
        supplier_id: Option<Uuid>,
        fuel_type_id: Uuid,
        fueling_date: DateTime<Utc>,
        odometer_km: i32,
        quantity_liters: Decimal,
        unit_price: Decimal,
        total_cost: Decimal,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FuelingDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        fueling_date: Option<DateTime<Utc>>,
        odometer_km: Option<i32>,
        quantity_liters: Option<Decimal>,
        unit_price: Option<Decimal>,
        total_cost: Option<Decimal>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FuelingDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
    ) -> Result<(Vec<FuelingWithDetailsDto>, i64), RepositoryError>;
}
