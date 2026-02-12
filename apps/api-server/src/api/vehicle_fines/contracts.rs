use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::vehicle_fine::{
    ChangeFineStatusPayload,
    CreateVehicleFinePayload, CreateVehicleFineTypePayload,
    FineStatus, FineSeverity,
    UpdateVehicleFinePayload, UpdateVehicleFineTypePayload,
    VehicleFineDto, VehicleFineStatusHistoryDto, VehicleFineTypeDto, VehicleFineWithDetailsDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleFineTypesListResponse {
    pub fine_types: Vec<VehicleFineTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleFinesListResponse {
    pub fines: Vec<VehicleFineWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
