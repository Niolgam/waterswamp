use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use domain::models::{CreateUnitCategoryDto, UpdateUnitCategoryDto};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

use super::contracts::{
    CreateUnitCategoryRequest, MessageResponse, UnitCategoryListResponse, UnitCategoryResponse,
    UpdateUnitCategoryRequest,
};

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /api/unit-categories
/// Creates a new unit category
pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateUnitCategoryRequest>,
) -> Result<(StatusCode, Json<UnitCategoryResponse>), AppError> {
    // Validate payload
    payload.validate()?;

    info!(name = %payload.name, "Creating new unit category");

    // Convert to domain DTO
    let dto = CreateUnitCategoryDto {
        name: payload.name,
        color_hex: payload.color_hex,
    };

    // Create category through service
    let category = state.unit_category_service.create_category(dto).await?;

    info!(category_id = %category.id, "Unit category created successfully");

    Ok((StatusCode::CREATED, Json(category.into())))
}

/// GET /api/unit-categories/:id
/// Finds category by ID
pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnitCategoryResponse>, AppError> {
    let category = state.unit_category_service.get_category(id).await?;
    Ok(Json(category.into()))
}

/// GET /api/unit-categories
/// Lists all categories
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<UnitCategoryListResponse>, AppError> {
    let categories = state.unit_category_service.list_all_categories().await?;
    let total = categories.len();

    Ok(Json(UnitCategoryListResponse {
        categories: categories.into_iter().map(Into::into).collect(),
        total,
    }))
}

/// PUT /api/unit-categories/:id
/// Updates existing category
pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUnitCategoryRequest>,
) -> Result<Json<UnitCategoryResponse>, AppError> {
    // Validate payload
    payload.validate()?;

    info!(category_id = %id, "Updating unit category");

    // Convert to domain DTO
    let dto = UpdateUnitCategoryDto {
        name: payload.name,
        color_hex: payload.color_hex,
    };

    // Update category through service
    let category = state.unit_category_service.update_category(id, dto).await?;

    info!(category_id = %category.id, "Unit category updated successfully");

    Ok(Json(category.into()))
}

/// DELETE /api/unit-categories/:id
/// Deletes category
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    info!(category_id = %id, "Deleting unit category");

    state.unit_category_service.delete_category(id).await?;

    info!(category_id = %id, "Unit category deleted successfully");

    Ok(Json(MessageResponse {
        message: "Unit category deleted successfully".to_string(),
    }))
}
