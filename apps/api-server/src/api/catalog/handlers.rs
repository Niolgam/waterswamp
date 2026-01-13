use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use application::services::catalog_service::CatalogService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::catalog::{ItemType};
use serde::Deserialize;
use std::sync::Arc;
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
pub struct GroupListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub parent_id: Option<Uuid>,
    pub item_type: Option<ItemType>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ItemListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub group_id: Option<Uuid>,
    pub is_stockable: Option<bool>,
    pub is_permanent: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ConversionListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub from_unit_id: Option<Uuid>,
    pub to_unit_id: Option<Uuid>,
}

fn default_limit() -> i64 {
    50
}

// Helper to get catalog service
fn get_catalog_service(state: &AppState) -> Arc<CatalogService> {
    state.catalog_service.clone()
}

// ============================
// Unit of Measure Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/catalog/units",
    tag = "Catalog - Units of Measure",
    request_body = CreateUnitOfMeasurePayload,
    responses(
        (status = 201, description = "Unit created successfully", body = UnitOfMeasureDto),
        (status = 409, description = "Symbol already exists"),
    )
)]
pub async fn create_unit_of_measure(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateUnitOfMeasurePayload>,
) -> Result<(StatusCode, Json<UnitOfMeasureDto>), (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .create_unit_of_measure(payload)
        .await
        .map(|unit| (StatusCode::CREATED, Json(unit)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/units/{id}",
    tag = "Catalog - Units of Measure",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 200, description = "Unit found", body = UnitOfMeasureDto),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn get_unit_of_measure(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnitOfMeasureDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .get_unit_of_measure(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/units",
    tag = "Catalog - Units of Measure",
    params(ListQuery),
    responses(
        (status = 200, description = "List of units", body = UnitsOfMeasureListResponse),
    )
)]
pub async fn list_units_of_measure(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<UnitsOfMeasureListResponse>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let (units, total) = service
        .list_units_of_measure(query.limit, query.offset, query.search)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    Ok(Json(UnitsOfMeasureListResponse {
        units,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    put,
    path = "/api/admin/catalog/units/{id}",
    tag = "Catalog - Units of Measure",
    params(("id" = Uuid, Path,)),
    request_body = UpdateUnitOfMeasurePayload,
    responses(
        (status = 200, description = "Unit updated", body = UnitOfMeasureDto),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn update_unit_of_measure(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUnitOfMeasurePayload>,
) -> Result<Json<UnitOfMeasureDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .update_unit_of_measure(id, payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/catalog/units/{id}",
    tag = "Catalog - Units of Measure",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 204, description = "Unit deleted"),
        (status = 404, description = "Unit not found"),
    )
)]
pub async fn delete_unit_of_measure(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .delete_unit_of_measure(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Catalog Group Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/catalog/groups",
    tag = "Catalog - Groups",
    request_body = CreateCatalogGroupPayload,
    responses(
        (status = 201, description = "Group created", body = CatalogGroupWithDetailsDto),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create_catalog_group(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateCatalogGroupPayload>,
) -> Result<(StatusCode, Json<CatalogGroupWithDetailsDto>), (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let group = service
        .create_catalog_group(payload)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    // Fetch with details
    service
        .get_catalog_group(group.id)
        .await
        .map(|g| (StatusCode::CREATED, Json(g)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/groups/{id}",
    tag = "Catalog - Groups",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 200, description = "Group found", body = CatalogGroupWithDetailsDto),
        (status = 404, description = "Group not found"),
    )
)]
pub async fn get_catalog_group(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CatalogGroupWithDetailsDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .get_catalog_group(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/groups",
    tag = "Catalog - Groups",
    params(GroupListQuery),
    responses(
        (status = 200, description = "List of groups", body = CatalogGroupsListResponse),
    )
)]
pub async fn list_catalog_groups(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<GroupListQuery>,
) -> Result<Json<CatalogGroupsListResponse>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let (groups, total) = service
        .list_catalog_groups(
            query.limit,
            query.offset,
            query.search,
            query.parent_id,
            query.item_type,
            query.is_active,
        )
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    Ok(Json(CatalogGroupsListResponse {
        groups,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/groups/tree",
    tag = "Catalog - Groups",
    responses(
        (status = 200, description = "Hierarchical tree", body = Vec<CatalogGroupTreeNode>),
    )
)]
pub async fn get_catalog_group_tree(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<CatalogGroupTreeNode>>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .get_catalog_group_tree()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    put,
    path = "/api/admin/catalog/groups/{id}",
    tag = "Catalog - Groups",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatalogGroupPayload,
    responses(
        (status = 200, description = "Group updated", body = CatalogGroupWithDetailsDto),
        (status = 404, description = "Group not found"),
    )
)]
pub async fn update_catalog_group(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCatalogGroupPayload>,
) -> Result<Json<CatalogGroupWithDetailsDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let _ = service
        .update_catalog_group(id, payload)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    // Fetch with details
    service
        .get_catalog_group(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/catalog/groups/{id}",
    tag = "Catalog - Groups",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 204, description = "Group deleted"),
        (status = 404, description = "Group not found"),
        (status = 409, description = "Group has children or items"),
    )
)]
pub async fn delete_catalog_group(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .delete_catalog_group(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Catalog Item Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/catalog/items",
    tag = "Catalog - Items",
    request_body = CreateCatalogItemPayload,
    responses(
        (status = 201, description = "Item created", body = CatalogItemWithDetailsDto),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create_catalog_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateCatalogItemPayload>,
) -> Result<(StatusCode, Json<CatalogItemWithDetailsDto>), (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let item = service
        .create_catalog_item(payload)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    // Fetch with details
    service
        .get_catalog_item(item.id)
        .await
        .map(|i| (StatusCode::CREATED, Json(i)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/items/{id}",
    tag = "Catalog - Items",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 200, description = "Item found", body = CatalogItemWithDetailsDto),
        (status = 404, description = "Item not found"),
    )
)]
pub async fn get_catalog_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CatalogItemWithDetailsDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .get_catalog_item(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/items",
    tag = "Catalog - Items",
    params(ItemListQuery),
    responses(
        (status = 200, description = "List of items", body = CatalogItemsListResponse),
    )
)]
pub async fn list_catalog_items(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ItemListQuery>,
) -> Result<Json<CatalogItemsListResponse>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let (items, total) = service
        .list_catalog_items(
            query.limit,
            query.offset,
            query.search,
            query.group_id,
            query.is_stockable,
            query.is_permanent,
            query.is_active,
        )
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    Ok(Json(CatalogItemsListResponse {
        items,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    put,
    path = "/api/admin/catalog/items/{id}",
    tag = "Catalog - Items",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatalogItemPayload,
    responses(
        (status = 200, description = "Item updated", body = CatalogItemWithDetailsDto),
        (status = 404, description = "Item not found"),
    )
)]
pub async fn update_catalog_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCatalogItemPayload>,
) -> Result<Json<CatalogItemWithDetailsDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let _ = service
        .update_catalog_item(id, payload)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    // Fetch with details
    service
        .get_catalog_item(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/catalog/items/{id}",
    tag = "Catalog - Items",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 204, description = "Item deleted"),
        (status = 404, description = "Item not found"),
    )
)]
pub async fn delete_catalog_item(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .delete_catalog_item(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Unit Conversion Handlers
// ============================

#[utoipa::path(
    post,
    path = "/api/admin/catalog/conversions",
    tag = "Catalog - Unit Conversions",
    request_body = CreateUnitConversionPayload,
    responses(
        (status = 201, description = "Conversion created", body = UnitConversionWithDetailsDto),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create_unit_conversion(
    _user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateUnitConversionPayload>,
) -> Result<(StatusCode, Json<UnitConversionWithDetailsDto>), (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let conversion = service
        .create_unit_conversion(payload)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    // Fetch with details
    service
        .get_unit_conversion(conversion.id)
        .await
        .map(|c| (StatusCode::CREATED, Json(c)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/conversions/{id}",
    tag = "Catalog - Unit Conversions",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 200, description = "Conversion found", body = UnitConversionWithDetailsDto),
        (status = 404, description = "Conversion not found"),
    )
)]
pub async fn get_unit_conversion(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnitConversionWithDetailsDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .get_unit_conversion(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    get,
    path = "/api/admin/catalog/conversions",
    tag = "Catalog - Unit Conversions",
    params(ConversionListQuery),
    responses(
        (status = 200, description = "List of conversions", body = UnitConversionsListResponse),
    )
)]
pub async fn list_unit_conversions(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<ConversionListQuery>,
) -> Result<Json<UnitConversionsListResponse>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let (conversions, total) = service
        .list_unit_conversions(
            query.limit,
            query.offset,
            query.from_unit_id,
            query.to_unit_id,
        )
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    Ok(Json(UnitConversionsListResponse {
        conversions,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    put,
    path = "/api/admin/catalog/conversions/{id}",
    tag = "Catalog - Unit Conversions",
    params(("id" = Uuid, Path,)),
    request_body = UpdateUnitConversionPayload,
    responses(
        (status = 200, description = "Conversion updated", body = UnitConversionWithDetailsDto),
        (status = 404, description = "Conversion not found"),
    )
)]
pub async fn update_unit_conversion(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUnitConversionPayload>,
) -> Result<Json<UnitConversionWithDetailsDto>, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    let _ = service
        .update_unit_conversion(id, payload)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;

    // Fetch with details
    service
        .get_unit_conversion(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(
    delete,
    path = "/api/admin/catalog/conversions/{id}",
    tag = "Catalog - Unit Conversions",
    params(("id" = Uuid, Path,)),
    responses(
        (status = 204, description = "Conversion deleted"),
        (status = 404, description = "Conversion not found"),
    )
)]
pub async fn delete_unit_conversion(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = get_catalog_service(&state);
    
    service
        .delete_unit_conversion(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
