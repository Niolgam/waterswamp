use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::{
    BlockMaterialPayload, CreateMaterialGroupPayload, CreateMaterialPayload,
    CreateWarehousePayload, TransferStockPayload, UpdateMaterialGroupPayload,
    UpdateMaterialPayload, UpdateStockMaintenancePayload, UpdateWarehousePayload,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

// ============================
// Request/Response Contracts
// ============================

#[derive(Debug, Deserialize, Validate)]
pub struct ListQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StockEntryRequest {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub quantity: Decimal,
    pub unit_value: Decimal,
    pub document_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StockExitRequest {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub quantity: Decimal,
    pub document_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StockAdjustmentRequest {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub adjustment_quantity: Decimal,
    #[validate(length(min = 10, max = 500))]
    pub reason: String,
}

// ============================
// Material Group Handlers
// ============================

/// GET /api/warehouse/material-groups
pub async fn list_material_groups(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, AppError> {
    params.validate()?;

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let result = state
        .warehouse_service
        .list_material_groups(limit, offset, params.search, None, None)
        .await?;

    Ok(Json(json!(result)))
}

/// GET /api/warehouse/material-groups/:id
pub async fn get_material_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let group = state.warehouse_service.get_material_group(id).await?;
    Ok(Json(json!(group)))
}

/// POST /api/warehouse/material-groups
pub async fn create_material_group(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(payload): Json<CreateMaterialGroupPayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let group = state.warehouse_service.create_material_group(payload).await?;

    Ok((StatusCode::CREATED, Json(json!(group))))
}

/// PUT /api/warehouse/material-groups/:id
pub async fn update_material_group(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMaterialGroupPayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let group = state
        .warehouse_service
        .update_material_group(id, payload)
        .await?;

    Ok(Json(json!(group)))
}

/// DELETE /api/warehouse/material-groups/:id
pub async fn delete_material_group(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.warehouse_service.delete_material_group(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Material Handlers
// ============================

/// GET /api/warehouse/materials
pub async fn list_materials(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, AppError> {
    params.validate()?;

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let result = state
        .warehouse_service
        .list_materials(limit, offset, params.search, None, None)
        .await?;

    Ok(Json(json!(result)))
}

/// GET /api/warehouse/materials/:id
pub async fn get_material(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let material = state.warehouse_service.get_material_with_group(id).await?;
    Ok(Json(json!(material)))
}

/// POST /api/warehouse/materials
pub async fn create_material(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(payload): Json<CreateMaterialPayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let material = state.warehouse_service.create_material(payload).await?;

    Ok((StatusCode::CREATED, Json(json!(material))))
}

/// PUT /api/warehouse/materials/:id
pub async fn update_material(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMaterialPayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let material = state.warehouse_service.update_material(id, payload).await?;

    Ok(Json(json!(material)))
}

/// DELETE /api/warehouse/materials/:id
pub async fn delete_material(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.warehouse_service.delete_material(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================
// Warehouse Handlers
// ============================

/// POST /api/warehouse/warehouses
pub async fn create_warehouse(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(payload): Json<CreateWarehousePayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let warehouse = state.warehouse_service.create_warehouse(payload).await?;

    Ok((StatusCode::CREATED, Json(json!(warehouse))))
}

/// GET /api/warehouse/warehouses/:id
pub async fn get_warehouse(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let warehouse = state.warehouse_service.get_warehouse_with_city(id).await?;
    Ok(Json(json!(warehouse)))
}

/// PUT /api/warehouse/warehouses/:id
pub async fn update_warehouse(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateWarehousePayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let warehouse = state.warehouse_service.update_warehouse(id, payload).await?;

    Ok(Json(json!(warehouse)))
}

// ============================
// Stock Movement Handlers
// ============================

/// POST /api/warehouse/stock/entry
pub async fn register_stock_entry(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<StockEntryRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    req.validate()?;

    let (stock, movement) = state
        .warehouse_service
        .register_stock_entry(
            req.warehouse_id,
            req.material_id,
            req.quantity,
            req.unit_value,
            user.id,
            req.document_number.as_deref(),
            req.notes.as_deref(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "stock": stock,
            "movement": movement,
            "message": "Entrada registrada com sucesso"
        })),
    ))
}

/// POST /api/warehouse/stock/exit
pub async fn register_stock_exit(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<StockExitRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    req.validate()?;

    let (stock, movement) = state
        .warehouse_service
        .register_stock_exit(
            req.warehouse_id,
            req.material_id,
            req.quantity,
            user.id,
            req.document_number.as_deref(),
            req.requisition_id,
            req.notes.as_deref(),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "stock": stock,
            "movement": movement,
            "message": "Saída registrada com sucesso"
        })),
    ))
}

/// POST /api/warehouse/stock/adjustment
pub async fn register_stock_adjustment(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<StockAdjustmentRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    req.validate()?;

    let (stock, movement) = state
        .warehouse_service
        .register_stock_adjustment(
            req.warehouse_id,
            req.material_id,
            req.adjustment_quantity,
            &req.reason,
            user.id,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "stock": stock,
            "movement": movement,
            "message": "Ajuste registrado com sucesso"
        })),
    ))
}

/// GET /api/warehouse/stock/:id
pub async fn get_warehouse_stock(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let stock = state.warehouse_service.get_warehouse_stock(id).await?;
    Ok(Json(json!(stock)))
}

// ============================
// Stock Maintenance Handlers
// ============================

/// PUT /api/warehouse/stock/:id/maintenance
/// Atualiza parâmetros de manutenção do estoque (min stock, resupply days, location)
pub async fn update_stock_maintenance(
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
    Json(payload): Json<UpdateStockMaintenancePayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let updated_stock = state
        .warehouse_service
        .update_stock_maintenance(stock_id, payload)
        .await?;

    Ok(Json(json!({
        "stock": updated_stock,
        "message": "Manutenção de estoque atualizada com sucesso"
    })))
}

/// POST /api/warehouse/stock/:id/block
/// Bloqueia um material, impedindo requisições
pub async fn block_material(
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
    user: CurrentUser,
    Json(payload): Json<BlockMaterialPayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let blocked_stock = state
        .warehouse_service
        .block_material(stock_id, payload, user.id)
        .await?;

    Ok(Json(json!({
        "stock": blocked_stock,
        "message": "Material bloqueado com sucesso"
    })))
}

/// DELETE /api/warehouse/stock/:id/block
/// Desbloqueia um material, permitindo requisições novamente
pub async fn unblock_material(
    State(state): State<AppState>,
    Path(stock_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let unblocked_stock = state.warehouse_service.unblock_material(stock_id).await?;

    Ok(Json(json!({
        "stock": unblocked_stock,
        "message": "Material desbloqueado com sucesso"
    })))
}

/// POST /api/warehouse/stock/transfer
/// Transfere estoque de um material para outro dentro do mesmo grupo
pub async fn transfer_stock(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(payload): Json<TransferStockPayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let (from_movement, to_movement) = state
        .warehouse_service
        .transfer_stock(payload, user.id)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "from_movement": from_movement,
            "to_movement": to_movement,
            "message": "Transferência de estoque realizada com sucesso"
        })),
    ))
}
