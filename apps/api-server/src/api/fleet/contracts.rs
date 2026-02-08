use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export domain types for API use
pub use domain::models::vehicle::{
    AcquisitionType, ChangeVehicleStatusPayload, CreateVehicleCategoryPayload,
    CreateVehicleColorPayload, CreateVehicleFuelTypePayload, CreateVehicleMakePayload,
    CreateVehicleModelPayload, CreateVehiclePayload, CreateVehicleTransmissionTypePayload,
    DocumentType, UpdateVehicleCategoryPayload, UpdateVehicleColorPayload,
    UpdateVehicleFuelTypePayload, UpdateVehicleMakePayload, UpdateVehicleModelPayload,
    UpdateVehiclePayload, UpdateVehicleTransmissionTypePayload, VehicleCategoryDto,
    VehicleColorDto, VehicleDocumentDto, VehicleDto, VehicleFuelTypeDto, VehicleMakeDto,
    VehicleModelDto, VehicleStatus, VehicleStatusHistoryDto, VehicleTransmissionTypeDto,
    VehicleWithDetailsDto,
};

// ============================
// List Responses
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleCategoriesListResponse {
    pub categories: Vec<VehicleCategoryDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleMakesListResponse {
    pub makes: Vec<VehicleMakeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleModelsListResponse {
    pub models: Vec<VehicleModelDto>,
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
