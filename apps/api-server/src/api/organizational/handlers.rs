use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::infra::state::AppState;
use application::services::organizational_service::{
    OrganizationService, OrganizationalUnitCategoryService, OrganizationalUnitService,
    OrganizationalUnitTypeService, SystemSettingsService,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::organizational::{ActivityArea, InternalUnitType};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// ============================
// Query Parameters
// ============================

#[derive(Debug, Deserialize)]
pub struct SystemSettingsListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationsListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UnitCategoriesListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub is_active: Option<bool>,
    pub is_siorg_managed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UnitTypesListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub is_active: Option<bool>,
    pub is_siorg_managed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationalUnitsListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub organization_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub unit_type_id: Option<Uuid>,
    pub activity_area: Option<ActivityArea>,
    pub internal_type: Option<InternalUnitType>,
    pub is_active: Option<bool>,
    pub is_siorg_managed: Option<bool>,
    pub search: Option<String>,
}

fn default_limit() -> i64 {
    50
}

// Helper functions to get services
fn get_settings_service(state: &AppState) -> Arc<SystemSettingsService> {
    state.system_settings_service.clone()
}

fn get_organization_service(state: &AppState) -> Arc<OrganizationService> {
    state.organization_service.clone()
}

fn get_unit_category_service(state: &AppState) -> Arc<OrganizationalUnitCategoryService> {
    state.organizational_unit_category_service.clone()
}

fn get_unit_type_service(state: &AppState) -> Arc<OrganizationalUnitTypeService> {
    state.organizational_unit_type_service.clone()
}

fn get_unit_service(state: &AppState) -> Arc<OrganizationalUnitService> {
    state.organizational_unit_service.clone()
}

