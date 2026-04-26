use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::warehouse::{
    CancelDisposalRequestPayload, CreateDisposalRequestPayload, DisposalRequestStatus,
    ManualExitPayload, ReturnEntryPayload, StandaloneEntryPayload, StockMovementDto,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct DisposalListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<DisposalRequestStatus>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MovementListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub catalog_item_id: Option<Uuid>,
    pub movement_type: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct MovementsListResponse {
    pub data: Vec<StockMovementDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub warehouse_id: Uuid,
}

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
                data: warehouses,
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

// ============================
// Stock Movement Handlers
// ============================

/// GET /api/admin/warehouses/:id/movements
/// List stock movements for a warehouse (audit trail)
pub async fn list_stock_movements(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Query(query): Query<MovementListQuery>,
) -> Result<Json<MovementsListResponse>, (StatusCode, String)> {
    state
        .warehouse_service
        .list_stock_movements(
            warehouse_id,
            query.limit,
            query.offset,
            query.catalog_item_id,
            query.movement_type,
        )
        .await
        .map(|(movements, total)| {
            Json(MovementsListResponse {
                data: movements,
                total,
                limit: query.limit,
                offset: query.offset,
                warehouse_id,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/warehouses/:id/entries
/// RF-009: Entrada Avulsa (doação ou ajuste de inventário)
pub async fn create_standalone_entry(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<StandaloneEntryPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    state
        .warehouse_service
        .create_standalone_entry(warehouse_id, payload, user.id)
        .await
        .map(|r| {
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "movements_created": r.movements_created,
                    "entry_type": r.entry_type,
                    "origin_description": r.origin_description,
                    "warehouse_id": r.warehouse_id
                })),
            )
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/warehouses/:id/returns
/// RF-011: Devolução de Requisição
pub async fn create_return_entry(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<ReturnEntryPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    state
        .warehouse_service
        .create_return_entry(warehouse_id, payload, user.id)
        .await
        .map(|r| {
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "movements_created": r.movements_created,
                    "requisition_id": r.requisition_id,
                    "warehouse_id": r.warehouse_id
                })),
            )
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/warehouses/:id/disposal-requests
/// RF-016: Cria pedido de desfazimento em AWAITING_SIGNATURE (RN-005, Ticket 1.1)
pub async fn create_disposal_request(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<CreateDisposalRequestPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    state
        .warehouse_service
        .create_disposal_request(warehouse_id, payload, user.id)
        .await
        .map(|r| (StatusCode::CREATED, Json(serde_json::json!(r))))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// GET /api/admin/warehouses/:id/disposal-requests
pub async fn list_disposal_requests(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Query(query): Query<DisposalListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .warehouse_service
        .list_disposal_requests(warehouse_id, query.limit, query.offset, query.status)
        .await
        .map(|(data, total)| {
            Json(serde_json::json!({
                "data": data,
                "total": total,
                "limit": query.limit,
                "offset": query.offset
            }))
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// GET /api/admin/disposal-requests/:id
pub async fn get_disposal_request(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .warehouse_service
        .get_disposal_request(request_id)
        .await
        .map(|r| Json(serde_json::json!(r)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/disposal-requests/:id/confirm-signature
/// RF-016: Confirma assinatura Gov.br e deduz estoque (Ticket 1.1)
pub async fn confirm_disposal_signature(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .warehouse_service
        .confirm_disposal_signature(request_id, user.id)
        .await
        .map(|r| Json(serde_json::json!(r)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/disposal-requests/:id/cancel
pub async fn cancel_disposal_request(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
    Json(payload): Json<CancelDisposalRequestPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .warehouse_service
        .cancel_disposal_request(request_id, payload, user.id)
        .await
        .map(|r| Json(serde_json::json!(r)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/warehouses/:id/manual-exits
/// RF-017: Saída por Ordem de Serviço ou saída manual
pub async fn create_manual_exit(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<ManualExitPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    state
        .warehouse_service
        .create_manual_exit(warehouse_id, payload, user.id)
        .await
        .map(|r| {
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "movements_created": r.movements_created,
                    "document_number": r.document_number,
                    "warehouse_id": r.warehouse_id
                })),
            )
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
