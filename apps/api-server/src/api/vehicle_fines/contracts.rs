use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::vehicle_fine::{
    ChangeFineStatusPayload, CreateVehicleFinePayload, CreateVehicleFineTypePayload, FineSeverity,
    FineStatus, UpdateVehicleFinePayload, UpdateVehicleFineTypePayload, VehicleFineDto,
    VehicleFineStatusHistoryDto, VehicleFineTypeDto, VehicleFineWithDetailsDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleFineTypesListResponse {
    pub data: Vec<VehicleFineTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VehicleFinesListResponse {
    pub data: Vec<VehicleFineWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