// ============================
// System Settings Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/organizational/settings",
    tag = "Organization - System Settings",
    request_body = CreateSystemSettingPayload,
    responses(
        (status = 201, description = "Setting created successfully", body = SystemSettingDto),
        (status = 409, description = "Setting key already exists"),
    )
)]
pub async fn create_system_setting(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateSystemSettingPayload>,
) -> Result<(StatusCode, Json<SystemSettingDto>), (StatusCode, String)> {
    let service = get_settings_service(&state);

    service
        .create(payload)
        .await
        .map(|setting| (StatusCode::CREATED, Json(setting)))
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/settings",
    tag = "Organization - System Settings",
    params(
        ("category" = Option<String>, Query, description = "Filter by category"),
        ("limit" = i64, Query, description = "Maximum number of items to return"),
        ("offset" = i64, Query, description = "Number of items to skip"),
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = SystemSettingsListResponse),
    )
)]
pub async fn list_system_settings(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<SystemSettingsListQuery>,
) -> Result<Json<SystemSettingsListResponse>, (StatusCode, String)> {
    let service = get_settings_service(&state);

    let (settings, total) = service
        .list(query.category.as_deref(), query.limit, query.offset)
        .await
        .map_err(|e| (e.status_code(), e.to_string()))?;

    Ok(Json(SystemSettingsListResponse {
        settings,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/settings/{key}",
    tag = "Organization - System Settings",
    params(
        ("key" = String, Path, description = "Setting key"),
    ),
    responses(
        (status = 200, description = "Setting found", body = SystemSettingDto),
        (status = 404, description = "Setting not found"),
    )
)]
pub async fn get_system_setting(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<SystemSettingDto>, (StatusCode, String)> {
    let service = get_settings_service(&state);

    service
        .get(&key)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/admin/organizational/settings/{key}",
    tag = "Organization - System Settings",
    params(
        ("key" = String, Path, description = "Setting key"),
    ),
    request_body = UpdateSystemSettingPayload,
    responses(
        (status = 200, description = "Setting updated successfully", body = SystemSettingDto),
        (status = 404, description = "Setting not found"),
    )
)]
pub async fn update_system_setting(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSystemSettingPayload>,
) -> Result<Json<SystemSettingDto>, (StatusCode, String)> {
    let service = get_settings_service(&state);

    service
        .update(&key, payload, Some(user.user_id))
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/organizational/settings/{key}",
    tag = "Organization - System Settings",
    params(
        ("key" = String, Path, description = "Setting key"),
    ),
    responses(
        (status = 204, description = "Setting deleted successfully"),
        (status = 404, description = "Setting not found"),
    )
)]
pub async fn delete_system_setting(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_settings_service(&state);

    service
        .delete(&key)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

// ============================
// Organization Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/organizational/organizations",
    tag = "Organization - Organizations",
    request_body = CreateOrganizationPayload,
    responses(
        (status = 201, description = "Organization created successfully", body = OrganizationDto),
        (status = 409, description = "CNPJ or SIORG code already exists"),
    )
)]
pub async fn create_organization(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationPayload>,
) -> Result<(StatusCode, Json<OrganizationDto>), (StatusCode, String)> {
    let service = get_organization_service(&state);

    service
        .create(payload)
        .await
        .map(|org| (StatusCode::CREATED, Json(org)))
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/organizations",
    tag = "Organization - Organizations",
    params(
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("limit" = i64, Query, description = "Maximum number of items to return"),
        ("offset" = i64, Query, description = "Number of items to skip"),
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = OrganizationsListResponse),
    )
)]
pub async fn list_organizations(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<OrganizationsListQuery>,
) -> Result<Json<OrganizationsListResponse>, (StatusCode, String)> {
    let service = get_organization_service(&state);

    let (organizations, total) = service
        .list(query.is_active, query.limit, query.offset)
        .await
        .map_err(|e| (e.status_code(), e.to_string()))?;

    Ok(Json(OrganizationsListResponse {
        organizations,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/organizations/{id}",
    tag = "Organization - Organizations",
    params(
        ("id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Organization found", body = OrganizationDto),
        (status = 404, description = "Organization not found"),
    )
)]
pub async fn get_organization(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationDto>, (StatusCode, String)> {
    let service = get_organization_service(&state);

    service
        .get_by_id(id)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/admin/organizational/organizations/{id}",
    tag = "Organization - Organizations",
    params(
        ("id" = Uuid, Path, description = "Organization ID"),
    ),
    request_body = UpdateOrganizationPayload,
    responses(
        (status = 200, description = "Organization updated successfully", body = OrganizationDto),
        (status = 404, description = "Organization not found"),
    )
)]
pub async fn update_organization(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationPayload>,
) -> Result<Json<OrganizationDto>, (StatusCode, String)> {
    let service = get_organization_service(&state);

    service
        .update(id, payload)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/organizational/organizations/{id}",
    tag = "Organization - Organizations",
    params(
        ("id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 404, description = "Organization not found"),
    )
)]
pub async fn delete_organization(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_organization_service(&state);

    service
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

// ============================
// Organizational Unit Category Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/organizational/unit-categories",
    tag = "Organization - Unit Categories",
    request_body = CreateOrganizationalUnitCategoryPayload,
    responses(
        (status = 201, description = "Category created successfully", body = OrganizationalUnitCategoryDto),
        (status = 409, description = "Category name already exists"),
    )
)]
pub async fn create_unit_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationalUnitCategoryPayload>,
) -> Result<(StatusCode, Json<OrganizationalUnitCategoryDto>), (StatusCode, String)> {
    let service = get_unit_category_service(&state);

    service
        .create(payload)
        .await
        .map(|category| (StatusCode::CREATED, Json(category)))
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/unit-categories",
    tag = "Organization - Unit Categories",
    params(
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("is_siorg_managed" = Option<bool>, Query, description = "Filter by SIORG management"),
        ("limit" = i64, Query, description = "Maximum number of items to return"),
        ("offset" = i64, Query, description = "Number of items to skip"),
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = OrganizationalUnitCategoriesListResponse),
    )
)]
pub async fn list_unit_categories(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<UnitCategoriesListQuery>,
) -> Result<Json<OrganizationalUnitCategoriesListResponse>, (StatusCode, String)> {
    let service = get_unit_category_service(&state);

    let (categories, total) = service
        .list(query.is_active, query.is_siorg_managed, query.limit, query.offset)
        .await
        .map_err(|e| (e.status_code(), e.to_string()))?;

    Ok(Json(OrganizationalUnitCategoriesListResponse {
        categories,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/unit-categories/{id}",
    tag = "Organization - Unit Categories",
    params(
        ("id" = Uuid, Path, description = "Category ID"),
    ),
    responses(
        (status = 200, description = "Category found", body = OrganizationalUnitCategoryDto),
        (status = 404, description = "Category not found"),
    )
)]
pub async fn get_unit_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationalUnitCategoryDto>, (StatusCode, String)> {
    let service = get_unit_category_service(&state);

    service
        .get_by_id(id)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/admin/organizational/unit-categories/{id}",
    tag = "Organization - Unit Categories",
    params(
        ("id" = Uuid, Path, description = "Category ID"),
    ),
    request_body = UpdateOrganizationalUnitCategoryPayload,
    responses(
        (status = 200, description = "Category updated successfully", body = OrganizationalUnitCategoryDto),
        (status = 404, description = "Category not found"),
    )
)]
pub async fn update_unit_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationalUnitCategoryPayload>,
) -> Result<Json<OrganizationalUnitCategoryDto>, (StatusCode, String)> {
    let service = get_unit_category_service(&state);

    service
        .update(id, payload)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/organizational/unit-categories/{id}",
    tag = "Organization - Unit Categories",
    params(
        ("id" = Uuid, Path, description = "Category ID"),
    ),
    responses(
        (status = 204, description = "Category deleted successfully"),
        (status = 404, description = "Category not found"),
    )
)]
pub async fn delete_unit_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_unit_category_service(&state);

    service
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

