use crate::errors::RepositoryError;
use crate::models::vehicle_fine::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait VehicleFineTypeRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleFineTypeDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(
        &self,
        code: &str,
        description: &str,
        severity: &FineSeverity,
        points: i32,
        fine_amount: Decimal,
        created_by: Option<Uuid>,
    ) -> Result<VehicleFineTypeDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        code: Option<&str>,
        description: Option<&str>,
        severity: Option<&FineSeverity>,
        points: Option<i32>,
        fine_amount: Option<Decimal>,
        is_active: Option<bool>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineTypeDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        severity: Option<FineSeverity>,
        is_active: Option<bool>,
    ) -> Result<(Vec<VehicleFineTypeDto>, i64), RepositoryError>;
}

#[async_trait]
pub trait VehicleFineRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleFineDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<VehicleFineWithDetailsDto>, RepositoryError>;
    async fn create(
        &self,
        vehicle_id: Uuid,
        fine_type_id: Uuid,
        supplier_id: Uuid,
        driver_id: Option<Uuid>,
        auto_number: Option<&str>,
        fine_date: DateTime<Utc>,
        notification_date: Option<DateTime<Utc>>,
        due_date: DateTime<Utc>,
        location: Option<&str>,
        sei_process_number: Option<&str>,
        fine_amount: Decimal,
        discount_amount: Option<Decimal>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleFineDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        vehicle_id: Option<Uuid>,
        fine_type_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        auto_number: Option<&str>,
        fine_date: Option<DateTime<Utc>>,
        notification_date: Option<DateTime<Utc>>,
        due_date: Option<DateTime<Utc>>,
        location: Option<&str>,
        sei_process_number: Option<&str>,
        fine_amount: Option<Decimal>,
        discount_amount: Option<Decimal>,
        paid_amount: Option<Decimal>,
        payment_date: Option<DateTime<Utc>>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineDto, RepositoryError>;
    async fn update_status(
        &self,
        id: Uuid,
        status: &FineStatus,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineDto, RepositoryError>;
    async fn soft_delete(&self, id: Uuid, deleted_by: Option<Uuid>) -> Result<bool, RepositoryError>;
    async fn restore(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        vehicle_id: Option<Uuid>,
        fine_type_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        status: Option<FineStatus>,
        search: Option<String>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleFineWithDetailsDto>, i64), RepositoryError>;
}

#[async_trait]
pub trait VehicleFineStatusHistoryRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_fine_id: Uuid,
        old_status: Option<FineStatus>,
        new_status: FineStatus,
        reason: Option<&str>,
        changed_by: Option<Uuid>,
    ) -> Result<VehicleFineStatusHistoryDto, RepositoryError>;
    async fn list_by_fine(&self, vehicle_fine_id: Uuid) -> Result<Vec<VehicleFineStatusHistoryDto>, RepositoryError>;
}
