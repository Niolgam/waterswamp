use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::fueling::{
    CreateFuelingPayload, FuelingDto, FuelingWithDetailsDto, UpdateFuelingPayload,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FuelingsListResponse {
    pub data: Vec<FuelingWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