// ============================
// Organizational Unit Type Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/organizational/unit-types",
    tag = "Organization - Unit Types",
    request_body = CreateOrganizationalUnitTypePayload,
    responses(
        (status = 201, description = "Type created successfully", body = OrganizationalUnitTypeDto),
        (status = 409, description = "Type code already exists"),
    )
)]
pub async fn create_unit_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationalUnitTypePayload>,
) -> Result<(StatusCode, Json<OrganizationalUnitTypeDto>), (StatusCode, String)> {
    let service = get_unit_type_service(&state);

    service
        .create(payload)
        .await
        .map(|unit_type| (StatusCode::CREATED, Json(unit_type)))
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/unit-types",
    tag = "Organization - Unit Types",
    params(
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("is_siorg_managed" = Option<bool>, Query, description = "Filter by SIORG management"),
        ("limit" = i64, Query, description = "Maximum number of items to return"),
        ("offset" = i64, Query, description = "Number of items to skip"),
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = OrganizationalUnitTypesListResponse),
    )
)]
pub async fn list_unit_types(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<UnitTypesListQuery>,
) -> Result<Json<OrganizationalUnitTypesListResponse>, (StatusCode, String)> {
    let service = get_unit_type_service(&state);

    let (types, total) = service
        .list(query.is_active, query.is_siorg_managed, query.limit, query.offset)
        .await
        .map_err(|e| (e.status_code(), e.to_string()))?;

    Ok(Json(OrganizationalUnitTypesListResponse {
        types,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/unit-types/{id}",
    tag = "Organization - Unit Types",
    params(
        ("id" = Uuid, Path, description = "Type ID"),
    ),
    responses(
        (status = 200, description = "Type found", body = OrganizationalUnitTypeDto),
        (status = 404, description = "Type not found"),
    )
)]
pub async fn get_unit_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationalUnitTypeDto>, (StatusCode, String)> {
    let service = get_unit_type_service(&state);

    service
        .get_by_id(id)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/admin/organizational/unit-types/{id}",
    tag = "Organization - Unit Types",
    params(
        ("id" = Uuid, Path, description = "Type ID"),
    ),
    request_body = UpdateOrganizationalUnitTypePayload,
    responses(
        (status = 200, description = "Type updated successfully", body = OrganizationalUnitTypeDto),
        (status = 404, description = "Type not found"),
    )
)]
pub async fn update_unit_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationalUnitTypePayload>,
) -> Result<Json<OrganizationalUnitTypeDto>, (StatusCode, String)> {
    let service = get_unit_type_service(&state);

    service
        .update(id, payload)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/organizational/unit-types/{id}",
    tag = "Organization - Unit Types",
    params(
        ("id" = Uuid, Path, description = "Type ID"),
    ),
    responses(
        (status = 204, description = "Type deleted successfully"),
        (status = 404, description = "Type not found"),
    )
)]
pub async fn delete_unit_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_unit_type_service(&state);

    service
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

