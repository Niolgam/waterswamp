use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::current_user::CurrentUser,
    infra::state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct StockSummaryParams {
    pub warehouse_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct DailyMovementsParams {
    pub warehouse_id: Option<Uuid>,
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SupplierPerfParams {
    pub limit: Option<i64>,
}

pub async fn get_stock_summary(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<StockSummaryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .dashboard_service
        .get_stock_summary(params.warehouse_id)
        .await
        .map(|rows| Json(serde_json::json!({ "data": rows })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_daily_movements(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<DailyMovementsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .dashboard_service
        .get_daily_movements(params.warehouse_id, params.days)
        .await
        .map(|rows| Json(serde_json::json!({ "data": rows })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_supplier_performance(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<SupplierPerfParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .dashboard_service
        .get_supplier_performance(params.limit)
        .await
        .map(|rows| Json(serde_json::json!({ "data": rows })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn refresh_all(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .dashboard_service
        .refresh_all()
        .await
        .map(|result| Json(serde_json::json!(result)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
