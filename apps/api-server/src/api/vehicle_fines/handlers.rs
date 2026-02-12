use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

fn default_limit() -> i64 {
    50
}

// ============================
// Fine Type handlers
// ============================

#[derive(Debug, Deserialize, IntoParams)]
pub struct FineTypeListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub severity: Option<FineSeverity>,
    pub is_active: Option<bool>,
}

pub async fn create_fine_type(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleFineTypePayload>,
) -> Result<(StatusCode, Json<VehicleFineTypeDto>), (StatusCode, String)> {
    state
        .vehicle_fine_service
        .create_fine_type(payload, Some(user.id))
        .await
        .map(|ft| (StatusCode::CREATED, Json(ft)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_fine_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleFineTypeDto>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .get_fine_type(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_fine_type(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleFineTypePayload>,
) -> Result<Json<VehicleFineTypeDto>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .update_fine_type(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_fine_type(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .delete_fine_type(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_fine_types(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<FineTypeListQuery>,
) -> Result<Json<VehicleFineTypesListResponse>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .list_fine_types(query.limit, query.offset, query.search, query.severity, query.is_active)
        .await
        .map(|(fine_types, total)| {
            Json(VehicleFineTypesListResponse {
                fine_types,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Fine handlers
// ============================

#[derive(Debug, Deserialize, IntoParams)]
pub struct FineListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub vehicle_id: Option<Uuid>,
    pub fine_type_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub status: Option<FineStatus>,
    pub search: Option<String>,
    #[serde(default)]
    pub include_deleted: bool,
}

pub async fn create_fine(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehicleFinePayload>,
) -> Result<(StatusCode, Json<VehicleFineWithDetailsDto>), (StatusCode, String)> {
    state
        .vehicle_fine_service
        .create_fine(payload, Some(user.id))
        .await
        .map(|f| (StatusCode::CREATED, Json(f)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_fine(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleFineWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .get_fine(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_fine(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleFinePayload>,
) -> Result<Json<VehicleFineWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .update_fine(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_fine(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .delete_fine(id, Some(user.id))
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn restore_fine(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleFineWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .restore_fine(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn change_fine_status(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChangeFineStatusPayload>,
) -> Result<Json<VehicleFineWithDetailsDto>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .change_fine_status(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_fine_status_history(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VehicleFineStatusHistoryDto>>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .get_fine_status_history(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_fines(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<FineListQuery>,
) -> Result<Json<VehicleFinesListResponse>, (StatusCode, String)> {
    state
        .vehicle_fine_service
        .list_fines(
            query.limit,
            query.offset,
            query.vehicle_id,
            query.fine_type_id,
            query.supplier_id,
            query.driver_id,
            query.status,
            query.search,
            query.include_deleted,
        )
        .await
        .map(|(fines, total)| {
            Json(VehicleFinesListResponse {
                fines,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
