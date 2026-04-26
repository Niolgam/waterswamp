use crate::extractors::current_user::CurrentUser;
use crate::infra::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::batch::{
    BatchOccurrenceStatus, CloseOccurrencePayload, CreateBatchQualityOccurrencePayload,
    FefoExitPayload, ResolveOccurrencePayload,
};
use serde::Deserialize;
use uuid::Uuid;

// ============================================================================
// Batch Stocks / FEFO
// ============================================================================

/// GET /api/admin/warehouses/:id/batch-stocks/:catalog_item_id
pub async fn list_batches(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path((warehouse_id, catalog_item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .list_batches(warehouse_id, catalog_item_id)
        .await
        .map(|data| Json(serde_json::json!({ "data": data })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// GET /api/admin/batch-stocks/near-expiry
#[derive(Debug, Deserialize)]
pub struct NearExpiryQuery {
    pub warehouse_id: Option<Uuid>,
    pub days_ahead: Option<i32>,
}

pub async fn list_near_expiry(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<NearExpiryQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .list_near_expiry(q.warehouse_id, q.days_ahead)
        .await
        .map(|data| Json(serde_json::json!({ "data": data })))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/warehouses/:id/fefo-exit
pub async fn fefo_exit(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<FefoExitPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .fefo_exit(warehouse_id, payload, user.id)
        .await
        .map(|result| Json(serde_json::json!(result)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================================================================
// Batch Quality Occurrences (RF-043)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OccurrenceListQuery {
    pub warehouse_id: Option<Uuid>,
    pub catalog_item_id: Option<Uuid>,
    pub batch_number: Option<String>,
    pub status: Option<BatchOccurrenceStatus>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// GET /api/admin/batch-quality-occurrences
pub async fn list_occurrences(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<OccurrenceListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .list_occurrences(
            q.warehouse_id,
            q.catalog_item_id,
            q.batch_number,
            q.status,
            q.limit,
            q.offset,
        )
        .await
        .map(|(data, total)| {
            Json(serde_json::json!({
                "data": data,
                "total": total,
                "limit": q.limit,
                "offset": q.offset,
            }))
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// GET /api/admin/batch-quality-occurrences/:id
pub async fn get_occurrence(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .get_occurrence(id)
        .await
        .map(|o| Json(serde_json::json!(o)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/batch-quality-occurrences
pub async fn create_occurrence(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateBatchQualityOccurrencePayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    state
        .batch_service
        .create_occurrence(payload, user.id)
        .await
        .map(|o| (StatusCode::CREATED, Json(serde_json::json!(o))))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/batch-quality-occurrences/:id/resolve
pub async fn resolve_occurrence(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveOccurrencePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .resolve_occurrence(id, payload, user.id)
        .await
        .map(|o| Json(serde_json::json!(o)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/batch-quality-occurrences/:id/close
pub async fn close_occurrence(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CloseOccurrencePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .batch_service
        .close_occurrence(id, payload, user.id)
        .await
        .map(|o| Json(serde_json::json!(o)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
