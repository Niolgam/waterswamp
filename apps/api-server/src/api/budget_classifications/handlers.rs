use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    BudgetClassificationTreeNode, CreateBudgetClassificationPayload,
    ListBudgetClassificationsQuery, PaginatedBudgetClassifications,
    UpdateBudgetClassificationPayload,
};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{BudgetClassificationResponse, BudgetClassificationWithParentResponse};

// ============================
// Handlers
// ============================

/// GET /admin/budget-classifications
#[utoipa::path(
    get,
    path = "/api/v1/admin/budget-classifications",
    tag = "Admin",
    params(
        ("limit" = Option<i64>, Query, description = "Limite de resultados por página"),
        ("offset" = Option<i64>, Query, description = "Offset para paginação"),
        ("search" = Option<String>, Query, description = "Termo de busca"),
        ("parent_id" = Option<Uuid>, Query, description = "Filtrar por ID do pai"),
        ("level" = Option<i32>, Query, description = "Filtrar por nível (1-5)"),
        ("is_active" = Option<bool>, Query, description = "Filtrar por status ativo")
    ),
    responses(
        (status = 200, description = "Lista de classificações orçamentárias", body = PaginatedBudgetClassifications),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_budget_classifications(
    State(state): State<AppState>,
    Query(params): Query<ListBudgetClassificationsQuery>,
) -> Result<Json<PaginatedBudgetClassifications>, AppError> {
    let result = state
        .budget_classifications_service
        .list(
            params.limit,
            params.offset,
            params.search,
            params.parent_id,
            params.level,
            params.is_active,
        )
        .await?;

    Ok(Json(result))
}

/// GET /admin/budget-classifications/tree
#[utoipa::path(
    get,
    path = "/api/v1/admin/budget-classifications/tree",
    tag = "Admin",
    responses(
        (status = 200, description = "Árvore hierárquica de classificações", body = Vec<BudgetClassificationTreeNode>),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_tree(
    State(state): State<AppState>,
) -> Result<Json<Vec<BudgetClassificationTreeNode>>, AppError> {
    let tree = state.budget_classifications_service.get_tree().await?;
    Ok(Json(tree))
}

/// GET /admin/budget-classifications/:id
#[utoipa::path(
    get,
    path = "/api/v1/admin/budget-classifications/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID da classificação orçamentária")
    ),
    responses(
        (status = 200, description = "Classificação encontrada", body = BudgetClassificationWithParentResponse),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Classificação não encontrada")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_budget_classification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BudgetClassificationWithParentResponse>, AppError> {
    let dto = state.budget_classifications_service.get(id).await?;

    Ok(Json(BudgetClassificationWithParentResponse {
        id: dto.id,
        parent_id: dto.parent_id,
        code_part: dto.code_part,
        full_code: dto.full_code,
        name: dto.name,
        level: dto.level,
        is_active: dto.is_active,
        parent_name: dto.parent_name,
        parent_full_code: dto.parent_full_code,
        created_at: dto.created_at,
        updated_at: dto.updated_at,
    }))
}

/// POST /admin/budget-classifications
#[utoipa::path(
    post,
    path = "/api/v1/admin/budget-classifications",
    tag = "Admin",
    request_body = CreateBudgetClassificationPayload,
    responses(
        (status = 201, description = "Classificação criada com sucesso", body = BudgetClassificationResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_budget_classification(
    State(state): State<AppState>,
    Json(payload): Json<CreateBudgetClassificationPayload>,
) -> Result<(StatusCode, Json<BudgetClassificationResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let dto = state.budget_classifications_service.create(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(BudgetClassificationResponse {
            id: dto.id,
            parent_id: dto.parent_id,
            code_part: dto.code_part,
            full_code: dto.full_code,
            name: dto.name,
            level: dto.level,
            is_active: dto.is_active,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
        }),
    ))
}

/// PUT /admin/budget-classifications/:id
#[utoipa::path(
    put,
    path = "/api/v1/admin/budget-classifications/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID da classificação orçamentária")
    ),
    request_body = UpdateBudgetClassificationPayload,
    responses(
        (status = 200, description = "Classificação atualizada com sucesso", body = BudgetClassificationResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Classificação não encontrada")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_budget_classification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBudgetClassificationPayload>,
) -> Result<Json<BudgetClassificationResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let dto = state
        .budget_classifications_service
        .update(id, payload)
        .await?;

    Ok(Json(BudgetClassificationResponse {
        id: dto.id,
        parent_id: dto.parent_id,
        code_part: dto.code_part,
        full_code: dto.full_code,
        name: dto.name,
        level: dto.level,
        is_active: dto.is_active,
        created_at: dto.created_at,
        updated_at: dto.updated_at,
    }))
}

/// DELETE /admin/budget-classifications/:id
#[utoipa::path(
    delete,
    path = "/api/v1/admin/budget-classifications/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID da classificação orçamentária")
    ),
    responses(
        (status = 204, description = "Classificação deletada com sucesso"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Classificação não encontrada"),
        (status = 409, description = "Classificação possui filhos")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_budget_classification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.budget_classifications_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
