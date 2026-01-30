use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    BuildingTypeDto, BuildingWithRelationsDto, CreateBuildingPayload, CreateBuildingTypePayload,
    CreateFloorPayload, CreateSitePayload, CreateSiteTypePayload, CreateSpacePayload,
    CreateSpaceTypePayload, FloorWithRelationsDto, ListBuildingsQuery, ListBuildingTypesQuery,
    ListFloorsQuery, ListSitesQuery, ListSiteTypesQuery, ListSpacesQuery, ListSpaceTypesQuery,
    SiteTypeDto, SiteWithRelationsDto, SpaceTypeDto, SpaceWithRelationsDto, UpdateBuildingPayload,
    UpdateBuildingTypePayload, UpdateFloorPayload, UpdateSitePayload, UpdateSiteTypePayload,
    UpdateSpacePayload, UpdateSpaceTypePayload,
};
use domain::pagination::Paginated;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    BuildingResponse, BuildingTypeResponse, FloorResponse, SiteResponse, SiteTypeResponse,
    SpaceResponse, SpaceTypeResponse,
};

// ============================
// Site Type Handlers
// ============================

/// GET /admin/locations/site-types
pub async fn list_site_types(
    State(state): State<AppState>,
    Query(params): Query<ListSiteTypesQuery>,
) -> Result<Json<Paginated<SiteTypeDto>>, AppError> {
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
) -> Result<Json<Paginated<BuildingTypeDto>>, AppError> {
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
) -> Result<Json<Paginated<SpaceTypeDto>>, AppError> {
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
// Site Handlers (Phase 3A)
// ============================

/// GET /admin/locations/sites
pub async fn list_sites(
    State(state): State<AppState>,
    Query(params): Query<ListSitesQuery>,
) -> Result<Json<Paginated<SiteWithRelationsDto>>, AppError> {
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
) -> Result<Json<Paginated<BuildingWithRelationsDto>>, AppError> {
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
) -> Result<Json<Paginated<FloorWithRelationsDto>>, AppError> {
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
