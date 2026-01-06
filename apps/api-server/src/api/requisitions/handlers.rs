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
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

// ============================
// Request Contracts
// ============================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRequisitionRequest {
    pub warehouse_id: Uuid,
    #[validate(length(min = 1))]
    pub items: Vec<CreateRequisitionItemPayload>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ApproveRequisitionRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RejectRequisitionRequest {
    #[validate(length(min = 10, max = 1000))]
    pub rejection_reason: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct FulfillRequisitionRequest {
    pub items: Vec<FulfillRequisitionItemPayload>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
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
#[utoipa::path(
    post,
    path = "/api/requisitions",
    tag = "Requisitions",
    request_body = CreateRequisitionRequest,
    responses(
        (status = 201, description = "Requisição criada com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Almoxarifado ou material não encontrado"),
    ),
    summary = "Criar nova requisição de materiais",
    security(("bearer" = []))
)]
pub async fn create_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
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
#[utoipa::path(
    get,
    path = "/api/requisitions/{id}",
    tag = "Requisitions",
    params(
        ("id" = Uuid, Path, description = "ID da requisição"),
    ),
    responses(
        (status = 200, description = "Requisição encontrada"),
        (status = 404, description = "Requisição não encontrada"),
    ),
    summary = "Obter requisição por ID",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    get,
    path = "/api/requisitions",
    tag = "Requisitions",
    params(
        ("limit" = Option<i64>, Query, description = "Número de itens por página", example = 20),
        ("offset" = Option<i64>, Query, description = "Deslocamento para paginação", example = 0),
        ("warehouse_id" = Option<Uuid>, Query, description = "Filtrar por almoxarifado"),
        ("requester_id" = Option<Uuid>, Query, description = "Filtrar por solicitante"),
        ("status" = Option<String>, Query, description = "Filtrar por status (pending, approved, rejected, fulfilled, cancelled)"),
    ),
    responses(
        (status = 200, description = "Lista de requisições retornada com sucesso"),
    ),
    summary = "Listar requisições com filtros",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    post,
    path = "/api/requisitions/{id}/approve",
    tag = "Requisitions",
    params(
        ("id" = Uuid, Path, description = "ID da requisição"),
    ),
    request_body = ApproveRequisitionRequest,
    responses(
        (status = 200, description = "Requisição aprovada com sucesso"),
        (status = 404, description = "Requisição não encontrada"),
        (status = 409, description = "Requisição não está pendente"),
    ),
    summary = "Aprovar requisição pendente",
    security(("bearer" = []))
)]
pub async fn approve_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
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
#[utoipa::path(
    post,
    path = "/api/requisitions/{id}/reject",
    tag = "Requisitions",
    params(
        ("id" = Uuid, Path, description = "ID da requisição"),
    ),
    request_body = RejectRequisitionRequest,
    responses(
        (status = 200, description = "Requisição rejeitada"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Requisição não encontrada"),
        (status = 409, description = "Requisição não está pendente"),
    ),
    summary = "Rejeitar requisição pendente",
    security(("bearer" = []))
)]
pub async fn reject_requisition(
    State(state): State<AppState>,
    _user: CurrentUser,
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
#[utoipa::path(
    post,
    path = "/api/requisitions/{id}/fulfill",
    tag = "Requisitions",
    params(
        ("id" = Uuid, Path, description = "ID da requisição"),
    ),
    request_body = FulfillRequisitionRequest,
    responses(
        (status = 200, description = "Requisição atendida com sucesso"),
        (status = 400, description = "Dados inválidos ou estoque insuficiente"),
        (status = 404, description = "Requisição não encontrada"),
        (status = 409, description = "Requisição não está aprovada"),
    ),
    summary = "Atender requisição aprovada (registra saídas de estoque)",
    security(("bearer" = []))
)]
pub async fn fulfill_requisition(
    State(state): State<AppState>,
    user: CurrentUser,
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
#[utoipa::path(
    delete,
    path = "/api/requisitions/{id}",
    tag = "Requisitions",
    params(
        ("id" = Uuid, Path, description = "ID da requisição"),
    ),
    responses(
        (status = 200, description = "Requisição cancelada"),
        (status = 404, description = "Requisição não encontrada"),
        (status = 409, description = "Requisição não pode ser cancelada (já foi aprovada ou rejeitada)"),
    ),
    summary = "Cancelar requisição pendente",
    security(("bearer" = []))
)]
pub async fn cancel_requisition(
    State(state): State<AppState>,
    _user: CurrentUser,
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
