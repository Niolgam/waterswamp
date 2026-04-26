use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::legacy_import::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::current_user::CurrentUser,
    infra::state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListJobsParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_jobs(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListJobsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(20).max(1).min(100);
    let offset = params.offset.unwrap_or(0).max(0);

    state
        .legacy_import_service
        .list_jobs(limit, offset)
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

pub async fn get_job(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ImportJobDto>, (StatusCode, String)> {
    state
        .legacy_import_service
        .get_job(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn import_suppliers(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<ImportSuppliersPayload>,
) -> Result<(StatusCode, Json<ImportJobDto>), (StatusCode, String)> {
    state
        .legacy_import_service
        .import_suppliers(payload, Some(user.id))
        .await
        .map(|job| (StatusCode::CREATED, Json(job)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn import_catalog_items(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<ImportCatalogItemsPayload>,
) -> Result<(StatusCode, Json<ImportJobDto>), (StatusCode, String)> {
    state
        .legacy_import_service
        .import_catalog_items(payload, Some(user.id))
        .await
        .map(|job| (StatusCode::CREATED, Json(job)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn import_initial_stock(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<ImportInitialStockPayload>,
) -> Result<(StatusCode, Json<ImportJobDto>), (StatusCode, String)> {
    state
        .legacy_import_service
        .import_initial_stock(payload, Some(user.id))
        .await
        .map(|job| (StatusCode::CREATED, Json(job)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
