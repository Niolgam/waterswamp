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

#[derive(Debug, Deserialize, IntoParams)]
pub struct DriverListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub driver_type: Option<DriverType>,
    pub is_active: Option<bool>,
}

fn default_limit() -> i64 {
    50
}

pub async fn create_driver(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateDriverPayload>,
) -> Result<(StatusCode, Json<DriverDto>), (StatusCode, String)> {
    state
        .driver_service
        .create_driver(payload, Some(user.id))
        .await
        .map(|d| (StatusCode::CREATED, Json(d)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_driver(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DriverDto>, (StatusCode, String)> {
    state
        .driver_service
        .get_driver(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_driver(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDriverPayload>,
) -> Result<Json<DriverDto>, (StatusCode, String)> {
    state
        .driver_service
        .update_driver(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_driver(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .driver_service
        .delete_driver(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_drivers(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<DriverListQuery>,
) -> Result<Json<DriversListResponse>, (StatusCode, String)> {
    state
        .driver_service
        .list_drivers(query.limit, query.offset, query.search, query.driver_type, query.is_active)
        .await
        .map(|(drivers, total)| {
            Json(DriversListResponse {
                drivers,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
