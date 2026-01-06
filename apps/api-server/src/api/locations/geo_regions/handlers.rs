use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    CreateCityPayload, CreateCountryPayload, CreateStatePayload, ListCitiesQuery,
    ListCountriesQuery, ListStatesQuery, PaginatedCities, PaginatedCountries, PaginatedStates,
    UpdateCityPayload, UpdateCountryPayload, UpdateStatePayload,
};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{CityResponse, CityWithStateResponse, CountryResponse, StateResponse};

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

