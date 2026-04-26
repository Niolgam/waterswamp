use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use domain::models::abc_analysis::RunAbcInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::current_user::CurrentUser,
    infra::state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AbcResultsParams {
    pub warehouse_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn run_analysis(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(input): Json<RunAbcInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .abc_analysis_service
        .run_analysis(input)
        .await
        .map(|summary| Json(serde_json::json!(summary)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_results(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<AbcResultsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50).max(1).min(500);
    let offset = params.offset.unwrap_or(0).max(0);

    state
        .abc_analysis_service
        .get_latest_results(params.warehouse_id, limit, offset)
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

pub async fn get_latest_run(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<AbcResultsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .abc_analysis_service
        .get_latest_run_at(params.warehouse_id)
        .await
        .map(|run_at| Json(serde_json::json!({ "latest_run_at": run_at })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
