use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::vehicle_fine::{
    CreateVehicleFinePayload, CreateVehicleFineTypePayload,
    FinePaymentStatus, FineSeverity,
    UpdateVehicleFinePayload, UpdateVehicleFineTypePayload,
    VehicleFineDto, VehicleFineTypeDto, VehicleFineWithDetailsDto,
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
