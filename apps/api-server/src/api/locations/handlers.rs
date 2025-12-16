use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    CreateBuildingTypePayload, CreateCityPayload, CreateDepartmentCategoryPayload,
    CreateSiteTypePayload, CreateSpaceTypePayload, CreateStatePayload, ListBuildingTypesQuery,
    ListCitiesQuery, ListDepartmentCategoriesQuery, ListSiteTypesQuery, ListSpaceTypesQuery,
    ListStatesQuery, PaginatedBuildingTypes, PaginatedCities, PaginatedDepartmentCategories,
    PaginatedSiteTypes, PaginatedSpaceTypes, PaginatedStates, UpdateBuildingTypePayload,
    UpdateCityPayload, UpdateDepartmentCategoryPayload, UpdateSiteTypePayload,
    UpdateSpaceTypePayload, UpdateStatePayload,
};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    BuildingTypeResponse, CityResponse, CityWithStateResponse, DepartmentCategoryResponse,
    SiteTypeResponse, SpaceTypeResponse, StateResponse,
};

// ============================
// State Handlers
// ============================

/// GET /admin/locations/states
pub async fn list_states(
    State(state): State<AppState>,
    Query(params): Query<ListStatesQuery>,
) -> Result<Json<PaginatedStates>, AppError> {
    let result = state
        .location_service
        .list_states(params.limit, params.offset, params.search)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/states/:id
pub async fn get_state(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<StateResponse>, AppError> {
    let state_dto = state.location_service.get_state(id).await?;

    Ok(Json(StateResponse {
        id: state_dto.id,
        name: state_dto.name,
        code: state_dto.code,
        created_at: state_dto.created_at,
        updated_at: state_dto.updated_at,
    }))
}

/// POST /admin/locations/states
pub async fn create_state(
    State(state): State<AppState>,
    Json(payload): Json<CreateStatePayload>,
) -> Result<(StatusCode, Json<StateResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let state_dto = state.location_service.create_state(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(StateResponse {
            id: state_dto.id,
            name: state_dto.name,
            code: state_dto.code,
            created_at: state_dto.created_at,
            updated_at: state_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/states/:id
pub async fn update_state(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatePayload>,
) -> Result<Json<StateResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let state_dto = state.location_service.update_state(id, payload).await?;

    Ok(Json(StateResponse {
        id: state_dto.id,
        name: state_dto.name,
        code: state_dto.code,
        created_at: state_dto.created_at,
        updated_at: state_dto.updated_at,
    }))
}

/// DELETE /admin/locations/states/:id
pub async fn delete_state(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_state(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// City Handlers
// ============================

/// GET /admin/locations/cities
pub async fn list_cities(
    State(state): State<AppState>,
    Query(params): Query<ListCitiesQuery>,
) -> Result<Json<PaginatedCities>, AppError> {
    let result = state
        .location_service
        .list_cities(params.limit, params.offset, params.search, params.state_id)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/cities/:id
pub async fn get_city(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CityWithStateResponse>, AppError> {
    let city_dto = state.location_service.get_city(id).await?;

    Ok(Json(CityWithStateResponse {
        id: city_dto.id,
        name: city_dto.name,
        state_id: city_dto.state_id,
        state_name: city_dto.state_name,
        state_code: city_dto.state_code,
        created_at: city_dto.created_at,
        updated_at: city_dto.updated_at,
    }))
}

/// POST /admin/locations/cities
pub async fn create_city(
    State(state): State<AppState>,
    Json(payload): Json<CreateCityPayload>,
) -> Result<(StatusCode, Json<CityResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let city_dto = state.location_service.create_city(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(CityResponse {
            id: city_dto.id,
            name: city_dto.name,
            state_id: city_dto.state_id,
            created_at: city_dto.created_at,
            updated_at: city_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/cities/:id
pub async fn update_city(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCityPayload>,
) -> Result<Json<CityResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let city_dto = state.location_service.update_city(id, payload).await?;

    Ok(Json(CityResponse {
        id: city_dto.id,
        name: city_dto.name,
        state_id: city_dto.state_id,
        created_at: city_dto.created_at,
        updated_at: city_dto.updated_at,
    }))
}

/// DELETE /admin/locations/cities/:id
pub async fn delete_city(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_city(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Site Type Handlers
// ============================

/// GET /admin/locations/site-types
pub async fn list_site_types(
    State(state): State<AppState>,
    Query(params): Query<ListSiteTypesQuery>,
) -> Result<Json<PaginatedSiteTypes>, AppError> {
    let result = state
        .location_service
        .list_site_types(params.limit, params.offset, params.search)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/site-types/:id
pub async fn get_site_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SiteTypeResponse>, AppError> {
    let site_type_dto = state.location_service.get_site_type(id).await?;

    Ok(Json(SiteTypeResponse {
        id: site_type_dto.id,
        name: site_type_dto.name,
        description: site_type_dto.description,
        created_at: site_type_dto.created_at,
        updated_at: site_type_dto.updated_at,
    }))
}

/// POST /admin/locations/site-types
pub async fn create_site_type(
    State(state): State<AppState>,
    Json(payload): Json<CreateSiteTypePayload>,
) -> Result<(StatusCode, Json<SiteTypeResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let site_type_dto = state.location_service.create_site_type(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(SiteTypeResponse {
            id: site_type_dto.id,
            name: site_type_dto.name,
            description: site_type_dto.description,
            created_at: site_type_dto.created_at,
            updated_at: site_type_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/site-types/:id
pub async fn update_site_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSiteTypePayload>,
) -> Result<Json<SiteTypeResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let site_type_dto = state.location_service.update_site_type(id, payload).await?;

    Ok(Json(SiteTypeResponse {
        id: site_type_dto.id,
        name: site_type_dto.name,
        description: site_type_dto.description,
        created_at: site_type_dto.created_at,
        updated_at: site_type_dto.updated_at,
    }))
}

/// DELETE /admin/locations/site-types/:id
pub async fn delete_site_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_site_type(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Building Type Handlers
// ============================

/// GET /admin/locations/building-types
pub async fn list_building_types(
    State(state): State<AppState>,
    Query(params): Query<ListBuildingTypesQuery>,
) -> Result<Json<PaginatedBuildingTypes>, AppError> {
    let result = state
        .location_service
        .list_building_types(params.limit, params.offset, params.search)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/building-types/:id
pub async fn get_building_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildingTypeResponse>, AppError> {
    let building_type_dto = state.location_service.get_building_type(id).await?;

    Ok(Json(BuildingTypeResponse {
        id: building_type_dto.id,
        name: building_type_dto.name,
        description: building_type_dto.description,
        created_at: building_type_dto.created_at,
        updated_at: building_type_dto.updated_at,
    }))
}

/// POST /admin/locations/building-types
pub async fn create_building_type(
    State(state): State<AppState>,
    Json(payload): Json<CreateBuildingTypePayload>,
) -> Result<(StatusCode, Json<BuildingTypeResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let building_type_dto = state.location_service.create_building_type(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(BuildingTypeResponse {
            id: building_type_dto.id,
            name: building_type_dto.name,
            description: building_type_dto.description,
            created_at: building_type_dto.created_at,
            updated_at: building_type_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/building-types/:id
pub async fn update_building_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBuildingTypePayload>,
) -> Result<Json<BuildingTypeResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let building_type_dto = state.location_service.update_building_type(id, payload).await?;

    Ok(Json(BuildingTypeResponse {
        id: building_type_dto.id,
        name: building_type_dto.name,
        description: building_type_dto.description,
        created_at: building_type_dto.created_at,
        updated_at: building_type_dto.updated_at,
    }))
}

/// DELETE /admin/locations/building-types/:id
pub async fn delete_building_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_building_type(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Space Type Handlers
// ============================

/// GET /admin/locations/space-types
pub async fn list_space_types(
    State(state): State<AppState>,
    Query(params): Query<ListSpaceTypesQuery>,
) -> Result<Json<PaginatedSpaceTypes>, AppError> {
    let result = state
        .location_service
        .list_space_types(params.limit, params.offset, params.search)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/space-types/:id
pub async fn get_space_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SpaceTypeResponse>, AppError> {
    let space_type_dto = state.location_service.get_space_type(id).await?;

    Ok(Json(SpaceTypeResponse {
        id: space_type_dto.id,
        name: space_type_dto.name,
        description: space_type_dto.description,
        created_at: space_type_dto.created_at,
        updated_at: space_type_dto.updated_at,
    }))
}

/// POST /admin/locations/space-types
pub async fn create_space_type(
    State(state): State<AppState>,
    Json(payload): Json<CreateSpaceTypePayload>,
) -> Result<(StatusCode, Json<SpaceTypeResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let space_type_dto = state.location_service.create_space_type(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(SpaceTypeResponse {
            id: space_type_dto.id,
            name: space_type_dto.name,
            description: space_type_dto.description,
            created_at: space_type_dto.created_at,
            updated_at: space_type_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/space-types/:id
pub async fn update_space_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSpaceTypePayload>,
) -> Result<Json<SpaceTypeResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let space_type_dto = state.location_service.update_space_type(id, payload).await?;

    Ok(Json(SpaceTypeResponse {
        id: space_type_dto.id,
        name: space_type_dto.name,
        description: space_type_dto.description,
        created_at: space_type_dto.created_at,
        updated_at: space_type_dto.updated_at,
    }))
}

/// DELETE /admin/locations/space-types/:id
pub async fn delete_space_type(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_space_type(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

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
