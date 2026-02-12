use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::driver::{
    CreateDriverPayload, DriverDto, DriverType, UpdateDriverPayload,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriversListResponse {
    pub drivers: Vec<DriverDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
