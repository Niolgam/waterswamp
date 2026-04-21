use crate::errors::RepositoryError;
use crate::models::asset_management::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

// ============================
// Vehicle Department Transfer Port
// ============================

#[async_trait]
pub trait VehicleDepartmentTransferRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_id: Uuid,
        dept_origem_id: Option<Uuid>,
        dept_destino_id: Uuid,
        data_efetiva: NaiveDate,
        motivo: &str,
        documento_sei: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDepartmentTransferDto, RepositoryError>;

    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
    ) -> Result<Vec<VehicleDepartmentTransferDto>, RepositoryError>;
}

// ============================
// Depreciation Config Port
// ============================

#[async_trait]
pub trait DepreciationConfigRepositoryPort: Send + Sync {
    async fn find_by_category(
        &self,
        vehicle_category_id: Uuid,
    ) -> Result<Option<DepreciationConfigDto>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DepreciationConfigDto>, RepositoryError>;

    async fn upsert(
        &self,
        vehicle_category_id: Uuid,
        useful_life_years: Decimal,
        residual_value_min: Decimal,
        is_active: bool,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<DepreciationConfigDto, RepositoryError>;

    async fn list(&self) -> Result<Vec<DepreciationConfigDto>, RepositoryError>;
}

// ============================
// Vehicle Incident Port
// ============================

#[async_trait]
pub trait VehicleIncidentRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_id: Uuid,
        tipo: VehicleIncidentType,
        data_ocorrencia: chrono::DateTime<chrono::Utc>,
        local_ocorrencia: Option<&str>,
        numero_bo: &str,
        numero_seguradora: Option<&str>,
        descricao: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleIncidentDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleIncidentDto>, RepositoryError>;

    async fn update_status(
        &self,
        id: Uuid,
        status: VehicleIncidentStatus,
        notas_resolucao: Option<&str>,
        numero_seguradora: Option<&str>,
        encerrado_por: Option<Uuid>,
        version: i32,
    ) -> Result<VehicleIncidentDto, RepositoryError>;

    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        status: Option<VehicleIncidentStatus>,
    ) -> Result<Vec<VehicleIncidentDto>, RepositoryError>;
}
