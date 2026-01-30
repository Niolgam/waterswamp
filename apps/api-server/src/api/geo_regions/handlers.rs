use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    CityWithStateDto, CountryDto, CreateCityPayload, CreateCountryPayload, CreateStatePayload,
    ListCitiesQuery, ListCountriesQuery, ListStatesQuery, StateWithCountryDto, UpdateCityPayload,
    UpdateCountryPayload, UpdateStatePayload,
};
use domain::pagination::Paginated;
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{CityResponse, CityWithStateResponse, CountryResponse, StateResponse};

// ============================
// Country Handlers
// ============================

/// GET /admin/locations/countries
#[utoipa::path(
    get,
    path = "/api/v1/admin/locations/countries",
    tag = "Admin",
    params(
        ("limit" = Option<i64>, Query, description = "Limite de resultados por página"),
        ("offset" = Option<i64>, Query, description = "Offset para paginação"),
        ("search" = Option<String>, Query, description = "Termo de busca")
    ),
    responses(
        (status = 200, description = "Lista de países", body = Paginated<CountryDto>),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_countries(
    State(state): State<AppState>,
    Query(params): Query<ListCountriesQuery>,
) -> Result<Json<Paginated<CountryDto>>, AppError> {
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
#[utoipa::path(
    get,
    path = "/api/v1/admin/locations/countries/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do país")
    ),
    responses(
        (status = 200, description = "País encontrado", body = CountryResponse),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "País não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_country(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CountryResponse>, AppError> {
    let country_dto = state.location_service.get_country(id).await?;

    Ok(Json(CountryResponse {
        id: country_dto.id,
        name: country_dto.name,
        iso2: country_dto.iso2,
        bacen_code: country_dto.bacen_code,
        created_at: country_dto.created_at,
        updated_at: country_dto.updated_at,
    }))
}

/// POST /admin/locations/countries
#[utoipa::path(
    post,
    path = "/api/v1/admin/locations/countries",
    tag = "Admin",
    request_body = CreateCountryPayload,
    responses(
        (status = 201, description = "País criado com sucesso", body = CountryResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
            iso2: country_dto.iso2,
            bacen_code: country_dto.bacen_code,
            created_at: country_dto.created_at,
            updated_at: country_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/countries/:id
#[utoipa::path(
    put,
    path = "/api/v1/admin/locations/countries/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do país")
    ),
    request_body = UpdateCountryPayload,
    responses(
        (status = 200, description = "País atualizado com sucesso", body = CountryResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "País não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
        iso2: country_dto.iso2,
        bacen_code: country_dto.bacen_code,
        created_at: country_dto.created_at,
        updated_at: country_dto.updated_at,
    }))
}

