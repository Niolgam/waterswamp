use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::alert::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::current_user::CurrentUser,
    infra::state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListAlertsParams {
    pub warehouse_id: Option<Uuid>,
    pub status: Option<StockAlertStatus>,
    pub alert_type: Option<StockAlertType>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_alerts(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListAlertsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(20).max(1).min(100);
    let offset = params.offset.unwrap_or(0).max(0);

    state
        .alert_service
        .list_alerts(params.warehouse_id, params.status, params.alert_type, limit, offset)
        .await
        .map(|(rows, total)| {
            Json(serde_json::json!({
                "data": rows,
                "total": total,
                "limit": limit,
                "offset": offset
            }))
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn create_alert(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(input): Json<CreateStockAlertInput>,
) -> Result<(StatusCode, Json<StockAlertDto>), (StatusCode, String)> {
    state
        .alert_service
        .create_alert(input)
        .await
        .map(|alert| (StatusCode::CREATED, Json(alert)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_alert(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<StockAlertDto>, (StatusCode, String)> {
    state
        .alert_service
        .get_alert(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn acknowledge_alert(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<AcknowledgeAlertPayload>,
) -> Result<Json<StockAlertDto>, (StatusCode, String)> {
    state
        .alert_service
        .acknowledge_alert(id, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn resolve_alert(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<StockAlertDto>, (StatusCode, String)> {
    state
        .alert_service
        .resolve_alert(id, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn process_sla_breaches(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .alert_service
        .process_sla_breaches()
        .await
        .map(|count| Json(serde_json::json!({ "breached_count": count })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
