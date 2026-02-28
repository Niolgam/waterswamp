use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use application::services::catalog_service::CatalogService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;
use uuid::Uuid;

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
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ClassListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub group_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct CatmatItemListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub class_id: Option<Uuid>,
    pub is_sustainable: Option<bool>,
    pub is_permanent: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct CatserItemListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub class_id: Option<Uuid>,
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

fn default_limit() -> i64 { 50 }
fn svc(state: &AppState) -> Arc<CatalogService> { state.catalog_service.clone() }

// ============================
// Unit of Measure Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/units-of-measure", tag = "Catalog - Units",
    request_body = CreateUnitOfMeasurePayload,
    responses((status = 201, description = "Unit created", body = UnitOfMeasureDto), (status = 409, description = "Symbol already exists"))
)]
pub async fn create_unit_of_measure(_user: CurrentUser, State(state): State<AppState>, Json(payload): Json<CreateUnitOfMeasurePayload>) -> Result<(StatusCode, Json<UnitOfMeasureDto>), (StatusCode, String)> {
    svc(&state).create_unit_of_measure(payload).await.map(|u| (StatusCode::CREATED, Json(u))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/units-of-measure/{id}", tag = "Catalog - Units",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Unit found", body = UnitOfMeasureDto), (status = 404, description = "Not found"))
)]
pub async fn get_unit_of_measure(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<UnitOfMeasureDto>, (StatusCode, String)> {
    svc(&state).get_unit_of_measure(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/units-of-measure", tag = "Catalog - Units",
    params(ListQuery),
    responses((status = 200, description = "List of units", body = UnitsOfMeasureListResponse))
)]
pub async fn list_units_of_measure(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<ListQuery>) -> Result<Json<UnitsOfMeasureListResponse>, (StatusCode, String)> {
    let (units, total) = svc(&state).list_units_of_measure(q.limit, q.offset, q.search).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(UnitsOfMeasureListResponse { units, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/units-of-measure/{id}", tag = "Catalog - Units",
    params(("id" = Uuid, Path,)),
    request_body = UpdateUnitOfMeasurePayload,
    responses((status = 200, description = "Unit updated", body = UnitOfMeasureDto), (status = 404, description = "Not found"))
)]
pub async fn update_unit_of_measure(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<UpdateUnitOfMeasurePayload>) -> Result<Json<UnitOfMeasureDto>, (StatusCode, String)> {
    svc(&state).update_unit_of_measure(id, payload).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/units-of-measure/{id}", tag = "Catalog - Units",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Unit deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_unit_of_measure(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_unit_of_measure(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// Unit Conversion Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/conversions", tag = "Catalog - Conversions",
    request_body = CreateUnitConversionPayload,
    responses((status = 201, description = "Conversion created", body = UnitConversionWithDetailsDto), (status = 400, description = "Validation error"))
)]
pub async fn create_unit_conversion(_user: CurrentUser, State(state): State<AppState>, Json(payload): Json<CreateUnitConversionPayload>) -> Result<(StatusCode, Json<UnitConversionWithDetailsDto>), (StatusCode, String)> {
    let c = svc(&state).create_unit_conversion(payload).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_unit_conversion(c.id).await.map(|c| (StatusCode::CREATED, Json(c))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/conversions/{id}", tag = "Catalog - Conversions",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Conversion found", body = UnitConversionWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn get_unit_conversion(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<UnitConversionWithDetailsDto>, (StatusCode, String)> {
    svc(&state).get_unit_conversion(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/conversions", tag = "Catalog - Conversions",
    params(ConversionListQuery),
    responses((status = 200, description = "List of conversions", body = UnitConversionsListResponse))
)]
pub async fn list_unit_conversions(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<ConversionListQuery>) -> Result<Json<UnitConversionsListResponse>, (StatusCode, String)> {
    let (conversions, total) = svc(&state).list_unit_conversions(q.limit, q.offset, q.from_unit_id, q.to_unit_id).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(UnitConversionsListResponse { conversions, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/conversions/{id}", tag = "Catalog - Conversions",
    params(("id" = Uuid, Path,)),
    request_body = UpdateUnitConversionPayload,
    responses((status = 200, description = "Conversion updated", body = UnitConversionWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn update_unit_conversion(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<UpdateUnitConversionPayload>) -> Result<Json<UnitConversionWithDetailsDto>, (StatusCode, String)> {
    let _ = svc(&state).update_unit_conversion(id, payload).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_unit_conversion(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/conversions/{id}", tag = "Catalog - Conversions",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Conversion deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_unit_conversion(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_unit_conversion(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// CATMAT Group Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/catmat/groups", tag = "CATMAT - Groups",
    request_body = CreateCatmatGroupPayload,
    responses((status = 201, description = "Group created", body = CatmatGroupDto), (status = 409, description = "Code already exists"))
)]
pub async fn create_catmat_group(_user: CurrentUser, State(state): State<AppState>, Json(p): Json<CreateCatmatGroupPayload>) -> Result<(StatusCode, Json<CatmatGroupDto>), (StatusCode, String)> {
    svc(&state).create_catmat_group(p).await.map(|g| (StatusCode::CREATED, Json(g))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/groups/{id}", tag = "CATMAT - Groups",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Group found", body = CatmatGroupDto), (status = 404, description = "Not found"))
)]
pub async fn get_catmat_group(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CatmatGroupDto>, (StatusCode, String)> {
    svc(&state).get_catmat_group(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/groups", tag = "CATMAT - Groups",
    params(GroupListQuery),
    responses((status = 200, description = "List of CATMAT groups", body = CatmatGroupsListResponse))
)]
pub async fn list_catmat_groups(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<GroupListQuery>) -> Result<Json<CatmatGroupsListResponse>, (StatusCode, String)> {
    let (groups, total) = svc(&state).list_catmat_groups(q.limit, q.offset, q.search, q.is_active).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(CatmatGroupsListResponse { groups, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/catmat/groups/{id}", tag = "CATMAT - Groups",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatmatGroupPayload,
    responses((status = 200, description = "Group updated", body = CatmatGroupDto), (status = 404, description = "Not found"))
)]
pub async fn update_catmat_group(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateCatmatGroupPayload>) -> Result<Json<CatmatGroupDto>, (StatusCode, String)> {
    svc(&state).update_catmat_group(id, p).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/catmat/groups/{id}", tag = "CATMAT - Groups",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Group deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_catmat_group(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_catmat_group(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/groups/tree", tag = "CATMAT - Groups",
    responses((status = 200, description = "CATMAT hierarchical tree", body = Vec<CatmatGroupTreeNode>))
)]
pub async fn get_catmat_tree(_user: CurrentUser, State(state): State<AppState>) -> Result<Json<Vec<CatmatGroupTreeNode>>, (StatusCode, String)> {
    svc(&state).get_catmat_tree().await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// CATMAT Class Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/catmat/classes", tag = "CATMAT - Classes",
    request_body = CreateCatmatClassPayload,
    responses((status = 201, description = "Class created", body = CatmatClassWithDetailsDto), (status = 409, description = "Code already exists"))
)]
pub async fn create_catmat_class(_user: CurrentUser, State(state): State<AppState>, Json(p): Json<CreateCatmatClassPayload>) -> Result<(StatusCode, Json<CatmatClassWithDetailsDto>), (StatusCode, String)> {
    let c = svc(&state).create_catmat_class(p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catmat_class(c.id).await.map(|c| (StatusCode::CREATED, Json(c))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/classes/{id}", tag = "CATMAT - Classes",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Class found", body = CatmatClassWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn get_catmat_class(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CatmatClassWithDetailsDto>, (StatusCode, String)> {
    svc(&state).get_catmat_class(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/classes", tag = "CATMAT - Classes",
    params(ClassListQuery),
    responses((status = 200, description = "List of CATMAT classes", body = CatmatClassesListResponse))
)]
pub async fn list_catmat_classes(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<ClassListQuery>) -> Result<Json<CatmatClassesListResponse>, (StatusCode, String)> {
    let (classes, total) = svc(&state).list_catmat_classes(q.limit, q.offset, q.search, q.group_id, q.is_active).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(CatmatClassesListResponse { classes, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/catmat/classes/{id}", tag = "CATMAT - Classes",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatmatClassPayload,
    responses((status = 200, description = "Class updated", body = CatmatClassWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn update_catmat_class(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateCatmatClassPayload>) -> Result<Json<CatmatClassWithDetailsDto>, (StatusCode, String)> {
    let _ = svc(&state).update_catmat_class(id, p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catmat_class(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/catmat/classes/{id}", tag = "CATMAT - Classes",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Class deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_catmat_class(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_catmat_class(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// CATMAT Item (PDM) Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/catmat/items", tag = "CATMAT - Items",
    request_body = CreateCatmatItemPayload,
    responses((status = 201, description = "Item created", body = CatmatItemWithDetailsDto), (status = 409, description = "Code already exists"))
)]
pub async fn create_catmat_item(_user: CurrentUser, State(state): State<AppState>, Json(p): Json<CreateCatmatItemPayload>) -> Result<(StatusCode, Json<CatmatItemWithDetailsDto>), (StatusCode, String)> {
    let i = svc(&state).create_catmat_item(p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catmat_item(i.id).await.map(|i| (StatusCode::CREATED, Json(i))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/items/{id}", tag = "CATMAT - Items",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Item found", body = CatmatItemWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn get_catmat_item(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CatmatItemWithDetailsDto>, (StatusCode, String)> {
    svc(&state).get_catmat_item(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catmat/items", tag = "CATMAT - Items",
    params(CatmatItemListQuery),
    responses((status = 200, description = "List of CATMAT items", body = CatmatItemsListResponse))
)]
pub async fn list_catmat_items(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<CatmatItemListQuery>) -> Result<Json<CatmatItemsListResponse>, (StatusCode, String)> {
    let (items, total) = svc(&state).list_catmat_items(q.limit, q.offset, q.search, q.class_id, q.is_sustainable, q.is_permanent, q.is_active).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(CatmatItemsListResponse { items, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/catmat/items/{id}", tag = "CATMAT - Items",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatmatItemPayload,
    responses((status = 200, description = "Item updated", body = CatmatItemWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn update_catmat_item(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateCatmatItemPayload>) -> Result<Json<CatmatItemWithDetailsDto>, (StatusCode, String)> {
    let _ = svc(&state).update_catmat_item(id, p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catmat_item(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/catmat/items/{id}", tag = "CATMAT - Items",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Item deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_catmat_item(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_catmat_item(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// CATSER Group Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/catser/groups", tag = "CATSER - Groups",
    request_body = CreateCatserGroupPayload,
    responses((status = 201, description = "Group created", body = CatserGroupDto), (status = 409, description = "Code already exists"))
)]
pub async fn create_catser_group(_user: CurrentUser, State(state): State<AppState>, Json(p): Json<CreateCatserGroupPayload>) -> Result<(StatusCode, Json<CatserGroupDto>), (StatusCode, String)> {
    svc(&state).create_catser_group(p).await.map(|g| (StatusCode::CREATED, Json(g))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/groups/{id}", tag = "CATSER - Groups",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Group found", body = CatserGroupDto), (status = 404, description = "Not found"))
)]
pub async fn get_catser_group(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CatserGroupDto>, (StatusCode, String)> {
    svc(&state).get_catser_group(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/groups", tag = "CATSER - Groups",
    params(GroupListQuery),
    responses((status = 200, description = "List of CATSER groups", body = CatserGroupsListResponse))
)]
pub async fn list_catser_groups(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<GroupListQuery>) -> Result<Json<CatserGroupsListResponse>, (StatusCode, String)> {
    let (groups, total) = svc(&state).list_catser_groups(q.limit, q.offset, q.search, q.is_active).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(CatserGroupsListResponse { groups, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/catser/groups/{id}", tag = "CATSER - Groups",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatserGroupPayload,
    responses((status = 200, description = "Group updated", body = CatserGroupDto), (status = 404, description = "Not found"))
)]
pub async fn update_catser_group(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateCatserGroupPayload>) -> Result<Json<CatserGroupDto>, (StatusCode, String)> {
    svc(&state).update_catser_group(id, p).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/catser/groups/{id}", tag = "CATSER - Groups",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Group deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_catser_group(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_catser_group(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/groups/tree", tag = "CATSER - Groups",
    responses((status = 200, description = "CATSER hierarchical tree", body = Vec<CatserGroupTreeNode>))
)]
pub async fn get_catser_tree(_user: CurrentUser, State(state): State<AppState>) -> Result<Json<Vec<CatserGroupTreeNode>>, (StatusCode, String)> {
    svc(&state).get_catser_tree().await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// CATSER Class Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/catser/classes", tag = "CATSER - Classes",
    request_body = CreateCatserClassPayload,
    responses((status = 201, description = "Class created", body = CatserClassWithDetailsDto), (status = 409, description = "Code already exists"))
)]
pub async fn create_catser_class(_user: CurrentUser, State(state): State<AppState>, Json(p): Json<CreateCatserClassPayload>) -> Result<(StatusCode, Json<CatserClassWithDetailsDto>), (StatusCode, String)> {
    let c = svc(&state).create_catser_class(p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catser_class(c.id).await.map(|c| (StatusCode::CREATED, Json(c))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/classes/{id}", tag = "CATSER - Classes",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Class found", body = CatserClassWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn get_catser_class(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CatserClassWithDetailsDto>, (StatusCode, String)> {
    svc(&state).get_catser_class(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/classes", tag = "CATSER - Classes",
    params(ClassListQuery),
    responses((status = 200, description = "List of CATSER classes", body = CatserClassesListResponse))
)]
pub async fn list_catser_classes(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<ClassListQuery>) -> Result<Json<CatserClassesListResponse>, (StatusCode, String)> {
    let (classes, total) = svc(&state).list_catser_classes(q.limit, q.offset, q.search, q.group_id, q.is_active).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(CatserClassesListResponse { classes, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/catser/classes/{id}", tag = "CATSER - Classes",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatserClassPayload,
    responses((status = 200, description = "Class updated", body = CatserClassWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn update_catser_class(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateCatserClassPayload>) -> Result<Json<CatserClassWithDetailsDto>, (StatusCode, String)> {
    let _ = svc(&state).update_catser_class(id, p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catser_class(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/catser/classes/{id}", tag = "CATSER - Classes",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Class deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_catser_class(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_catser_class(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ============================
// CATSER Item (Serviço) Handlers
// ============================

#[utoipa::path(post, path = "/api/admin/catalog/catser/items", tag = "CATSER - Items",
    request_body = CreateCatserItemPayload,
    responses((status = 201, description = "Item created", body = CatserItemWithDetailsDto), (status = 409, description = "Code already exists"))
)]
pub async fn create_catser_item(_user: CurrentUser, State(state): State<AppState>, Json(p): Json<CreateCatserItemPayload>) -> Result<(StatusCode, Json<CatserItemWithDetailsDto>), (StatusCode, String)> {
    let i = svc(&state).create_catser_item(p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catser_item(i.id).await.map(|i| (StatusCode::CREATED, Json(i))).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/items/{id}", tag = "CATSER - Items",
    params(("id" = Uuid, Path,)),
    responses((status = 200, description = "Item found", body = CatserItemWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn get_catser_item(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<CatserItemWithDetailsDto>, (StatusCode, String)> {
    svc(&state).get_catser_item(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(get, path = "/api/admin/catalog/catser/items", tag = "CATSER - Items",
    params(CatserItemListQuery),
    responses((status = 200, description = "List of CATSER items", body = CatserItemsListResponse))
)]
pub async fn list_catser_items(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<CatserItemListQuery>) -> Result<Json<CatserItemsListResponse>, (StatusCode, String)> {
    let (items, total) = svc(&state).list_catser_items(q.limit, q.offset, q.search, q.class_id, q.is_active).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(CatserItemsListResponse { items, total, limit: q.limit, offset: q.offset }))
}

#[utoipa::path(put, path = "/api/admin/catalog/catser/items/{id}", tag = "CATSER - Items",
    params(("id" = Uuid, Path,)),
    request_body = UpdateCatserItemPayload,
    responses((status = 200, description = "Item updated", body = CatserItemWithDetailsDto), (status = 404, description = "Not found"))
)]
pub async fn update_catser_item(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<UpdateCatserItemPayload>) -> Result<Json<CatserItemWithDetailsDto>, (StatusCode, String)> {
    let _ = svc(&state).update_catser_item(id, p).await.map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    svc(&state).get_catser_item(id).await.map(Json).map_err(|e| (StatusCode::from(&e), e.to_string()))
}

#[utoipa::path(delete, path = "/api/admin/catalog/catser/items/{id}", tag = "CATSER - Items",
    params(("id" = Uuid, Path,)),
    responses((status = 204, description = "Item deleted"), (status = 404, description = "Not found"))
)]
pub async fn delete_catser_item(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    svc(&state).delete_catser_item(id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::from(&e), e.to_string()))
}
