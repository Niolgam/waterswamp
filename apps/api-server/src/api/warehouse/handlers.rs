use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    CreateMaterialGroupPayload, CreateMaterialPayload, ListMaterialGroupsQuery,
    ListMaterialsQuery, PaginatedMaterialGroups, PaginatedMaterials,
    UpdateMaterialGroupPayload, UpdateMaterialPayload,
};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{MaterialGroupResponse, MaterialResponse, MaterialWithGroupResponse};

// ============================
// Material Group Handlers
// ============================

/// GET /admin/warehouse/material-groups
pub async fn list_material_groups(
    State(state): State<AppState>,
    Query(params): Query<ListMaterialGroupsQuery>,
) -> Result<Json<PaginatedMaterialGroups>, AppError> {
    let result = state
        .warehouse_service
        .list_material_groups(
            params.limit.unwrap_or(10),
            params.offset.unwrap_or(0),
            params.search,
            params.is_personnel_exclusive,
            params.is_active,
        )
        .await?;

    Ok(Json(result))
}

/// GET /admin/warehouse/material-groups/:id
pub async fn get_material_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MaterialGroupResponse>, AppError> {
    let material_group_dto = state.warehouse_service.get_material_group(id).await?;

    Ok(Json(MaterialGroupResponse {
        id: material_group_dto.id,
        code: material_group_dto.code,
        name: material_group_dto.name,
        description: material_group_dto.description,
        expense_element: material_group_dto.expense_element,
        is_personnel_exclusive: material_group_dto.is_personnel_exclusive,
        is_active: material_group_dto.is_active,
        created_at: material_group_dto.created_at,
        updated_at: material_group_dto.updated_at,
    }))
}

/// POST /admin/warehouse/material-groups
pub async fn create_material_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateMaterialGroupPayload>,
) -> Result<(StatusCode, Json<MaterialGroupResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let material_group_dto = state.warehouse_service.create_material_group(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(MaterialGroupResponse {
            id: material_group_dto.id,
            code: material_group_dto.code,
            name: material_group_dto.name,
            description: material_group_dto.description,
            expense_element: material_group_dto.expense_element,
            is_personnel_exclusive: material_group_dto.is_personnel_exclusive,
            is_active: material_group_dto.is_active,
            created_at: material_group_dto.created_at,
            updated_at: material_group_dto.updated_at,
        }),
    ))
}

/// PUT /admin/warehouse/material-groups/:id
pub async fn update_material_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMaterialGroupPayload>,
) -> Result<Json<MaterialGroupResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let material_group_dto = state
        .warehouse_service
        .update_material_group(id, payload)
        .await?;

    Ok(Json(MaterialGroupResponse {
        id: material_group_dto.id,
        code: material_group_dto.code,
        name: material_group_dto.name,
        description: material_group_dto.description,
        expense_element: material_group_dto.expense_element,
        is_personnel_exclusive: material_group_dto.is_personnel_exclusive,
        is_active: material_group_dto.is_active,
        created_at: material_group_dto.created_at,
        updated_at: material_group_dto.updated_at,
    }))
}

/// DELETE /admin/warehouse/material-groups/:id
pub async fn delete_material_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.warehouse_service.delete_material_group(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Material Handlers
// ============================

/// GET /admin/warehouse/materials
pub async fn list_materials(
    State(state): State<AppState>,
    Query(params): Query<ListMaterialsQuery>,
) -> Result<Json<PaginatedMaterials>, AppError> {
    let result = state
        .warehouse_service
        .list_materials(
            params.limit.unwrap_or(10),
            params.offset.unwrap_or(0),
            params.search,
            params.material_group_id,
            params.is_active,
        )
        .await?;

    Ok(Json(result))
}

/// GET /admin/warehouse/materials/:id
pub async fn get_material(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MaterialWithGroupResponse>, AppError> {
    let material_dto = state.warehouse_service.get_material_with_group(id).await?;

    Ok(Json(MaterialWithGroupResponse {
        id: material_dto.id,
        material_group_id: material_dto.material_group_id,
        material_group_code: material_dto.material_group_code,
        material_group_name: material_dto.material_group_name,
        name: material_dto.name,
        estimated_value: material_dto.estimated_value,
        unit_of_measure: material_dto.unit_of_measure,
        specification: material_dto.specification,
        search_links: material_dto.search_links,
        catmat_code: material_dto.catmat_code,
        photo_url: material_dto.photo_url,
        is_active: material_dto.is_active,
        created_at: material_dto.created_at,
        updated_at: material_dto.updated_at,
    }))
}

/// POST /admin/warehouse/materials
pub async fn create_material(
    State(state): State<AppState>,
    Json(payload): Json<CreateMaterialPayload>,
) -> Result<(StatusCode, Json<MaterialResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let material_dto = state.warehouse_service.create_material(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(MaterialResponse {
            id: material_dto.id,
            material_group_id: material_dto.material_group_id,
            name: material_dto.name,
            estimated_value: material_dto.estimated_value,
            unit_of_measure: material_dto.unit_of_measure,
            specification: material_dto.specification,
            search_links: material_dto.search_links,
            catmat_code: material_dto.catmat_code,
            photo_url: material_dto.photo_url,
            is_active: material_dto.is_active,
            created_at: material_dto.created_at,
            updated_at: material_dto.updated_at,
        }),
    ))
}

/// PUT /admin/warehouse/materials/:id
pub async fn update_material(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMaterialPayload>,
) -> Result<Json<MaterialResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let material_dto = state.warehouse_service.update_material(id, payload).await?;

    Ok(Json(MaterialResponse {
        id: material_dto.id,
        material_group_id: material_dto.material_group_id,
        name: material_dto.name,
        estimated_value: material_dto.estimated_value,
        unit_of_measure: material_dto.unit_of_measure,
        specification: material_dto.specification,
        search_links: material_dto.search_links,
        catmat_code: material_dto.catmat_code,
        photo_url: material_dto.photo_url,
        is_active: material_dto.is_active,
        created_at: material_dto.created_at,
        updated_at: material_dto.updated_at,
    }))
}

/// DELETE /admin/warehouse/materials/:id
pub async fn delete_material(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.warehouse_service.delete_material(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
