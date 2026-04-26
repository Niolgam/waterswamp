use crate::extractors::current_user::CurrentUser;
use crate::infra::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::warehouse::{
    CancelInventorySessionPayload, ConfirmGovbrSignatureInventoryPayload,
    CreateInventorySessionPayload, InventorySessionStatus, ReconcileInventoryPayload,
    SubmitCountPayload,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SessionListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<InventorySessionStatus>,
}

fn default_limit() -> i64 {
    50
}

/// POST /api/admin/warehouses/:warehouse_id/inventory-sessions
pub async fn create_session(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<CreateInventorySessionPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    state
        .inventory_service
        .create_session(warehouse_id, payload, user.id)
        .await
        .map(|s| (StatusCode::CREATED, Json(serde_json::json!(s))))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// GET /api/admin/warehouses/:warehouse_id/inventory-sessions
pub async fn list_sessions(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(warehouse_id): Path<Uuid>,
    Query(query): Query<SessionListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .list_sessions(warehouse_id, query.limit, query.offset, query.status)
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

/// GET /api/admin/inventory-sessions/:id
pub async fn get_session(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .get_session(session_id)
        .await
        .map(|s| Json(serde_json::json!(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/inventory-sessions/:id/start-counting
pub async fn start_counting(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .start_counting(session_id)
        .await
        .map(|s| Json(serde_json::json!(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/inventory-sessions/:id/submit-count
pub async fn submit_count(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<SubmitCountPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .submit_count(session_id, payload)
        .await
        .map(|item| Json(serde_json::json!(item)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/inventory-sessions/:id/start-reconciliation
pub async fn start_reconciliation(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .start_reconciliation(session_id)
        .await
        .map(|s| Json(serde_json::json!(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/inventory-sessions/:id/reconcile
pub async fn reconcile(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<ReconcileInventoryPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .reconcile(session_id, payload, user.id)
        .await
        .map(|s| Json(serde_json::json!(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/inventory-sessions/:id/confirm-govbr-signature
pub async fn confirm_govbr_signature(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(_payload): Json<ConfirmGovbrSignatureInventoryPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .confirm_govbr_signature(session_id, user.id)
        .await
        .map(|s| Json(serde_json::json!(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// POST /api/admin/inventory-sessions/:id/cancel
pub async fn cancel_session(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<CancelInventorySessionPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .inventory_service
        .cancel_session(session_id, payload)
        .await
        .map(|s| Json(serde_json::json!(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
