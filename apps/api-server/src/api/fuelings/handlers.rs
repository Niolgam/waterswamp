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
pub struct FuelingListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub vehicle_id: Option<Uuid>,
    pub driver_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
}

fn default_limit() -> i64 {
    50
}

pub async fn create_fueling(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateFuelingPayload>,
) -> Result<(StatusCode, Json<FuelingWithDetailsDto>), (StatusCode, String)> {
    state
        .fueling_service
        .create_fueling(payload, Some(user.id))
        .await
        .map(|f| (StatusCode::CREATED, Json(f)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_fueling(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FuelingWithDetailsDto>, (StatusCode, String)> {
    state
        .fueling_service
        .get_fueling(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_fueling(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFuelingPayload>,
) -> Result<Json<FuelingWithDetailsDto>, (StatusCode, String)> {
    state
        .fueling_service
        .update_fueling(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_fueling(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .fueling_service
        .delete_fueling(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_fuelings(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<FuelingListQuery>,
) -> Result<Json<FuelingsListResponse>, (StatusCode, String)> {
    state
        .fueling_service
        .list_fuelings(query.limit, query.offset, query.vehicle_id, query.driver_id, query.supplier_id)
        .await
        .map(|(fuelings, total)| {
            Json(FuelingsListResponse {
                fuelings,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
