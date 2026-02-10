use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::vehicle::VehicleStatus;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

// ============================
// Query Parameters
// ============================

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ModelListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub make_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct VehicleListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub status: Option<VehicleStatus>,
    pub model_id: Option<Uuid>,
    pub fuel_type_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_search_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

fn default_search_limit() -> i64 {
    10
}

// ============================
// Vehicle Category Handlers
// ============================

pub async fn create_vehicle_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleCategoryPayload>,
) -> Result<(StatusCode, Json<VehicleCategoryDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle_category(payload)
        .await
        .map(|c| (StatusCode::CREATED, Json(c)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleCategoryDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_category(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_categories(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<VehicleCategoriesListResponse>, (StatusCode, String)> {
    let (categories, total) = state
        .vehicle_service
        .list_vehicle_categories(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleCategoriesListResponse {
        categories,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleCategoryPayload>,
) -> Result<Json<VehicleCategoryDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle_category(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_category(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_category(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Make Handlers
// ============================

pub async fn create_vehicle_make(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleMakePayload>,
) -> Result<(StatusCode, Json<VehicleMakeDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle_make(payload)
        .await
        .map(|m| (StatusCode::CREATED, Json(m)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle_make(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleMakeDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_make(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_makes(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<VehicleMakesListResponse>, (StatusCode, String)> {
    let (makes, total) = state
        .vehicle_service
        .list_vehicle_makes(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleMakesListResponse {
        makes,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle_make(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleMakePayload>,
) -> Result<Json<VehicleMakeDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle_make(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_make(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_make(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Model Handlers
// ============================

pub async fn create_vehicle_model(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleModelPayload>,
) -> Result<(StatusCode, Json<VehicleModelDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle_model(payload)
        .await
        .map(|m| (StatusCode::CREATED, Json(m)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle_model(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleModelWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_model(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_models(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ModelListQuery>,
) -> Result<Json<VehicleModelsListResponse>, (StatusCode, String)> {
    let (models, total) = state
        .vehicle_service
        .list_vehicle_models(query.limit, query.offset, query.search, query.make_id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleModelsListResponse {
        models,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle_model(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleModelPayload>,
) -> Result<Json<VehicleModelDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle_model(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_model(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_model(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Color Handlers
// ============================

pub async fn create_vehicle_color(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleColorPayload>,
) -> Result<(StatusCode, Json<VehicleColorDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle_color(payload)
        .await
        .map(|c| (StatusCode::CREATED, Json(c)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle_color(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleColorDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_color(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_colors(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<VehicleColorsListResponse>, (StatusCode, String)> {
    let (colors, total) = state
        .vehicle_service
        .list_vehicle_colors(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleColorsListResponse {
        colors,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle_color(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleColorPayload>,
) -> Result<Json<VehicleColorDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle_color(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_color(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_color(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Fuel Type Handlers
// ============================

pub async fn create_vehicle_fuel_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleFuelTypePayload>,
) -> Result<(StatusCode, Json<VehicleFuelTypeDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle_fuel_type(payload)
        .await
        .map(|f| (StatusCode::CREATED, Json(f)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle_fuel_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleFuelTypeDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_fuel_type(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_fuel_types(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<VehicleFuelTypesListResponse>, (StatusCode, String)> {
    let (fuel_types, total) = state
        .vehicle_service
        .list_vehicle_fuel_types(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleFuelTypesListResponse {
        fuel_types,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle_fuel_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleFuelTypePayload>,
) -> Result<Json<VehicleFuelTypeDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle_fuel_type(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_fuel_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_fuel_type(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Transmission Type Handlers
// ============================

pub async fn create_vehicle_transmission_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleTransmissionTypePayload>,
) -> Result<(StatusCode, Json<VehicleTransmissionTypeDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle_transmission_type(payload)
        .await
        .map(|t| (StatusCode::CREATED, Json(t)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle_transmission_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleTransmissionTypeDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_transmission_type(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_transmission_types(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<VehicleTransmissionTypesListResponse>, (StatusCode, String)> {
    let (transmission_types, total) = state
        .vehicle_service
        .list_vehicle_transmission_types(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleTransmissionTypesListResponse {
        transmission_types,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle_transmission_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleTransmissionTypePayload>,
) -> Result<Json<VehicleTransmissionTypeDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle_transmission_type(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_transmission_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_transmission_type(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Handlers
// ============================

pub async fn create_vehicle(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehiclePayload>,
) -> Result<(StatusCode, Json<VehicleWithDetailsDto>), (StatusCode, String)> {
    state
        .vehicle_service
        .create_vehicle(payload, Some(user.id))
        .await
        .map(|v| (StatusCode::CREATED, Json(v)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_vehicle(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicles(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<VehicleListQuery>,
) -> Result<Json<VehiclesListResponse>, (StatusCode, String)> {
    let (vehicles, total) = state
        .vehicle_service
        .list_vehicles(
            query.limit,
            query.offset,
            query.search,
            query.status,
            query.model_id,
            query.fuel_type_id,
            query.department_id,
            query.include_deleted,
        )
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehiclesListResponse {
        vehicles,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

pub async fn update_vehicle(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehiclePayload>,
) -> Result<Json<VehicleWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .update_vehicle(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle(id, Some(user.id))
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn change_vehicle_status(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChangeVehicleStatusPayload>,
) -> Result<Json<VehicleWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_service
        .change_vehicle_status(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn search_vehicles(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<VehicleSearchResponse>, (StatusCode, String)> {
    let vehicles = state
        .vehicle_service
        .search_vehicles(&query.q, query.limit)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleSearchResponse { vehicles }))
}

pub async fn get_vehicle_status_history(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VehicleStatusHistoryDto>>, (StatusCode, String)> {
    state
        .vehicle_service
        .get_vehicle_status_history(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Vehicle Document Handlers
// ============================

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub document_type: DocumentType,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub description: Option<String>,
}

/// Register document metadata. The file itself should be uploaded
/// separately to local storage, and the file_path is generated here.
pub async fn upload_vehicle_document(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<VehicleDocumentDto>), (StatusCode, String)> {
    // Max 50MB check
    if payload.file_size > 50 * 1024 * 1024 {
        return Err((StatusCode::BAD_REQUEST, "Arquivo excede o tamanho máximo de 50MB".to_string()));
    }

    if payload.file_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Nome do arquivo é obrigatório".to_string()));
    }

    // Generate the storage path
    let upload_dir = format!("uploads/vehicles/{}", vehicle_id);
    let stored_name = format!("{}_{}", Uuid::new_v4(), payload.file_name);
    let file_path = format!("{}/{}", upload_dir, stored_name);

    // Create directory (best-effort, actual file write would happen via a separate binary upload)
    let _ = tokio::fs::create_dir_all(&upload_dir).await;

    state
        .vehicle_service
        .create_vehicle_document(
            vehicle_id,
            payload.document_type,
            &payload.file_name,
            &file_path,
            payload.file_size,
            &payload.mime_type,
            payload.description.as_deref(),
            Some(user.id),
        )
        .await
        .map(|d| (StatusCode::CREATED, Json(d)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_vehicle_documents(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<Json<Vec<VehicleDocumentDto>>, (StatusCode, String)> {
    state
        .vehicle_service
        .list_vehicle_documents(vehicle_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_vehicle_document(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path((_vehicle_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_service
        .delete_vehicle_document(doc_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
