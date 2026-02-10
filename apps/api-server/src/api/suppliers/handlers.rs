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
pub struct SupplierListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub supplier_type: Option<SupplierType>,
    pub is_active: Option<bool>,
}

fn default_limit() -> i64 {
    50
}

pub async fn create_supplier(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateSupplierPayload>,
) -> Result<(StatusCode, Json<SupplierWithDetailsDto>), (StatusCode, String)> {
    state
        .supplier_service
        .create_supplier(payload, Some(user.id))
        .await
        .map(|s| (StatusCode::CREATED, Json(s)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_supplier(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SupplierWithDetailsDto>, (StatusCode, String)> {
    state
        .supplier_service
        .get_supplier(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_supplier(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSupplierPayload>,
) -> Result<Json<SupplierWithDetailsDto>, (StatusCode, String)> {
    state
        .supplier_service
        .update_supplier(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_supplier(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .supplier_service
        .delete_supplier(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_suppliers(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<SupplierListQuery>,
) -> Result<Json<SuppliersListResponse>, (StatusCode, String)> {
    state
        .supplier_service
        .list_suppliers(query.limit, query.offset, query.search, query.supplier_type, query.is_active)
        .await
        .map(|(suppliers, total)| {
            Json(SuppliersListResponse {
                suppliers,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
