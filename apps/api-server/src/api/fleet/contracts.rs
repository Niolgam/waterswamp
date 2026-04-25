use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export domain types for API use
pub use domain::models::asset_management::{
    // Sinistros
    VehicleIncidentType, VehicleIncidentStatus, VehicleIncidentDto,
    CreateVehicleIncidentPayload, UpdateVehicleIncidentPayload,
    // Transferências departamentais
    VehicleDepartmentTransferDto, CreateVehicleDepartmentTransferPayload,
    // Depreciação
    DepreciationConfigDto, UpsertDepreciationConfigPayload, DepreciationCalculationDto,
    // Baixa/desfazimento
    DisposalStatus, DisposalDestination,
    VehicleDisposalProcessDto, CreateDisposalProcessPayload, AdvanceDisposalPayload,
    VehicleDisposalStepDto, CreateDisposalStepPayload,
    // Catálogos ADM
    FleetFuelCatalogDto, CreateFleetFuelCatalogPayload, UpdateFleetFuelCatalogPayload,
    FleetMaintenanceServiceDto, CreateFleetMaintenanceServicePayload, UpdateFleetMaintenanceServicePayload,
    FleetSystemParamDto, UpsertFleetSystemParamPayload,
    FleetChecklistTemplateDto, CreateFleetChecklistTemplatePayload,
    FleetChecklistItemDto, CreateFleetChecklistItemPayload,
};

pub use domain::models::vehicle::{
    AcquisitionType, AllocationStatus, ChangeOperationalStatusPayload, ChangeVehicleStatusPayload,
    CreateVehicleCategoryPayload, CreateVehicleColorPayload, CreateVehicleFuelTypePayload,
    CreateVehicleMakePayload, CreateVehicleModelPayload, CreateVehiclePayload,
    CreateVehicleTransmissionTypePayload, DocumentType, OperationalStatus,
    UpdateVehicleCategoryPayload, UpdateVehicleColorPayload, UpdateVehicleFuelTypePayload,
    UpdateVehicleMakePayload, UpdateVehicleModelPayload, UpdateVehiclePayload,
    UpdateVehicleTransmissionTypePayload, VehicleCategoryDto, VehicleColorDto, VehicleDocumentDto,
    VehicleDto, VehicleFuelTypeDto, VehicleMakeDto, VehicleModelDto, VehicleModelWithDetailsDto,
    VehicleStatus, VehicleStatusHistoryDto, VehicleTransmissionTypeDto, VehicleWithDetailsDto,
};

// ============================
// List Responses
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleCategoriesListResponse {
    pub data: Vec<VehicleCategoryDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleMakesListResponse {
    pub data: Vec<VehicleMakeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleModelsListResponse {
    pub data: Vec<VehicleModelWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleColorsListResponse {
    pub colors: Vec<VehicleColorDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleFuelTypesListResponse {
    pub fuel_types: Vec<VehicleFuelTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleTransmissionTypesListResponse {
    pub transmission_types: Vec<VehicleTransmissionTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehiclesListResponse {
    pub vehicles: Vec<VehicleWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleSearchResponse {
    pub vehicles: Vec<VehicleDto>,
}
