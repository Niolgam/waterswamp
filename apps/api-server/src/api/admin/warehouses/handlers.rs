use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct WarehouseListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub warehouse_type: Option<WarehouseType>,
    pub city_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct StockListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub is_blocked: Option<bool>,
}

fn default_limit() -> i64 {
    50
}

pub async fn create_warehouse(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateWarehousePayload>,
) -> Result<(StatusCode, Json<WarehouseWithDetailsDto>), (StatusCode, String)> {
    state
        .warehouse_service
        .create_warehouse(payload)
        .await
        .map(|w| (StatusCode::CREATED, Json(w)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_warehouse(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WarehouseWithDetailsDto>, (StatusCode, String)> {
    state
        .warehouse_service
        .get_warehouse(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_warehouse(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateWarehousePayload>,
) -> Result<Json<WarehouseWithDetailsDto>, (StatusCode, String)> {
    state
        .warehouse_service
        .update_warehouse(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_warehouse(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .warehouse_service
        .delete_warehouse(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_warehouses(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<WarehouseListQuery>,
) -> Result<Json<WarehousesListResponse>, (StatusCode, String)> {
    state
        .warehouse_service
        .list_warehouses(
            query.limit,
            query.offset,
            query.search,
            query.warehouse_type,
            query.city_id,
            query.is_active,
        )
        .await
        .map(|(warehouses, total)| {
            Json(WarehousesListResponse {
                warehouses,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Warehouse Stock Handlers
// ============================

pub async fn get_stock(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
) -> Result<Json<WarehouseStockWithDetailsDto>, (StatusCode, String)> {
    state
        .warehouse_service
        .get_stock(stock_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_warehouse_stocks(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Query(query): Query<StockListQuery>,
) -> Result<Json<WarehouseStocksListResponse>, (StatusCode, String)> {
    state
        .warehouse_service
        .list_warehouse_stocks(
            warehouse_id,
            query.limit,
            query.offset,
            query.search,
            query.is_blocked,
        )
        .await
        .map(|(stocks, total)| {
            Json(WarehouseStocksListResponse {
                stocks,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_stock_params(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
    Json(payload): Json<UpdateStockParamsPayload>,
) -> Result<Json<WarehouseStockDto>, (StatusCode, String)> {
    state
        .warehouse_service
        .update_stock_params(stock_id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn block_stock(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
    Json(payload): Json<BlockStockPayload>,
) -> Result<Json<WarehouseStockDto>, (StatusCode, String)> {
    state
        .warehouse_service
        .block_stock(stock_id, payload, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn unblock_stock(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
) -> Result<Json<WarehouseStockDto>, (StatusCode, String)> {
    state
        .warehouse_service
        .unblock_stock(stock_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
