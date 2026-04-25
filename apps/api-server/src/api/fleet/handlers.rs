use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::middleware::idempotency::IdempotencyKey;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use application::errors::ServiceError;
use domain::models::vehicle::VehicleStatus;
use domain::models::odometer::{
    CreateOdometerReadingPayload, ResolveQuarantinePayload, StatusLeitura,
};
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
    let (data, total) = state
        .vehicle_service
        .list_vehicle_categories(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleCategoriesListResponse {
        data,
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
    let (data, total) = state
        .vehicle_service
        .list_vehicle_makes(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleMakesListResponse {
        data,
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
    let (data, total) = state
        .vehicle_service
        .list_vehicle_models(query.limit, query.offset, query.search, query.make_id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(VehicleModelsListResponse {
        data,
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

pub async fn change_operational_status(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChangeOperationalStatusPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.vehicle_service.change_operational_status(id, payload, Some(user.id)).await {
        Ok(vehicle) => (StatusCode::OK, Json(vehicle)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        )
            .into_response(),
        Err(ServiceError::Conflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "vehicle-not-allocatable",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        )
            .into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
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
        return Err((
            StatusCode::BAD_REQUEST,
            "Arquivo excede o tamanho máximo de 50MB".to_string(),
        ));
    }

    if payload.file_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Nome do arquivo é obrigatório".to_string(),
        ));
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

// ============================
// Odometer Handlers (DRS 4.3)
// ============================

#[derive(Debug, Deserialize, IntoParams)]
pub struct OdometerListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<StatusLeitura>,
}

/// `POST /fleet/vehicles/{id}/odometer`
///
/// Registra uma leitura de odômetro. Requer header `Idempotency-Key: <uuid-v4>`.
/// Retries com o mesmo `Idempotency-Key` retornam o resultado original (DRS 4.4).
pub async fn register_odometer_reading(
    user: CurrentUser,
    idempotency: IdempotencyKey,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(mut payload): Json<CreateOdometerReadingPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    payload.veiculo_id = id;
    let reading = state
        .odometer_service
        .register_reading(payload, idempotency.0, Some(user.id))
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(&reading).unwrap())))
}

/// `GET /fleet/vehicles/{id}/odometer`
pub async fn list_odometer_readings(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<OdometerListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (readings, total) = state
        .odometer_service
        .list_readings(id, query.limit, query.offset, query.status)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": readings, "total": total })))
}

/// `GET /fleet/vehicles/{id}/odometer/projection`
pub async fn get_odometer_projection(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let projection = state
        .odometer_service
        .get_projection(id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::to_value(&projection).unwrap()))
}

/// `PUT /fleet/odometer/{reading_id}/resolve`
///
/// Resolve uma leitura em quarentena: valida ou rejeita (RF-INS-03 / RN16).
pub async fn resolve_odometer_quarantine(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(reading_id): Path<Uuid>,
    Json(payload): Json<ResolveQuarantinePayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.odometer_service.resolve_quarantine(reading_id, payload).await {
        Ok(reading) => (StatusCode::OK, Json(reading)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        )
            .into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ============================
// Asset Management Handlers (RF-AST-06/11/12)
// ============================

// Asset management types come via `use super::contracts::*`

// ── RF-AST-06: Transferência Departamental ──

pub async fn register_department_transfer(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateVehicleDepartmentTransferPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.asset_management_service.transfer_department(id, payload, Some(user.id)).await {
        Ok(t) => (StatusCode::CREATED, Json(t)).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

pub async fn list_department_transfers(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let transfers = state
        .asset_management_service
        .list_transfers(id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": transfers })))
}

// ── RF-AST-11: Depreciação ──

pub async fn upsert_depreciation_config(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<UpsertDepreciationConfigPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.asset_management_service.upsert_depreciation_config(payload, Some(user.id)).await {
        Ok(c) => (StatusCode::OK, Json(c)).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

pub async fn list_depreciation_configs(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let configs = state
        .asset_management_service
        .list_depreciation_configs()
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": configs })))
}

pub async fn get_vehicle_depreciation(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let calc = state
        .asset_management_service
        .calculate_depreciation(id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::to_value(&calc).unwrap()))
}

// ── RF-AST-12: Sinistros ──

pub async fn open_incident(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateVehicleIncidentPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.asset_management_service.open_incident(id, payload, Some(user.id)).await {
        Ok(inc) => (StatusCode::CREATED, Json(inc)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        ).into_response(),
        Err(ServiceError::Conflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "vehicle-not-allocatable",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        ).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct IncidentListQuery {
    pub status: Option<VehicleIncidentStatus>,
}

pub async fn list_incidents(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<IncidentListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let incidents = state
        .asset_management_service
        .list_incidents(id, query.status)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": incidents })))
}

pub async fn update_incident(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(incident_id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleIncidentPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.asset_management_service.update_incident(incident_id, payload, Some(user.id)).await {
        Ok(inc) => (StatusCode::OK, Json(inc)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        ).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-AST-09/10: Processo de Baixa ──

#[derive(Debug, serde::Deserialize)]
pub struct DisposalListQuery {
    pub status: Option<DisposalStatus>,
}

pub async fn open_disposal(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateDisposalProcessPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.asset_management_service.open_disposal(id, payload, Some(user.id)).await {
        Ok(d) => (StatusCode::CREATED, Json(d)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        ).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

pub async fn get_disposal_by_vehicle(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let disposal = state
        .asset_management_service
        .get_disposal_by_vehicle(id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": disposal })))
}

pub async fn list_disposals(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<DisposalListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .asset_management_service
        .list_disposals(query.status)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn advance_disposal(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(disposal_id): Path<Uuid>,
    Json(payload): Json<AdvanceDisposalPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.asset_management_service.advance_disposal(disposal_id, payload, Some(user.id)).await {
        Ok(d) => (StatusCode::OK, Json(d)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        ).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

pub async fn add_disposal_step(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(disposal_id): Path<Uuid>,
    Json(payload): Json<CreateDisposalStepPayload>,
) -> Result<(StatusCode, Json<VehicleDisposalStepDto>), (StatusCode, String)> {
    state
        .asset_management_service
        .add_disposal_step(disposal_id, payload, Some(user.id))
        .await
        .map(|s| (StatusCode::CREATED, Json(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_disposal_steps(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(disposal_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let steps = state
        .asset_management_service
        .list_disposal_steps(disposal_id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": steps })))
}

// ── RF-ADM-07: Catálogo de Combustíveis ──

pub async fn list_fuels(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .asset_management_service
        .list_fuels(true)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_fuel(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateFleetFuelCatalogPayload>,
) -> Result<(StatusCode, Json<FleetFuelCatalogDto>), (StatusCode, String)> {
    state
        .asset_management_service
        .create_fuel(payload, Some(user.id))
        .await
        .map(|f| (StatusCode::CREATED, Json(f)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_fuel(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFleetFuelCatalogPayload>,
) -> Result<Json<FleetFuelCatalogDto>, (StatusCode, String)> {
    state
        .asset_management_service
        .update_fuel(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ── RF-ADM-08: Catálogo de Serviços de Manutenção ──

pub async fn list_maintenance_services(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .asset_management_service
        .list_maintenance_services(true)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_maintenance_service(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateFleetMaintenanceServicePayload>,
) -> Result<(StatusCode, Json<FleetMaintenanceServiceDto>), (StatusCode, String)> {
    state
        .asset_management_service
        .create_maintenance_service(payload, Some(user.id))
        .await
        .map(|s| (StatusCode::CREATED, Json(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_maintenance_service(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFleetMaintenanceServicePayload>,
) -> Result<Json<FleetMaintenanceServiceDto>, (StatusCode, String)> {
    state
        .asset_management_service
        .update_maintenance_service(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ── RF-ADM-01: Parâmetros do Sistema ──

pub async fn list_system_params(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .asset_management_service
        .list_system_params()
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn upsert_system_param(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<UpsertFleetSystemParamPayload>,
) -> Result<Json<FleetSystemParamDto>, (StatusCode, String)> {
    state
        .asset_management_service
        .upsert_system_param(payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ── RF-ADM-02: Templates de Checklist ──

pub async fn list_checklist_templates(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .asset_management_service
        .list_checklist_templates(true)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn create_checklist_template(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateFleetChecklistTemplatePayload>,
) -> Result<(StatusCode, Json<FleetChecklistTemplateDto>), (StatusCode, String)> {
    state
        .asset_management_service
        .create_checklist_template(payload, Some(user.id))
        .await
        .map(|t| (StatusCode::CREATED, Json(t)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn add_checklist_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<CreateFleetChecklistItemPayload>,
) -> Result<(StatusCode, Json<FleetChecklistItemDto>), (StatusCode, String)> {
    state
        .asset_management_service
        .add_checklist_item(template_id, payload)
        .await
        .map(|i| (StatusCode::CREATED, Json(i)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_checklist_items(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let items = state
        .asset_management_service
        .list_checklist_items(template_id)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": items })))
}

// ── RF-MNT: Manutenção (atalhos via /fleet/vehicles/{id}/maintenance) ──

use domain::models::maintenance::{
    CreateMaintenanceOrderPayload, MaintenanceCostSummaryDto, MaintenanceOrderStatus,
};

#[derive(Debug, serde::Deserialize)]
pub struct MaintenanceListQuery {
    pub status: Option<MaintenanceOrderStatus>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

pub async fn open_maintenance_order(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Json(payload): Json<CreateMaintenanceOrderPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.maintenance_service.open_order(vehicle_id, payload, Some(user.id)).await {
        Ok(o) => (StatusCode::CREATED, Json(o)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "type": "optimistic-lock-failure",
                "title": "Conflict",
                "status": 409,
                "detail": msg
            })),
        ).into_response(),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

pub async fn list_maintenance_orders(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Query(q): Query<MaintenanceListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (orders, total) = state
        .maintenance_service
        .list_orders(Some(vehicle_id), q.status, q.limit, q.offset)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": orders, "total": total })))
}

pub async fn get_maintenance_cost_summary(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
) -> Result<Json<MaintenanceCostSummaryDto>, (StatusCode, String)> {
    state
        .maintenance_service
        .cost_summary(vehicle_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
