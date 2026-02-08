use crate::errors::RepositoryError;
use crate::models::vehicle::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

// ============================
// Vehicle Category Repository Port
// ============================

#[async_trait]
pub trait VehicleCategoryRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleCategoryDto>, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, name: &str, description: Option<&str>, is_active: bool) -> Result<VehicleCategoryDto, RepositoryError>;
    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>, is_active: Option<bool>) -> Result<VehicleCategoryDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleCategoryDto>, i64), RepositoryError>;
}

// ============================
// Vehicle Make Repository Port
// ============================

#[async_trait]
pub trait VehicleMakeRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleMakeDto>, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, name: &str) -> Result<VehicleMakeDto, RepositoryError>;
    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleMakeDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleMakeDto>, i64), RepositoryError>;
}

// ============================
// Vehicle Model Repository Port
// ============================

#[async_trait]
pub trait VehicleModelRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleModelDto>, RepositoryError>;
    async fn exists_by_name_in_make(&self, name: &str, make_id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_name_in_make_excluding(&self, name: &str, make_id: Uuid, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, make_id: Uuid, name: &str) -> Result<VehicleModelDto, RepositoryError>;
    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleModelDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>, make_id: Option<Uuid>) -> Result<(Vec<VehicleModelDto>, i64), RepositoryError>;
}

// ============================
// Vehicle Color Repository Port
// ============================

#[async_trait]
pub trait VehicleColorRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleColorDto>, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, name: &str, hex_code: Option<&str>) -> Result<VehicleColorDto, RepositoryError>;
    async fn update(&self, id: Uuid, name: Option<&str>, hex_code: Option<&str>, is_active: Option<bool>) -> Result<VehicleColorDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleColorDto>, i64), RepositoryError>;
}

// ============================
// Vehicle Fuel Type Repository Port
// ============================

#[async_trait]
pub trait VehicleFuelTypeRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleFuelTypeDto>, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, name: &str) -> Result<VehicleFuelTypeDto, RepositoryError>;
    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleFuelTypeDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleFuelTypeDto>, i64), RepositoryError>;
}

// ============================
// Vehicle Transmission Type Repository Port
// ============================

#[async_trait]
pub trait VehicleTransmissionTypeRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleTransmissionTypeDto>, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, name: &str) -> Result<VehicleTransmissionTypeDto, RepositoryError>;
    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleTransmissionTypeDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleTransmissionTypeDto>, i64), RepositoryError>;
}

// ============================
// Vehicle Repository Port
// ============================

#[async_trait]
pub trait VehicleRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<VehicleWithDetailsDto>, RepositoryError>;
    async fn exists_by_license_plate(&self, plate: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_license_plate_excluding(&self, plate: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_chassis(&self, chassis: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_chassis_excluding(&self, chassis: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_renavam(&self, renavam: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_renavam_excluding(&self, renavam: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;

    async fn create(
        &self,
        license_plate: &str,
        chassis_number: &str,
        renavam: &str,
        engine_number: Option<&str>,
        category_id: Uuid,
        make_id: Uuid,
        model_id: Uuid,
        color_id: Uuid,
        fuel_type_id: Uuid,
        transmission_type_id: Option<Uuid>,
        manufacture_year: i32,
        model_year: i32,
        passenger_capacity: Option<i32>,
        load_capacity_kg: Option<Decimal>,
        engine_displacement: Option<i32>,
        horsepower: Option<i32>,
        acquisition_type: AcquisitionType,
        acquisition_date: Option<NaiveDate>,
        purchase_value: Option<Decimal>,
        patrimony_number: Option<&str>,
        department_id: Option<Uuid>,
        status: VehicleStatus,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        license_plate: Option<&str>,
        chassis_number: Option<&str>,
        renavam: Option<&str>,
        engine_number: Option<&str>,
        category_id: Option<Uuid>,
        make_id: Option<Uuid>,
        model_id: Option<Uuid>,
        color_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        transmission_type_id: Option<Uuid>,
        manufacture_year: Option<i32>,
        model_year: Option<i32>,
        passenger_capacity: Option<i32>,
        load_capacity_kg: Option<Decimal>,
        engine_displacement: Option<i32>,
        horsepower: Option<i32>,
        acquisition_type: Option<AcquisitionType>,
        acquisition_date: Option<NaiveDate>,
        purchase_value: Option<Decimal>,
        patrimony_number: Option<&str>,
        department_id: Option<Uuid>,
        status: Option<VehicleStatus>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleDto, RepositoryError>;

    async fn soft_delete(&self, id: Uuid, deleted_by: Option<Uuid>) -> Result<bool, RepositoryError>;
    async fn restore(&self, id: Uuid) -> Result<bool, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        status: Option<VehicleStatus>,
        category_id: Option<Uuid>,
        make_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        department_id: Option<Uuid>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleWithDetailsDto>, i64), RepositoryError>;

    async fn search_autocomplete(&self, query: &str, limit: i64) -> Result<Vec<VehicleDto>, RepositoryError>;
}

// ============================
// Vehicle Document Repository Port
// ============================

#[async_trait]
pub trait VehicleDocumentRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleDocumentDto>, RepositoryError>;
    async fn create(
        &self,
        vehicle_id: Uuid,
        document_type: DocumentType,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        description: Option<&str>,
        uploaded_by: Option<Uuid>,
    ) -> Result<VehicleDocumentDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list_by_vehicle(&self, vehicle_id: Uuid) -> Result<Vec<VehicleDocumentDto>, RepositoryError>;
}

// ============================
// Vehicle Status History Repository Port
// ============================

#[async_trait]
pub trait VehicleStatusHistoryRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_id: Uuid,
        old_status: Option<VehicleStatus>,
        new_status: VehicleStatus,
        reason: Option<&str>,
        changed_by: Option<Uuid>,
    ) -> Result<VehicleStatusHistoryDto, RepositoryError>;
    async fn list_by_vehicle(&self, vehicle_id: Uuid) -> Result<Vec<VehicleStatusHistoryDto>, RepositoryError>;
}
