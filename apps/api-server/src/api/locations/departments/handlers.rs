use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    CreateDepartmentCategoryPayload, ListDepartmentCategoriesQuery,
    PaginatedDepartmentCategories, UpdateDepartmentCategoryPayload,
};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::DepartmentCategoryResponse;

// ============================
// Department Category Handlers
// ============================

/// GET /admin/locations/department-categories
pub async fn list_department_categories(
    State(state): State<AppState>,
    Query(params): Query<ListDepartmentCategoriesQuery>,
) -> Result<Json<PaginatedDepartmentCategories>, AppError> {
    let result = state
        .location_service
        .list_department_categories(params.limit, params.offset, params.search)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/department-categories/:id
pub async fn get_department_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DepartmentCategoryResponse>, AppError> {
    let department_category_dto = state.location_service.get_department_category(id).await?;

    Ok(Json(DepartmentCategoryResponse {
        id: department_category_dto.id,
        name: department_category_dto.name,
        description: department_category_dto.description,
        created_at: department_category_dto.created_at,
        updated_at: department_category_dto.updated_at,
    }))
}

/// POST /admin/locations/department-categories
pub async fn create_department_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateDepartmentCategoryPayload>,
) -> Result<(StatusCode, Json<DepartmentCategoryResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let department_category_dto = state
        .location_service
        .create_department_category(payload)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(DepartmentCategoryResponse {
            id: department_category_dto.id,
            name: department_category_dto.name,
            description: department_category_dto.description,
            created_at: department_category_dto.created_at,
            updated_at: department_category_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/department-categories/:id
pub async fn update_department_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDepartmentCategoryPayload>,
) -> Result<Json<DepartmentCategoryResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let department_category_dto = state
        .location_service
        .update_department_category(id, payload)
        .await?;

    Ok(Json(DepartmentCategoryResponse {
        id: department_category_dto.id,
        name: department_category_dto.name,
        description: department_category_dto.description,
        created_at: department_category_dto.created_at,
        updated_at: department_category_dto.updated_at,
    }))
}

/// DELETE /admin/locations/department-categories/:id
pub async fn delete_department_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .location_service
        .delete_department_category(id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

