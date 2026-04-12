use crate::extractors::current_user::CurrentUser;
use crate::infra::{errors::AppError, state::AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::warehouse::{
    CancelTransferPayload, ConfirmTransferPayload, InitiateTransferPayload, RejectTransferPayload,
    StockTransferStatus,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TransferListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub source_warehouse_id: Option<Uuid>,
    pub destination_warehouse_id: Option<Uuid>,
    pub status: Option<StockTransferStatus>,
}

/// GET /api/admin/transfers
pub async fn list_transfers(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<TransferListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let offset = query.offset.unwrap_or(0).max(0);

    let (transfers, total) = state
        .stock_transfer_service
        .list_transfers(
            limit,
            offset,
            query.source_warehouse_id,
            query.destination_warehouse_id,
            query.status,
        )
        .await?;

    Ok(Json(serde_json::json!({
        "data": transfers,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

/// GET /api/admin/transfers/:id
pub async fn get_transfer(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let transfer = state.stock_transfer_service.get_transfer(id).await?;
    Ok(Json(serde_json::json!(transfer)))
}

/// POST /api/admin/warehouses/:warehouse_id/transfers
/// RF-018 Step 1: Initiate transfer from source warehouse
pub async fn initiate_transfer(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<InitiateTransferPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let transfer = state
        .stock_transfer_service
        .initiate_transfer(warehouse_id, payload, user.id)
        .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!(transfer))))
}

/// POST /api/admin/transfers/:id/confirm
/// RF-018 Step 2a: Destination confirms receipt
pub async fn confirm_transfer(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConfirmTransferPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let transfer = state
        .stock_transfer_service
        .confirm_transfer(id, payload, user.id)
        .await?;

    Ok(Json(serde_json::json!(transfer)))
}

/// POST /api/admin/transfers/:id/reject
/// RF-018 Step 2b: Destination rejects transfer
pub async fn reject_transfer(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectTransferPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let transfer = state
        .stock_transfer_service
        .reject_transfer(id, payload, user.id)
        .await?;

    Ok(Json(serde_json::json!(transfer)))
}

/// POST /api/admin/transfers/:id/cancel
/// Source cancels a pending transfer
pub async fn cancel_transfer(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelTransferPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let transfer = state
        .stock_transfer_service
        .cancel_transfer(id, payload, user.id)
        .await?;

    Ok(Json(serde_json::json!(transfer)))
}
