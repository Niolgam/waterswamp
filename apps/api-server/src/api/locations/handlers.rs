use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use domain::models::{
    CreateBuildingPayload, CreateBuildingTypePayload, CreateCityPayload, CreateCountryPayload,
    CreateDepartmentCategoryPayload, CreateFloorPayload, CreateSitePayload, CreateSiteTypePayload,
    CreateSpacePayload, CreateSpaceTypePayload, CreateStatePayload, ListBuildingsQuery,
    ListBuildingTypesQuery, ListCitiesQuery, ListCountriesQuery, ListDepartmentCategoriesQuery,
    ListFloorsQuery, ListSitesQuery, ListSiteTypesQuery, ListSpacesQuery, ListSpaceTypesQuery,
    ListStatesQuery, PaginatedBuildings, PaginatedBuildingTypes, PaginatedCities,
    PaginatedCountries, PaginatedDepartmentCategories, PaginatedFloors, PaginatedSites,
    PaginatedSiteTypes, PaginatedSpaceTypes, PaginatedStates, UpdateBuildingPayload,
    UpdateBuildingTypePayload, UpdateCityPayload, UpdateCountryPayload,
    UpdateDepartmentCategoryPayload, UpdateFloorPayload, UpdateSitePayload, UpdateSiteTypePayload,
    UpdateSpacePayload, UpdateSpaceTypePayload, UpdateStatePayload,
};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    BuildingResponse, BuildingTypeResponse, CityResponse, CityWithStateResponse, CountryResponse,
    DepartmentCategoryResponse, FloorResponse, SiteResponse, SiteTypeResponse, SpaceResponse,
    SpaceTypeResponse, StateResponse, StateWithCountryResponse,
};

// ============================
// Country Handlers
// ============================

