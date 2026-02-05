use super::contracts::*;
use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use domain::models::requisition::{
    ApproveRequisitionPayload, AuditContext, CancelRequisitionPayload, RejectRequisitionPayload,
    RollbackPayload,
};
use uuid::Uuid;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extracts the client IP address from headers
fn extract_ip(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (for proxied requests)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(s) = xff.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            return Some(s.split(',').next().unwrap_or(s).trim().to_string());
        }
    }

    // Try X-Real-IP
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(s) = xri.to_str() {
            return Some(s.to_string());
        }
    }

    None
}

/// Extracts the user agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Creates an AuditContext from the current request
fn create_audit_context(user: &CurrentUser, headers: &HeaderMap) -> AuditContext {
    AuditContext::new(user.id)
        .with_ip(extract_ip(headers))
        .with_user_agent(extract_user_agent(headers))
}

// ============================================================================
// REQUISITION HANDLERS
// ============================================================================

/// GET /api/admin/requisitions
/// List requisitions with optional filters
pub async fn list_requisitions(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(query): Query<RequisitionListQuery>,
) -> Result<Json<RequisitionListResponse>, AppError> {
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let offset = query.offset.unwrap_or(0).max(0);

    let (requisitions, total) = state
        .requisition_service
        .list_requisitions(limit, offset, query.status, query.requester_id, query.warehouse_id)
        .await?;

    Ok(Json(RequisitionListResponse {
        data: requisitions.into_iter().map(Into::into).collect(),
        total,
        limit,
        offset,
    }))
}

/// GET /api/admin/requisitions/:id
/// Get a single requisition by ID
pub async fn get_requisition(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<RequisitionResponse>, AppError> {
    let requisition = state.requisition_service.get_requisition(id).await?;
    Ok(Json(requisition.into()))
}

/// POST /api/admin/requisitions/:id/approve
/// Approve a pending requisition
pub async fn approve_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApproveRequest>,
) -> Result<Json<RequisitionResponse>, AppError> {
    let ctx = create_audit_context(&user, &headers);

    let requisition = state
        .requisition_service
        .approve_requisition(
            id,
            &ctx,
            ApproveRequisitionPayload {
                notes: payload.notes,
            },
        )
        .await?;

    Ok(Json(requisition.into()))
}

/// POST /api/admin/requisitions/:id/reject
/// Reject a pending requisition
pub async fn reject_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectRequest>,
) -> Result<Json<RequisitionResponse>, AppError> {
    let ctx = create_audit_context(&user, &headers);

    let requisition = state
        .requisition_service
        .reject_requisition(
            id,
            &ctx,
            RejectRequisitionPayload {
                reason: payload.reason,
            },
        )
        .await?;

    Ok(Json(requisition.into()))
}

/// POST /api/admin/requisitions/:id/cancel
/// Cancel a requisition
pub async fn cancel_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ctx = create_audit_context(&user, &headers);

    let result = state
        .requisition_service
        .cancel_requisition(
            id,
            &ctx,
            CancelRequisitionPayload {
                reason: payload.reason,
            },
        )
        .await?;

    Ok(Json(result))
}

// ============================================================================
// AUDIT / HISTORY HANDLERS
// ============================================================================

/// GET /api/admin/requisitions/:id/history
/// Get the audit history for a requisition
pub async fn get_requisition_history(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryListResponse>, AppError> {
    let history = state
        .requisition_service
        .get_requisition_history(id, query.limit)
        .await?;

    Ok(Json(HistoryListResponse {
        data: history.into_iter().map(Into::into).collect(),
        requisition_id: id,
    }))
}

/// GET /api/admin/requisitions/:id/rollback-points
/// Get available rollback points for a requisition
pub async fn get_rollback_points(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Query(query): Query<RollbackPointsQuery>,
) -> Result<Json<RollbackPointsResponse>, AppError> {
    let points = state
        .requisition_service
        .get_rollback_points(id, query.limit)
        .await?;

    Ok(Json(RollbackPointsResponse {
        data: points.into_iter().map(Into::into).collect(),
        requisition_id: id,
    }))
}

/// POST /api/admin/requisitions/:id/rollback
/// Rollback a requisition to a previous state
pub async fn rollback_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<RollbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ctx = create_audit_context(&user, &headers);

    let result = state
        .requisition_service
        .rollback_requisition(
            id,
            &ctx,
            RollbackPayload {
                history_id: payload.history_id,
                reason: payload.reason,
            },
        )
        .await?;

    Ok(Json(result))
}

// ============================================================================
// REQUISITION ITEM HANDLERS
// ============================================================================

/// GET /api/admin/requisitions/:id/items
/// Get items for a requisition
pub async fn get_requisition_items(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ItemResponse>>, AppError> {
    let items = state
        .requisition_service
        .get_requisition_items(id)
        .await?;

    Ok(Json(items.into_iter().map(Into::into).collect()))
}

/// POST /api/admin/requisitions/:req_id/items/:item_id/delete
/// Soft delete a requisition item (using POST because DELETE doesn't support body)
pub async fn delete_requisition_item(
    State(state): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    Path((_, item_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<DeleteItemRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ctx = create_audit_context(&user, &headers);

    state
        .requisition_service
        .soft_delete_item(item_id, &ctx, &payload.reason)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Item excluído com sucesso"
    })))
}

/// POST /api/admin/requisitions/:req_id/items/:item_id/restore
/// Restore a soft-deleted requisition item
pub async fn restore_requisition_item(
    State(state): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    Path((_, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ItemResponse>, AppError> {
    let ctx = create_audit_context(&user, &headers);

    let item = state
        .requisition_service
        .restore_item(item_id, &ctx)
        .await?;

    Ok(Json(item.into()))
}