/// DELETE /admin/locations/countries/:id
#[utoipa::path(
    delete,
    path = "/api/v1/admin/locations/countries/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do país")
    ),
    responses(
        (status = 204, description = "País deletado com sucesso"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "País não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
#[utoipa::path(
    get,
    path = "/api/v1/admin/locations/states",
    tag = "Admin",
    params(
        ("limit" = Option<i64>, Query, description = "Limite de resultados por página"),
        ("offset" = Option<i64>, Query, description = "Offset para paginação"),
        ("search" = Option<String>, Query, description = "Termo de busca"),
        ("country_id" = Option<Uuid>, Query, description = "Filtrar por ID do país")
    ),
    responses(
        (status = 200, description = "Lista de estados", body = Paginated<StateWithCountryDto>),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_states(
    State(state): State<AppState>,
    Query(params): Query<ListStatesQuery>,
) -> Result<Json<Paginated<StateWithCountryDto>>, AppError> {
    let result = state
        .location_service
        .list_states(params.limit, params.offset, params.search, params.country_id)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/states/:id
#[utoipa::path(
    get,
    path = "/api/v1/admin/locations/states/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do estado")
    ),
    responses(
        (status = 200, description = "Estado encontrado", body = StateResponse),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Estado não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_state(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<StateResponse>, AppError> {
    let state_dto = state.location_service.get_state(id).await?;

    Ok(Json(StateResponse {
        id: state_dto.id,
        name: state_dto.name,
        abbreviation: state_dto.abbreviation,
        ibge_code: state_dto.ibge_code,
        country_id: state_dto.country_id,
        created_at: state_dto.created_at,
        updated_at: state_dto.updated_at,
    }))
}

/// POST /admin/locations/states
#[utoipa::path(
    post,
    path = "/api/v1/admin/locations/states",
    tag = "Admin",
    request_body = CreateStatePayload,
    responses(
        (status = 201, description = "Estado criado com sucesso", body = StateResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
            abbreviation: state_dto.abbreviation,
            ibge_code: state_dto.ibge_code,
            country_id: state_dto.country_id,
            created_at: state_dto.created_at,
            updated_at: state_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/states/:id
#[utoipa::path(
    put,
    path = "/api/v1/admin/locations/states/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do estado")
    ),
    request_body = UpdateStatePayload,
    responses(
        (status = 200, description = "Estado atualizado com sucesso", body = StateResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Estado não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
        abbreviation: state_dto.abbreviation,
        ibge_code: state_dto.ibge_code,
        country_id: state_dto.country_id,
        created_at: state_dto.created_at,
        updated_at: state_dto.updated_at,
    }))
}

/// DELETE /admin/locations/states/:id
#[utoipa::path(
    delete,
    path = "/api/v1/admin/locations/states/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do estado")
    ),
    responses(
        (status = 204, description = "Estado deletado com sucesso"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Estado não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
#[utoipa::path(
    get,
    path = "/api/v1/admin/locations/cities",
    tag = "Admin",
    params(
        ("limit" = Option<i64>, Query, description = "Limite de resultados por página"),
        ("offset" = Option<i64>, Query, description = "Offset para paginação"),
        ("search" = Option<String>, Query, description = "Termo de busca"),
        ("state_id" = Option<Uuid>, Query, description = "Filtrar por ID do estado")
    ),
    responses(
        (status = 200, description = "Lista de cidades", body = Paginated<CityWithStateDto>),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_cities(
    State(state): State<AppState>,
    Query(params): Query<ListCitiesQuery>,
) -> Result<Json<Paginated<CityWithStateDto>>, AppError> {
    let result = state
        .location_service
        .list_cities(params.limit, params.offset, params.search, params.state_id)
        .await?;

    Ok(Json(result))
}

/// GET /admin/locations/cities/:id
#[utoipa::path(
    get,
    path = "/api/v1/admin/locations/cities/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID da cidade")
    ),
    responses(
        (status = 200, description = "Cidade encontrada", body = CityWithStateResponse),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Cidade não encontrada")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_city(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CityWithStateResponse>, AppError> {
    let city_dto = state.location_service.get_city(id).await?;

    Ok(Json(CityWithStateResponse {
        id: city_dto.id,
        name: city_dto.name,
        ibge_code: city_dto.ibge_code,
        state_id: city_dto.state_id,
        state_name: city_dto.state_name,
        state_abbreviation: city_dto.state_abbreviation,
        state_ibge_code: city_dto.state_ibge_code,
        country_id: city_dto.country_id,
        country_name: city_dto.country_name,
        country_iso2: city_dto.country_iso2,
        country_bacen_code: city_dto.country_bacen_code,
        created_at: city_dto.created_at,
        updated_at: city_dto.updated_at,
    }))
}

/// POST /admin/locations/cities
#[utoipa::path(
    post,
    path = "/api/v1/admin/locations/cities",
    tag = "Admin",
    request_body = CreateCityPayload,
    responses(
        (status = 201, description = "Cidade criada com sucesso", body = CityResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
            ibge_code: city_dto.ibge_code,
            state_id: city_dto.state_id,
            created_at: city_dto.created_at,
            updated_at: city_dto.updated_at,
        }),
    ))
}

/// PUT /admin/locations/cities/:id
#[utoipa::path(
    put,
    path = "/api/v1/admin/locations/cities/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID da cidade")
    ),
    request_body = UpdateCityPayload,
    responses(
        (status = 200, description = "Cidade atualizada com sucesso", body = CityResponse),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Cidade não encontrada")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
        ibge_code: city_dto.ibge_code,
        state_id: city_dto.state_id,
        created_at: city_dto.created_at,
        updated_at: city_dto.updated_at,
    }))
}

/// DELETE /admin/locations/cities/:id
#[utoipa::path(
    delete,
    path = "/api/v1/admin/locations/cities/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID da cidade")
    ),
    responses(
        (status = 204, description = "Cidade deletada com sucesso"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Cidade não encontrada")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_city(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.location_service.delete_city(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

