use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::organizational::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::extractors::current_user::CurrentUser;
use crate::infra::state::AppState;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ListQueueParams {
    pub status: Option<SyncStatus>,
    pub entity_type: Option<SiorgEntityType>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ListHistoryParams {
    pub entity_type: Option<SiorgEntityType>,
    pub siorg_code: Option<i32>,
    pub change_type: Option<SiorgChangeType>,
    pub requires_review: Option<bool>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

// ============================================================================
// Response Models
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct QueueStatsResponse {
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
    pub conflicts: i64,
    pub skipped: i64,
}

// ============================================================================
// Queue Management Handlers
// ============================================================================

/// List sync queue items with filters
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/queue",
    params(ListQueueParams),
    responses(
        (status = 200, description = "Queue items retrieved", body = Vec<SiorgSyncQueueItem>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_queue_items(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListQueueParams>,
) -> Result<Json<Vec<SiorgSyncQueueItem>>, (StatusCode, String)> {
    let items = state
        .siorg_sync_queue_repository
        .list(params.status, params.entity_type, params.limit, params.offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

/// Get sync queue statistics
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/queue/stats",
    responses(
        (status = 200, description = "Queue statistics", body = QueueStatsResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_queue_stats(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<QueueStatsResponse>, (StatusCode, String)> {
    let repo = &state.siorg_sync_queue_repository;

    // Count by status concurrently
    let (pending, processing, completed, failed, conflicts, skipped) = tokio::try_join!(
        repo.count_by_status(SyncStatus::Pending),
        repo.count_by_status(SyncStatus::Processing),
        repo.count_by_status(SyncStatus::Completed),
        repo.count_by_status(SyncStatus::Failed),
        repo.count_by_status(SyncStatus::Conflict),
        repo.count_by_status(SyncStatus::Skipped),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(QueueStatsResponse {
        pending,
        processing,
        completed,
        failed,
        conflicts,
        skipped,
    }))
}

/// Get queue item details
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/queue/{id}",
    params(
        ("id" = Uuid, Path, description = "Queue item ID")
    ),
    responses(
        (status = 200, description = "Queue item found", body = SiorgSyncQueueItem),
        (status = 404, description = "Queue item not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_queue_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SiorgSyncQueueItem>, (StatusCode, String)> {
    let item = state
        .siorg_sync_queue_repository
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Queue item not found".to_string()))?;

    Ok(Json(item))
}

/// Delete queue item
#[utoipa::path(
    delete,
    path = "/api/admin/organizational/sync/queue/{id}",
    params(
        ("id" = Uuid, Path, description = "Queue item ID")
    ),
    responses(
        (status = 204, description = "Queue item deleted"),
        (status = 404, description = "Queue item not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_queue_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .siorg_sync_queue_repository
        .delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Conflict Resolution Handlers
// ============================================================================

/// List conflicts (queue items with CONFLICT status)
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/conflicts",
    params(
        ("limit" = Option<i64>, Query, description = "Max number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "Conflicts retrieved", body = Vec<SiorgSyncQueueItem>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_conflicts(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListQueueParams>,
) -> Result<Json<Vec<SiorgSyncQueueItem>>, (StatusCode, String)> {
    let limit = params.limit;
    let offset = params.offset;

    let conflicts = state
        .siorg_sync_queue_repository
        .get_conflicts(limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(conflicts))
}

/// Get conflict details with field-by-field diff
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/conflicts/{id}",
    params(
        ("id" = Uuid, Path, description = "Queue item ID")
    ),
    responses(
        (status = 200, description = "Conflict details", body = ConflictDetail),
        (status = 404, description = "Conflict not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_conflict_detail(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConflictDetail>, (StatusCode, String)> {
    // Create conflict resolution service on demand
    let service = application::services::conflict_resolution_service::ConflictResolutionService::new(
        state.siorg_sync_queue_repository.clone(),
        state.siorg_history_repository.clone(),
        state.organization_service.repository.clone(),
        state.organizational_unit_service.unit_repository.clone(),
    );

    let detail = service
        .get_conflict_detail(id)
        .await
        .map_err(|e| match e {
            application::services::conflict_resolution_service::ServiceError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg)
            }
            application::services::conflict_resolution_service::ServiceError::InvalidOperation(
                msg,
            ) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(Json(detail))
}

/// Resolve conflict
#[utoipa::path(
    post,
    path = "/api/admin/organizational/sync/conflicts/{id}/resolve",
    params(
        ("id" = Uuid, Path, description = "Queue item ID")
    ),
    request_body = ResolveConflictPayload,
    responses(
        (status = 200, description = "Conflict resolved"),
        (status = 404, description = "Conflict not found"),
        (status = 400, description = "Invalid resolution"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn resolve_conflict(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveConflictPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Create conflict resolution service on demand
    let service = application::services::conflict_resolution_service::ConflictResolutionService::new(
        state.siorg_sync_queue_repository.clone(),
        state.siorg_history_repository.clone(),
        state.organization_service.repository.clone(),
        state.organizational_unit_service.unit_repository.clone(),
    );

    service
        .resolve_conflict(id, payload, Some(user.id))
        .await
        .map_err(|e| match e {
            application::services::conflict_resolution_service::ServiceError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg)
            }
            application::services::conflict_resolution_service::ServiceError::InvalidOperation(
                msg,
            )
            | application::services::conflict_resolution_service::ServiceError::InvalidData(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::OK)
}

// ============================================================================
// History Handlers
// ============================================================================

/// List sync history with filters
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/history",
    params(ListHistoryParams),
    responses(
        (status = 200, description = "History retrieved", body = Vec<SiorgHistoryItem>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_history(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListHistoryParams>,
) -> Result<Json<Vec<SiorgHistoryItem>>, (StatusCode, String)> {
    let items = state
        .siorg_history_repository
        .list(
            params.entity_type,
            params.siorg_code,
            params.change_type,
            params.requires_review,
            params.limit,
            params.offset,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

/// Get history item details
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/history/{id}",
    params(
        ("id" = Uuid, Path, description = "History item ID")
    ),
    responses(
        (status = 200, description = "History item found", body = SiorgHistoryItem),
        (status = 404, description = "History item not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_history_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SiorgHistoryItem>, (StatusCode, String)> {
    let item = state
        .siorg_history_repository
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "History item not found".to_string()))?;

    Ok(Json(item))
}

/// Get history for a specific entity
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/history/entity/{entity_type}/{siorg_code}",
    params(
        ("entity_type" = SiorgEntityType, Path, description = "Entity type"),
        ("siorg_code" = i32, Path, description = "SIORG code"),
        ("limit" = Option<i64>, Query, description = "Max number of results")
    ),
    responses(
        (status = 200, description = "Entity history retrieved", body = Vec<SiorgHistoryItem>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_entity_history(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path((entity_type, siorg_code)): Path<(String, i32)>,
    Query(params): Query<ListHistoryParams>,
) -> Result<Json<Vec<SiorgHistoryItem>>, (StatusCode, String)> {
    // Parse entity type
    let entity_type: SiorgEntityType = serde_json::from_value(serde_json::json!(entity_type))
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid entity type: {}", e)))?;

    let limit = params.limit;

    let items = state
        .siorg_history_repository
        .get_entity_history(entity_type, siorg_code, limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

/// Mark history item as reviewed
#[utoipa::path(
    post,
    path = "/api/admin/organizational/sync/history/{id}/review",
    params(
        ("id" = Uuid, Path, description = "History item ID")
    ),
    request_body = ReviewHistoryPayload,
    responses(
        (status = 200, description = "History item marked as reviewed"),
        (status = 404, description = "History item not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn review_history_item(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewHistoryPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .siorg_history_repository
        .mark_reviewed(id, user.id, payload.notes)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewHistoryPayload {
    pub notes: Option<String>,
}
