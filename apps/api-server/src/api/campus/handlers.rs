use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::campus::{CreateCampusDto, UpdateCampusDto};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    CampusListResponse, CampusResponse, CreateCampusRequest, MessageResponse, UpdateCampusRequest,
};

// =============================================================================
// QUERY PARAMS
// =============================================================================

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CityQuery {
    pub city_id: Uuid,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /api/campus
/// Creates a new campus
pub async fn create_campus(
    State(state): State<AppState>,
    Json(payload): Json<CreateCampusRequest>,
) -> Result<(StatusCode, Json<CampusResponse>), AppError> {
    // Validate payload
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    info!(
        name = %payload.name,
        acronym = %payload.acronym,
        "Creating new campus"
    );

    // Convert to domain DTO
    let dto = CreateCampusDto {
        name: payload.name,
        acronym: payload.acronym,
        city_id: payload.city_id,
        coordinates: payload.coordinates,
        address: payload.address,
    };

    // Create campus through service
    let campus = state.campus_service.create_campus(dto).await?;

    info!(campus_id = %campus.id, "Campus created successfully");

    Ok((StatusCode::CREATED, Json(campus.into())))
}

/// GET /api/campus/:id
/// Finds campus by ID
pub async fn get_campus(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CampusResponse>, AppError> {
    let campus = state.campus_service.get_campus(id).await?;
    Ok(Json(campus.into()))
}

/// GET /api/campus/acronym/:acronym
/// Finds campus by acronym
pub async fn get_campus_by_acronym(
    State(state): State<AppState>,
    Path(acronym): Path<String>,
) -> Result<Json<CampusResponse>, AppError> {
    let campus = state.campus_service.get_campus_by_acronym(&acronym).await?;
    Ok(Json(campus.into()))
}

/// GET /api/campus/name/:name
/// Finds campus by name
pub async fn get_campus_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CampusResponse>, AppError> {
    let campus = state.campus_service.get_campus_by_name(&name).await?;
    Ok(Json(campus.into()))
}

/// GET /api/campus
/// Lists all campuses (with optional pagination)
pub async fn list_campuses(
    State(state): State<AppState>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<CampusListResponse>, AppError> {
    let campus_list = if params.limit.is_some() || params.offset.is_some() {
        state
            .campus_service
            .list_campuses_paginated(params.limit, params.offset)
            .await?
    } else {
        state.campus_service.list_all_campuses().await?
    };

    let total = state.campus_service.count_campuses().await?;

    Ok(Json(CampusListResponse {
        campuses: campus_list.into_iter().map(Into::into).collect(),
        total,
    }))
}

/// GET /api/campus/search/city?city_id=uuid
/// Finds campuses by city ID
pub async fn search_by_city(
    State(state): State<AppState>,
    Query(params): Query<CityQuery>,
) -> Result<Json<CampusListResponse>, AppError> {
    let campus_list = state.campus_service.find_by_city(params.city_id).await?;

    Ok(Json(CampusListResponse {
        campuses: campus_list.into_iter().map(Into::into).collect(),
        total: campus_list.len() as i64,
    }))
}

/// PUT /api/campus/:id
/// Updates existing campus
pub async fn update_campus(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCampusRequest>,
) -> Result<Json<CampusResponse>, AppError> {
    // Validate payload
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    info!(campus_id = %id, "Updating campus");

    // Convert to domain DTO
    let dto = UpdateCampusDto {
        name: payload.name,
        acronym: payload.acronym,
        city_id: payload.city_id,
        coordinates: payload.coordinates,
        address: payload.address,
    };

    // Update campus through service
    let campus = state.campus_service.update_campus(id, dto).await?;

    info!(campus_id = %campus.id, "Campus updated successfully");

    Ok(Json(campus.into()))
}

/// DELETE /api/campus/:id
/// Deletes campus
pub async fn delete_campus(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    info!(campus_id = %id, "Deleting campus");

    state.campus_service.delete_campus(id).await?;

    info!(campus_id = %id, "Campus deleted successfully");

    Ok(Json(MessageResponse {
        message: "Campus deleted successfully".to_string(),
    }))
}
