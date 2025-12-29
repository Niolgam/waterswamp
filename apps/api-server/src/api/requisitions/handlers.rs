use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    CreateRequisitionItemPayload, FulfillRequisitionItemPayload, RequisitionStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

// ============================
// Request Contracts
// ============================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRequisitionRequest {
    pub warehouse_id: Uuid,
    #[validate(length(min = 1))]
    pub items: Vec<CreateRequisitionItemPayload>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ApproveRequisitionRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RejectRequisitionRequest {
    #[validate(length(min = 10, max = 1000))]
    pub rejection_reason: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FulfillRequisitionRequest {
    pub items: Vec<FulfillRequisitionItemPayload>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRequisitionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub warehouse_id: Option<Uuid>,
    pub requester_id: Option<Uuid>,
    pub status: Option<RequisitionStatus>,
}

// ============================
// Requisition Handlers
// ============================

/// POST /api/requisitions
pub async fn create_requisition(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreateRequisitionRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    req.validate()?;

    let (requisition, items) = state
        .requisition_workflow_service
        .create_requisition(req.warehouse_id, user.id, req.items, req.notes)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "requisition": requisition,
            "items": items,
            "message": "Requisição criada com sucesso"
        })),
    ))
}

/// GET /api/requisitions/:id
pub async fn get_requisition(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let requisition = state
        .requisition_workflow_service
        .get_requisition_with_details(id)
        .await?;

    Ok(Json(json!(requisition)))
}

/// GET /api/requisitions
pub async fn list_requisitions(
    State(state): State<AppState>,
    Query(params): Query<ListRequisitionsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let (requisitions, total) = state
        .requisition_workflow_service
        .list_requisitions(
            limit,
            offset,
            params.warehouse_id,
            params.requester_id,
            params.status,
            None,
            None,
        )
        .await?;

    Ok(Json(json!({
        "requisitions": requisitions,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

/// POST /api/requisitions/:id/approve
pub async fn approve_requisition(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveRequisitionRequest>,
) -> Result<Json<Value>, AppError> {
    let requisition = state
        .requisition_workflow_service
        .approve_requisition(id, user.id, req.notes)
        .await?;

    Ok(Json(json!({
        "requisition": requisition,
        "message": "Requisição aprovada com sucesso"
    })))
}

/// POST /api/requisitions/:id/reject
pub async fn reject_requisition(
    State(state): State<AppState>,
    CurrentUser(_user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<RejectRequisitionRequest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;

    let requisition = state
        .requisition_workflow_service
        .reject_requisition(id, req.rejection_reason)
        .await?;

    Ok(Json(json!({
        "requisition": requisition,
        "message": "Requisição rejeitada"
    })))
}

/// POST /api/requisitions/:id/fulfill
pub async fn fulfill_requisition(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<FulfillRequisitionRequest>,
) -> Result<Json<Value>, AppError> {
    let requisition = state
        .requisition_workflow_service
        .fulfill_requisition(id, user.id, req.items, req.notes)
        .await?;

    Ok(Json(json!({
        "requisition": requisition,
        "message": "Requisição atendida com sucesso"
    })))
}

/// DELETE /api/requisitions/:id
pub async fn cancel_requisition(
    State(state): State<AppState>,
    CurrentUser(_user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let requisition = state
        .requisition_workflow_service
        .cancel_requisition(id)
        .await?;

    Ok(Json(json!({
        "requisition": requisition,
        "message": "Requisição cancelada"
    })))
}
