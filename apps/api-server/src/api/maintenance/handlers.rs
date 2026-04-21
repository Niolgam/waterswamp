use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use application::errors::ServiceError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

fn occ_response(msg: String) -> axum::response::Response {
    use axum::response::IntoResponse;
    (
        StatusCode::CONFLICT,
        Json(serde_json::json!({
            "type": "optimistic-lock-failure",
            "title": "Conflict",
            "status": 409,
            "detail": msg
        })),
    )
        .into_response()
}

// ── RF-MNT-01: Listar / Abrir OS ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    pub vehicle_id: Option<Uuid>,
    pub status: Option<MaintenanceOrderStatus>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 { 50 }

pub async fn list_orders(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<OrderListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (orders, total) = state
        .maintenance_service
        .list_orders(q.vehicle_id, q.status, q.limit, q.offset)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": orders, "total": total })))
}

pub async fn open_order(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Json(payload): Json<CreateMaintenanceOrderPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.maintenance_service.open_order(vehicle_id, payload, Some(user.id)).await {
        Ok(o) => (StatusCode::CREATED, Json(o)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

pub async fn get_order(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MaintenanceOrderDto>, (StatusCode, String)> {
    state
        .maintenance_service
        .get_order(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ── RF-MNT-02: Avançar status da OS ──────────────────────────────────────

pub async fn advance_order(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdvanceMaintenanceOrderPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.maintenance_service.advance_order(id, payload, Some(user.id)).await {
        Ok(o) => (StatusCode::OK, Json(o)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-MNT-03: Itens de serviço ───────────────────────────────────────────

pub async fn add_item(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<CreateMaintenanceOrderItemPayload>,
) -> Result<(StatusCode, Json<MaintenanceOrderItemDto>), (StatusCode, String)> {
    state
        .maintenance_service
        .add_item(order_id, payload, Some(user.id))
        .await
        .map(|i| (StatusCode::CREATED, Json(i)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_items(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .maintenance_service
        .list_items(order_id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

// ── RF-MNT-04: Custo por veículo ──────────────────────────────────────────

pub async fn get_cost_summary(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<Json<MaintenanceCostSummaryDto>, (StatusCode, String)> {
    state
        .maintenance_service
        .cost_summary(vehicle_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
