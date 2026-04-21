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

// ============================
// Vehicle Disposal Port
// ============================

#[async_trait]
pub trait VehicleDisposalRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_id: Uuid,
        destino: DisposalDestination,
        justificativa: &str,
        numero_laudo: &str,
        documento_sei: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDisposalProcessDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleDisposalProcessDto>, RepositoryError>;

    async fn find_by_vehicle(&self, vehicle_id: Uuid) -> Result<Option<VehicleDisposalProcessDto>, RepositoryError>;

    async fn advance_status(
        &self,
        id: Uuid,
        new_status: DisposalStatus,
        concluido_por: Option<Uuid>,
        cancelado_por: Option<Uuid>,
        motivo_cancelamento: Option<&str>,
        version: i32,
    ) -> Result<VehicleDisposalProcessDto, RepositoryError>;

    async fn list(&self, status: Option<DisposalStatus>) -> Result<Vec<VehicleDisposalProcessDto>, RepositoryError>;

    // RF-AST-10 steps
    async fn add_step(
        &self,
        disposal_id: Uuid,
        descricao: &str,
        documento_sei: &str,
        data_execucao: chrono::NaiveDate,
        responsavel_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDisposalStepDto, RepositoryError>;

    async fn list_steps(&self, disposal_id: Uuid) -> Result<Vec<VehicleDisposalStepDto>, RepositoryError>;
}

// ============================
// Fleet Fuel Catalog Port
// ============================

#[async_trait]
pub trait FleetFuelCatalogRepositoryPort: Send + Sync {
    async fn create(
        &self,
        nome: &str,
        catmat_item_id: Option<Uuid>,
        unidade: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FleetFuelCatalogDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FleetFuelCatalogDto>, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        nome: Option<&str>,
        catmat_item_id: Option<Option<Uuid>>,
        unidade: Option<&str>,
        ativo: Option<bool>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FleetFuelCatalogDto, RepositoryError>;

    async fn list(&self, only_active: bool) -> Result<Vec<FleetFuelCatalogDto>, RepositoryError>;
}

// ============================
// Fleet Maintenance Services Port
// ============================

#[async_trait]
pub trait FleetMaintenanceServiceRepositoryPort: Send + Sync {
    async fn create(
        &self,
        nome: &str,
        catser_item_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FleetMaintenanceServiceDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FleetMaintenanceServiceDto>, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        nome: Option<&str>,
        catser_item_id: Option<Option<Uuid>>,
        ativo: Option<bool>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FleetMaintenanceServiceDto, RepositoryError>;

    async fn list(&self, only_active: bool) -> Result<Vec<FleetMaintenanceServiceDto>, RepositoryError>;
}

// ============================
// Fleet System Params Port
// ============================

#[async_trait]
pub trait FleetSystemParamRepositoryPort: Send + Sync {
    async fn find_by_key(&self, chave: &str) -> Result<Option<FleetSystemParamDto>, RepositoryError>;

    async fn upsert(
        &self,
        chave: &str,
        valor: &str,
        descricao: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FleetSystemParamDto, RepositoryError>;

    async fn list(&self) -> Result<Vec<FleetSystemParamDto>, RepositoryError>;
}

// ============================
// Fleet Checklist Template Port
// ============================

#[async_trait]
pub trait FleetChecklistTemplateRepositoryPort: Send + Sync {
    async fn create(
        &self,
        nome: &str,
        descricao: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FleetChecklistTemplateDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FleetChecklistTemplateDto>, RepositoryError>;

    async fn list(&self, only_active: bool) -> Result<Vec<FleetChecklistTemplateDto>, RepositoryError>;

    async fn add_item(
        &self,
        template_id: Uuid,
        descricao: &str,
        obrigatorio: bool,
        ordem: i32,
    ) -> Result<FleetChecklistItemDto, RepositoryError>;

    async fn list_items(&self, template_id: Uuid) -> Result<Vec<FleetChecklistItemDto>, RepositoryError>;
}
