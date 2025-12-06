use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{CreateOrganizationalUnitDto, UpdateOrganizationalUnitDto};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    CreateOrganizationalUnitRequest, MessageResponse, OrganizationalUnitListResponse,
    OrganizationalUnitResponse, UpdateOrganizationalUnitRequest,
};

// =============================================================================
// QUERY PARAMS
// =============================================================================

#[derive(Deserialize)]
pub struct ParentQuery {
    pub parent_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CategoryQuery {
    pub category_id: Uuid,
}

#[derive(Deserialize)]
pub struct CampusQuery {
    pub campus_id: Uuid,
}

#[derive(Deserialize)]
pub struct AcronymQuery {
    pub acronym: String,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /api/organizational-units
/// Creates a new organizational unit
pub async fn create_unit(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationalUnitRequest>,
) -> Result<(StatusCode, Json<OrganizationalUnitResponse>), AppError> {
    // Validate payload
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    info!(name = %payload.name, "Creating new organizational unit");

    // Convert to domain DTO
    let dto = CreateOrganizationalUnitDto {
        name: payload.name,
        acronym: payload.acronym,
        category_id: payload.category_id,
        parent_id: payload.parent_id,
        description: payload.description,
        is_uorg: payload.is_uorg,
        campus_id: payload.campus_id,
    };

    // Create unit through service
    let unit = state.organizational_unit_service.create_unit(dto).await?;

    info!(unit_id = %unit.id, "Organizational unit created successfully");

    Ok((StatusCode::CREATED, Json(unit.into())))
}

/// GET /api/organizational-units/:id
/// Finds unit by ID
pub async fn get_unit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationalUnitResponse>, AppError> {
    let unit = state.organizational_unit_service.get_unit(id).await?;
    Ok(Json(unit.into()))
}

/// GET /api/organizational-units/acronym?acronym=XXX
/// Finds unit by acronym
pub async fn get_unit_by_acronym(
    State(state): State<AppState>,
    Query(params): Query<AcronymQuery>,
) -> Result<Json<OrganizationalUnitResponse>, AppError> {
    let unit = state
        .organizational_unit_service
        .get_unit_by_acronym(&params.acronym)
        .await?;
    Ok(Json(unit.into()))
}

/// GET /api/organizational-units
/// Lists all units
pub async fn list_units(
    State(state): State<AppState>,
) -> Result<Json<OrganizationalUnitListResponse>, AppError> {
    let units = state.organizational_unit_service.list_all_units().await?;
    let total = state.organizational_unit_service.count_units().await?;

    Ok(Json(OrganizationalUnitListResponse {
        units: units.into_iter().map(Into::into).collect(),
        total,
    }))
}

/// GET /api/organizational-units/root
/// Lists root units (units without parent)
pub async fn list_root_units(
    State(state): State<AppState>,
) -> Result<Json<OrganizationalUnitListResponse>, AppError> {
    let units = state.organizational_unit_service.list_root_units().await?;

    Ok(Json(OrganizationalUnitListResponse {
        units: units.into_iter().map(Into::into).collect(),
        total: units.len() as i64,
    }))
}

/// GET /api/organizational-units/by-parent?parent_id=uuid
/// Lists units by parent
pub async fn list_by_parent(
    State(state): State<AppState>,
    Query(params): Query<ParentQuery>,
) -> Result<Json<OrganizationalUnitListResponse>, AppError> {
    let units = state
        .organizational_unit_service
        .list_by_parent(params.parent_id)
        .await?;

    Ok(Json(OrganizationalUnitListResponse {
        units: units.into_iter().map(Into::into).collect(),
        total: units.len() as i64,
    }))
}

/// GET /api/organizational-units/by-category?category_id=uuid
/// Lists units by category
pub async fn list_by_category(
    State(state): State<AppState>,
    Query(params): Query<CategoryQuery>,
) -> Result<Json<OrganizationalUnitListResponse>, AppError> {
    let units = state
        .organizational_unit_service
        .list_by_category(params.category_id)
        .await?;

    Ok(Json(OrganizationalUnitListResponse {
        units: units.into_iter().map(Into::into).collect(),
        total: units.len() as i64,
    }))
}

/// GET /api/organizational-units/by-campus?campus_id=uuid
/// Lists units by campus
pub async fn list_by_campus(
    State(state): State<AppState>,
    Query(params): Query<CampusQuery>,
) -> Result<Json<OrganizationalUnitListResponse>, AppError> {
    let units = state
        .organizational_unit_service
        .list_by_campus(params.campus_id)
        .await?;

    Ok(Json(OrganizationalUnitListResponse {
        units: units.into_iter().map(Into::into).collect(),
        total: units.len() as i64,
    }))
}

/// PUT /api/organizational-units/:id
/// Updates existing unit
pub async fn update_unit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationalUnitRequest>,
) -> Result<Json<OrganizationalUnitResponse>, AppError> {
    // Validate payload
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    info!(unit_id = %id, "Updating organizational unit");

    // Convert to domain DTO
    let dto = UpdateOrganizationalUnitDto {
        name: payload.name,
        acronym: payload.acronym,
        category_id: payload.category_id,
        parent_id: Some(payload.parent_id),
        description: payload.description,
        is_uorg: payload.is_uorg,
        campus_id: Some(payload.campus_id),
    };

    // Update unit through service
    let unit = state.organizational_unit_service.update_unit(id, dto).await?;

    info!(unit_id = %unit.id, "Organizational unit updated successfully");

    Ok(Json(unit.into()))
}

/// DELETE /api/organizational-units/:id
/// Deletes unit
pub async fn delete_unit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    info!(unit_id = %id, "Deleting organizational unit");

    state.organizational_unit_service.delete_unit(id).await?;

    info!(unit_id = %id, "Organizational unit deleted successfully");

    Ok(Json(MessageResponse {
        message: "Organizational unit deleted successfully".to_string(),
    }))
}
