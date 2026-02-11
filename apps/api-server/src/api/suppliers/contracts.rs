use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::supplier::{
    CreateSupplierPayload, SupplierDto, SupplierWithDetailsDto,
    UpdateSupplierPayload,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuppliersListResponse {
    pub suppliers: Vec<SupplierWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