// ============================
// Organizational Unit Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/organizational/units",
    tag = "Organization - Organizational Units",
    request_body = CreateOrganizationalUnitPayload,
    responses(
        (status = 201, description = "Unit created successfully", body = OrganizationalUnitDto),
        (status = 400, description = "Invalid parent or circular reference"),
        (status = 409, description = "SIORG code already exists"),
    )
)]
pub async fn create_organizational_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationalUnitPayload>,
) -> Result<(StatusCode, Json<OrganizationalUnitDto>), (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .create(payload)
        .await
        .map(|unit| (StatusCode::CREATED, Json(unit)))
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/units",
    tag = "Organization - Organizational Units",
    params(
        ("organization_id" = Option<Uuid>, Query, description = "Filter by organization"),
        ("parent_id" = Option<Uuid>, Query, description = "Filter by parent unit"),
        ("category_id" = Option<Uuid>, Query, description = "Filter by category"),
        ("unit_type_id" = Option<Uuid>, Query, description = "Filter by type"),
        ("activity_area" = Option<ActivityArea>, Query, description = "Filter by activity area"),
        ("internal_type" = Option<InternalUnitType>, Query, description = "Filter by internal type"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("is_siorg_managed" = Option<bool>, Query, description = "Filter by SIORG management"),
        ("search" = Option<String>, Query, description = "Search by name"),
        ("limit" = i64, Query, description = "Maximum number of items to return"),
        ("offset" = i64, Query, description = "Number of items to skip"),
    ),
    responses(
        (status = 200, description = "List retrieved successfully", body = OrganizationalUnitsListResponse),
    )
)]
pub async fn list_organizational_units(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<OrganizationalUnitsListQuery>,
) -> Result<Json<OrganizationalUnitsListResponse>, (StatusCode, String)> {
    let service = get_unit_service(&state);

    let (units, total) = service
        .list(
            query.organization_id,
            query.parent_id,
            query.category_id,
            query.unit_type_id,
            query.activity_area,
            query.internal_type,
            query.is_active,
            query.is_siorg_managed,
            query.search.as_deref(),
            query.limit,
            query.offset,
        )
        .await
        .map_err(|e| (e.status_code(), e.to_string()))?;

    Ok(Json(OrganizationalUnitsListResponse {
        units,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/units/tree",
    tag = "Organization - Organizational Units",
    params(
        ("organization_id" = Option<Uuid>, Query, description = "Filter by organization"),
    ),
    responses(
        (status = 200, description = "Tree retrieved successfully", body = Vec<OrganizationalUnitTreeNode>),
    )
)]
pub async fn get_organizational_units_tree(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<OrganizationsListQuery>,
) -> Result<Json<Vec<OrganizationalUnitTreeNode>>, (StatusCode, String)> {
    let service = get_unit_service(&state);

    // Reusing organization_id from the query (if provided)
    service
        .get_tree(None)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/units/{id}",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Unit ID"),
    ),
    responses(
        (status = 200, description = "Unit found", body = OrganizationalUnitWithDetailsDto),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn get_organizational_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationalUnitWithDetailsDto>, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .get_by_id_with_details(id)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/units/{id}/children",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Parent unit ID"),
    ),
    responses(
        (status = 200, description = "Children retrieved successfully", body = Vec<OrganizationalUnitDto>),
    )
)]
pub async fn get_organizational_unit_children(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<OrganizationalUnitDto>>, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .get_children(id)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/units/{id}/path",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Unit ID"),
    ),
    responses(
        (status = 200, description = "Path to root retrieved successfully", body = Vec<OrganizationalUnitDto>),
    )
)]
pub async fn get_organizational_unit_path(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<OrganizationalUnitDto>>, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .get_path_to_root(id)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/admin/organizational/units/{id}",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Unit ID"),
    ),
    request_body = UpdateOrganizationalUnitPayload,
    responses(
        (status = 200, description = "Unit updated successfully", body = OrganizationalUnitDto),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn update_organizational_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationalUnitPayload>,
) -> Result<Json<OrganizationalUnitDto>, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .update(id, payload)
        .await
        .map(Json)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/organizational/units/{id}",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Unit ID"),
    ),
    responses(
        (status = 204, description = "Unit deleted successfully"),
        (status = 404, description = "Unit not found"),
        (status = 400, description = "Unit has children and cannot be deleted"),
    )
)]
pub async fn delete_organizational_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/admin/organizational/units/{id}/deactivate",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Unit ID"),
    ),
    responses(
        (status = 204, description = "Unit deactivated successfully"),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn deactivate_organizational_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(reason): Json<Option<String>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .deactivate(id, reason)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/admin/organizational/units/{id}/activate",
    tag = "Organization - Organizational Units",
    params(
        ("id" = Uuid, Path, description = "Unit ID"),
    ),
    responses(
        (status = 204, description = "Unit activated successfully"),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn activate_organizational_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_unit_service(&state);

    service
        .activate(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (e.status_code(), e.to_string()))
}

// ============================
// SIORG Sync Handlers
// ============================

#[derive(Debug, Deserialize)]
pub struct SyncOrganizationRequest {
    pub siorg_code: i32,
}

#[derive(Debug, Deserialize)]
pub struct SyncUnitRequest {
    pub siorg_code: i32,
}

#[derive(Debug, Deserialize)]
pub struct SyncOrgUnitsRequest {
    pub org_siorg_code: i32,
}

#[utoipa::path(
    post,
    path = "/api/admin/organizational/sync/organization",
    tag = "Organization - SIORG Sync",
    request_body = SyncOrganizationRequest,
    responses(
        (status = 200, description = "Organization synced successfully", body = OrganizationDto),
        (status = 404, description = "Organization not found in SIORG"),
        (status = 500, description = "Sync failed"),
    )
)]
pub async fn sync_organization(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<SyncOrganizationRequest>,
) -> Result<Json<OrganizationDto>, (StatusCode, String)> {
    let sync_service = state.siorg_sync_service.clone();

    sync_service
        .sync_organization(payload.siorg_code)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/admin/organizational/sync/unit",
    tag = "Organization - SIORG Sync",
    request_body = SyncUnitRequest,
    responses(
        (status = 200, description = "Unit synced successfully", body = OrganizationalUnitDto),
        (status = 404, description = "Unit not found in SIORG"),
        (status = 500, description = "Sync failed"),
    )
)]
pub async fn sync_unit(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<SyncUnitRequest>,
) -> Result<Json<OrganizationalUnitDto>, (StatusCode, String)> {
    let sync_service = state.siorg_sync_service.clone();

    sync_service
        .sync_unit(payload.siorg_code)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/admin/organizational/sync/org-units",
    tag = "Organization - SIORG Sync",
    request_body = SyncOrgUnitsRequest,
    responses(
        (status = 200, description = "Bulk sync completed", body = String),
        (status = 500, description = "Sync failed"),
    )
)]
pub async fn sync_organization_units(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<SyncOrgUnitsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let sync_service = state.siorg_sync_service.clone();

    let summary = sync_service
        .sync_organization_units(payload.org_siorg_code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "total_processed": summary.total_processed,
        "created": summary.created,
        "updated": summary.updated,
        "failed": summary.failed,
        "errors": summary.errors
    })))
}

#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/health",
    tag = "Organization - SIORG Sync",
    responses(
        (status = 200, description = "SIORG API is healthy"),
        (status = 503, description = "SIORG API is unavailable"),
    )
)]
pub async fn check_siorg_health(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let sync_service = state.siorg_sync_service.clone();

    let is_healthy = sync_service
        .check_health()
        .await
        .unwrap_or(false);

    if is_healthy {
        Ok(Json(json!({
            "status": "healthy",
            "siorg_api": "available"
        })))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "SIORG API is unavailable".to_string(),
        ))
    }
}