/// GET /admin/locations/countries
pub async fn list_countries(
    State(state): State<AppState>,
    Query(params): Query<ListCountriesQuery>,
) -> Result<Json<PaginatedCountries>, AppError> {
    let result = state
        .location_service
        .list_countries(
            params.limit.unwrap_or(10),
            params.offset.unwrap_or(0),
            params.search,
        )
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/countries/:id
pub async fn get_country(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CountryResponse>, AppError> {
    let country_dto = state.location_service.get_country(id).await?;

    Ok(Json(CountryResponse {
        id: country_dto.id,
        name: country_dto.name,
        code: country_dto.code,
        created_at: country_dto.created_at,
        updated_at: country_dto.updated_at,
    }))
}

/// POST /admin/locations/countries
pub async fn create_country(
    State(state): State<AppState>,
    Json(payload): Json<CreateCountryPayload>,
) -> Result<(StatusCode, Json<CountryResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let country_dto = state.location_service.create_country(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(CountryResponse {
            id: country_dto.id,
            name: country_dto.name,
            code: country_dto.code,
            created_at: country_dto.created_at,
            updated_at: country_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/countries/:id
pub async fn update_country(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCountryPayload>,
) -> Result<Json<CountryResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let country_dto = state.location_service.update_country(id, payload).await?;

    Ok(Json(CountryResponse {
        id: country_dto.id,
        name: country_dto.name,
        code: country_dto.code,
        created_at: country_dto.created_at,
        updated_at: country_dto.updated_at,
    }))
}

/// DELETE /admin/locations/countries/:id
pub async fn delete_country(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_country(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

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
        .list_states(params.limit, params.offset, params.search, params.country_id)
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
        country_id: state_dto.country_id,
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
            country_id: state_dto.country_id,
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
        country_id: state_dto.country_id,
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
        country_id: city_dto.country_id,
        country_name: city_dto.country_name,
        country_code: city_dto.country_code,
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

// ============================
// Site Handlers (Phase 3A)
// ============================

/// GET /admin/locations/sites
pub async fn list_sites(
    State(state): State<AppState>,
    Query(params): Query<ListSitesQuery>,
) -> Result<Json<PaginatedSites>, AppError> {
    let result = state
        .location_service
        .list_sites(
            params.limit,
            params.offset,
            params.search,
            params.city_id,
            params.site_type_id,
        )
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/sites/:id
pub async fn get_site(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SiteResponse>, AppError> {
    let site = state.location_service.get_site(id).await?;

    Ok(Json(SiteResponse {
        id: site.id,
        name: site.name,
        city_id: site.city_id,
        city_name: site.city_name,
        state_id: site.state_id,
        state_name: site.state_name,
        state_code: site.state_code,
        site_type_id: site.site_type_id,
        site_type_name: site.site_type_name,
        address: site.address,
        created_at: site.created_at,
        updated_at: site.updated_at,
    }))
}

/// POST /admin/locations/sites
pub async fn create_site(
    State(state): State<AppState>,
    Json(payload): Json<CreateSitePayload>,
) -> Result<(StatusCode, Json<SiteResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let site = state.location_service.create_site(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(SiteResponse {
            id: site.id,
            name: site.name,
            city_id: site.city_id,
            city_name: site.city_name,
            state_id: site.state_id,
            state_name: site.state_name,
            state_code: site.state_code,
            site_type_id: site.site_type_id,
            site_type_name: site.site_type_name,
            address: site.address,
            created_at: site.created_at,
            updated_at: site.updated_at,
        }),
    ))
}

/// PUT /admin/locations/sites/:id
pub async fn update_site(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSitePayload>,
) -> Result<Json<SiteResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let site = state.location_service.update_site(id, payload).await?;

    Ok(Json(SiteResponse {
        id: site.id,
        name: site.name,
        city_id: site.city_id,
        city_name: site.city_name,
        state_id: site.state_id,
        state_name: site.state_name,
        state_code: site.state_code,
        site_type_id: site.site_type_id,
        site_type_name: site.site_type_name,
        address: site.address,
        created_at: site.created_at,
        updated_at: site.updated_at,
    }))
}

/// DELETE /admin/locations/sites/:id
pub async fn delete_site(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_site(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Building Handlers (Phase 3B)
// ============================

/// GET /admin/locations/buildings
pub async fn list_buildings(
    State(state): State<AppState>,
    Query(params): Query<ListBuildingsQuery>,
) -> Result<Json<PaginatedBuildings>, AppError> {
    let result = state
        .location_service
        .list_buildings(
            params.limit,
            params.offset,
            params.search,
            params.site_id,
            params.building_type_id,
        )
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/buildings/:id
pub async fn get_building(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildingResponse>, AppError> {
    let building = state.location_service.get_building(id).await?;

    Ok(Json(BuildingResponse {
        id: building.id,
        name: building.name,
        site_id: building.site_id,
        site_name: building.site_name,
        city_id: building.city_id,
        city_name: building.city_name,
        state_id: building.state_id,
        state_name: building.state_name,
        state_code: building.state_code,
        building_type_id: building.building_type_id,
        building_type_name: building.building_type_name,
        description: building.description,
        created_at: building.created_at,
        updated_at: building.updated_at,
    }))
}

/// POST /admin/locations/buildings
pub async fn create_building(
    State(state): State<AppState>,
    Json(payload): Json<CreateBuildingPayload>,
) -> Result<(StatusCode, Json<BuildingResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let building = state.location_service.create_building(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(BuildingResponse {
            id: building.id,
            name: building.name,
            site_id: building.site_id,
            site_name: building.site_name,
            city_id: building.city_id,
            city_name: building.city_name,
            state_id: building.state_id,
            state_name: building.state_name,
            state_code: building.state_code,
            building_type_id: building.building_type_id,
            building_type_name: building.building_type_name,
            description: building.description,
            created_at: building.created_at,
            updated_at: building.updated_at,
        }),
    ))
}

/// PUT /admin/locations/buildings/:id
pub async fn update_building(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBuildingPayload>,
) -> Result<Json<BuildingResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let building = state.location_service.update_building(id, payload).await?;

    Ok(Json(BuildingResponse {
        id: building.id,
        name: building.name,
        site_id: building.site_id,
        site_name: building.site_name,
        city_id: building.city_id,
        city_name: building.city_name,
        state_id: building.state_id,
        state_name: building.state_name,
        state_code: building.state_code,
        building_type_id: building.building_type_id,
        building_type_name: building.building_type_name,
        description: building.description,
        created_at: building.created_at,
        updated_at: building.updated_at,
    }))
}

/// DELETE /admin/locations/buildings/:id
pub async fn delete_building(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_building(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Floor Handlers (Phase 3C)
// ============================

/// GET /admin/locations/floors
pub async fn list_floors(
    State(state): State<AppState>,
    Query(params): Query<ListFloorsQuery>,
) -> Result<Json<PaginatedFloors>, AppError> {
    let result = state
        .location_service
        .list_floors(params.limit, params.offset, params.search, params.building_id)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/floors/:id
pub async fn get_floor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FloorResponse>, AppError> {
    let floor = state.location_service.get_floor(id).await?;

    Ok(Json(FloorResponse {
        id: floor.id,
        floor_number: floor.floor_number,
        building_id: floor.building_id,
        building_name: floor.building_name,
        site_id: floor.site_id,
        site_name: floor.site_name,
        city_id: floor.city_id,
        city_name: floor.city_name,
        state_id: floor.state_id,
        state_name: floor.state_name,
        state_code: floor.state_code,
        description: floor.description,
        created_at: floor.created_at,
        updated_at: floor.updated_at,
    }))
}

/// POST /admin/locations/floors
pub async fn create_floor(
    State(state): State<AppState>,
    Json(payload): Json<CreateFloorPayload>,
) -> Result<(StatusCode, Json<FloorResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let floor = state.location_service.create_floor(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(FloorResponse {
            id: floor.id,
            floor_number: floor.floor_number,
            building_id: floor.building_id,
            building_name: floor.building_name,
            site_id: floor.site_id,
            site_name: floor.site_name,
            city_id: floor.city_id,
            city_name: floor.city_name,
            state_id: floor.state_id,
            state_name: floor.state_name,
            state_code: floor.state_code,
            description: floor.description,
            created_at: floor.created_at,
            updated_at: floor.updated_at,
        }),
    ))
}

/// PUT /admin/locations/floors/:id
pub async fn update_floor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFloorPayload>,
) -> Result<Json<FloorResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let floor = state.location_service.update_floor(id, payload).await?;

    Ok(Json(FloorResponse {
        id: floor.id,
        floor_number: floor.floor_number,
        building_id: floor.building_id,
        building_name: floor.building_name,
        site_id: floor.site_id,
        site_name: floor.site_name,
        city_id: floor.city_id,
        city_name: floor.city_name,
        state_id: floor.state_id,
        state_name: floor.state_name,
        state_code: floor.state_code,
        description: floor.description,
        created_at: floor.created_at,
        updated_at: floor.updated_at,
    }))
}

/// DELETE /admin/locations/floors/:id
pub async fn delete_floor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_floor(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// SPACE HANDLERS (Phase 3D)
// =============================================================================

/// GET /admin/locations/spaces
pub async fn list_spaces(
    State(state): State<AppState>,
    Query(query): Query<ListSpacesQuery>,
) -> Result<Json<Value>, AppError> {
    let result = state
        .location_service
        .list_spaces(
            query.limit,
            query.offset,
            query.search,
            query.floor_id,
            query.space_type_id,
        )
        .await?;

    let spaces: Vec<SpaceResponse> = result
        .spaces
        .into_iter()
        .map(|space| SpaceResponse {
            id: space.id,
            name: space.name,
            floor_id: space.floor_id,
            floor_number: space.floor_number,
            building_id: space.building_id,
            building_name: space.building_name,
            site_id: space.site_id,
            site_name: space.site_name,
            city_id: space.city_id,
            city_name: space.city_name,
            state_id: space.state_id,
            state_name: space.state_name,
            state_code: space.state_code,
            space_type_id: space.space_type_id,
            space_type_name: space.space_type_name,
            description: space.description,
            created_at: space.created_at,
            updated_at: space.updated_at,
        })
        .collect();

    Ok(Json(json!({
        "spaces": spaces,
        "total": result.total,
        "limit": result.limit,
        "offset": result.offset
    })))
}

/// POST /admin/locations/spaces
pub async fn create_space(
    State(state): State<AppState>,
    Json(payload): Json<CreateSpacePayload>,
) -> Result<(StatusCode, Json<SpaceResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let space = state.location_service.create_space(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(SpaceResponse {
            id: space.id,
            name: space.name,
            floor_id: space.floor_id,
            floor_number: space.floor_number,
            building_id: space.building_id,
            building_name: space.building_name,
            site_id: space.site_id,
            site_name: space.site_name,
            city_id: space.city_id,
            city_name: space.city_name,
            state_id: space.state_id,
            state_name: space.state_name,
            state_code: space.state_code,
            space_type_id: space.space_type_id,
            space_type_name: space.space_type_name,
            description: space.description,
            created_at: space.created_at,
            updated_at: space.updated_at,
        }),
    ))
}

/// GET /admin/locations/spaces/:id
pub async fn get_space(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SpaceResponse>, AppError> {
    let space = state.location_service.get_space(id).await?;

    Ok(Json(SpaceResponse {
        id: space.id,
        name: space.name,
        floor_id: space.floor_id,
        floor_number: space.floor_number,
        building_id: space.building_id,
        building_name: space.building_name,
        site_id: space.site_id,
        site_name: space.site_name,
        city_id: space.city_id,
        city_name: space.city_name,
        state_id: space.state_id,
        state_name: space.state_name,
        state_code: space.state_code,
        space_type_id: space.space_type_id,
        space_type_name: space.space_type_name,
        description: space.description,
        created_at: space.created_at,
        updated_at: space.updated_at,
    }))
}

/// PUT /admin/locations/spaces/:id
pub async fn update_space(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSpacePayload>,
) -> Result<Json<SpaceResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let space = state.location_service.update_space(id, payload).await?;

    Ok(Json(SpaceResponse {
        id: space.id,
        name: space.name,
        floor_id: space.floor_id,
        floor_number: space.floor_number,
        building_id: space.building_id,
        building_name: space.building_name,
        site_id: space.site_id,
        site_name: space.site_name,
        city_id: space.city_id,
        city_name: space.city_name,
        state_id: space.state_id,
        state_name: space.state_name,
        state_code: space.state_code,
        space_type_id: space.space_type_id,
        space_type_name: space.space_type_name,
        description: space.description,
        created_at: space.created_at,
        updated_at: space.updated_at,
    }))
}

/// DELETE /admin/locations/spaces/:id
pub async fn delete_space(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_space(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
