use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::warehouse::{
    BlockStockPayload, CreateWarehousePayload, UpdateStockParamsPayload, UpdateWarehousePayload,
    WarehouseStockDto, WarehouseStockWithDetailsDto, WarehouseType, WarehouseWithDetailsDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WarehousesListResponse {
    pub warehouses: Vec<WarehouseWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WarehouseStocksListResponse {
    pub stocks: Vec<WarehouseStockWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
