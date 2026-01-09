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
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

// ============================
// Request/Response Contracts
// ============================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct StockEntryRequest {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub quantity: Decimal,
    pub unit_value: Decimal,
    pub document_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct StockExitRequest {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub quantity: Decimal,
    pub document_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
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
#[utoipa::path(
    get,
    path = "/api/warehouse/material-groups",
    tag = "Material Groups",
    params(
        ("limit" = Option<i64>, Query, description = "Número de itens por página (1-100)", example = 20),
        ("offset" = Option<i64>, Query, description = "Deslocamento para paginação", example = 0),
        ("search" = Option<String>, Query, description = "Busca por código ou nome do grupo"),
    ),
    responses(
        (status = 200, description = "Lista de grupos de materiais retornada com sucesso"),
        (status = 400, description = "Parâmetros de consulta inválidos"),
    ),
    summary = "Listar grupos de materiais"
)]
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
#[utoipa::path(
    get,
    path = "/api/warehouse/material-groups/{id}",
    tag = "Material Groups",
    params(
        ("id" = Uuid, Path, description = "ID do grupo de materiais"),
    ),
    responses(
        (status = 200, description = "Grupo de materiais encontrado"),
        (status = 404, description = "Grupo de materiais não encontrado"),
    ),
    summary = "Obter grupo de materiais por ID"
)]
pub async fn get_material_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let group = state.warehouse_service.get_material_group(id).await?;
    Ok(Json(json!({"material_group": group})))
}

/// POST /api/warehouse/material-groups
#[utoipa::path(
    post,
    path = "/api/warehouse/material-groups",
    tag = "Material Groups",
    request_body = CreateMaterialGroupPayload,
    responses(
        (status = 201, description = "Grupo de materiais criado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 409, description = "Código do grupo já existe"),
    ),
    summary = "Criar novo grupo de materiais",
    security(("bearer" = []))
)]
pub async fn create_material_group(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(payload): Json<CreateMaterialGroupPayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let group = state.warehouse_service.create_material_group(payload).await?;

    Ok((StatusCode::CREATED, Json(json!({"material_group": group}))))
}

/// PUT /api/warehouse/material-groups/:id
#[utoipa::path(
    put,
    path = "/api/warehouse/material-groups/{id}",
    tag = "Material Groups",
    params(
        ("id" = Uuid, Path, description = "ID do grupo de materiais"),
    ),
    request_body = UpdateMaterialGroupPayload,
    responses(
        (status = 200, description = "Grupo de materiais atualizado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Grupo de materiais não encontrado"),
        (status = 409, description = "Código do grupo já existe"),
    ),
    summary = "Atualizar grupo de materiais",
    security(("bearer" = []))
)]
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

    Ok(Json(json!({"material_group": group})))
}

/// DELETE /api/warehouse/material-groups/:id
#[utoipa::path(
    delete,
    path = "/api/warehouse/material-groups/{id}",
    tag = "Material Groups",
    params(
        ("id" = Uuid, Path, description = "ID do grupo de materiais"),
    ),
    responses(
        (status = 204, description = "Grupo de materiais excluído com sucesso"),
        (status = 404, description = "Grupo de materiais não encontrado"),
        (status = 409, description = "Grupo possui materiais associados"),
    ),
    summary = "Excluir grupo de materiais",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    get,
    path = "/api/warehouse/materials",
    tag = "Materials",
    params(
        ("limit" = Option<i64>, Query, description = "Número de itens por página (1-100)", example = 20),
        ("offset" = Option<i64>, Query, description = "Deslocamento para paginação", example = 0),
        ("search" = Option<String>, Query, description = "Busca por nome ou código do material"),
    ),
    responses(
        (status = 200, description = "Lista de materiais retornada com sucesso"),
        (status = 400, description = "Parâmetros de consulta inválidos"),
    ),
    summary = "Listar materiais e serviços"
)]
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
#[utoipa::path(
    get,
    path = "/api/warehouse/materials/{id}",
    tag = "Materials",
    params(
        ("id" = Uuid, Path, description = "ID do material"),
    ),
    responses(
        (status = 200, description = "Material encontrado"),
        (status = 404, description = "Material não encontrado"),
    ),
    summary = "Obter material por ID"
)]
pub async fn get_material(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let material = state.warehouse_service.get_material_with_group(id).await?;
    Ok(Json(json!({"material": material})))
}

/// POST /api/warehouse/materials
#[utoipa::path(
    post,
    path = "/api/warehouse/materials",
    tag = "Materials",
    request_body = CreateMaterialPayload,
    responses(
        (status = 201, description = "Material criado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Grupo de materiais não encontrado"),
        (status = 409, description = "Material já existe no grupo"),
    ),
    summary = "Criar novo material ou serviço",
    security(("bearer" = []))
)]
pub async fn create_material(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(payload): Json<CreateMaterialPayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let material = state.warehouse_service.create_material(payload).await?;

    Ok((StatusCode::CREATED, Json(json!({"material": material}))))
}

/// PUT /api/warehouse/materials/:id
#[utoipa::path(
    put,
    path = "/api/warehouse/materials/{id}",
    tag = "Materials",
    params(
        ("id" = Uuid, Path, description = "ID do material"),
    ),
    request_body = UpdateMaterialPayload,
    responses(
        (status = 200, description = "Material atualizado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Material não encontrado"),
        (status = 409, description = "Nome já existe no grupo"),
    ),
    summary = "Atualizar material ou serviço",
    security(("bearer" = []))
)]
pub async fn update_material(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMaterialPayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let material = state.warehouse_service.update_material(id, payload).await?;

    Ok(Json(json!({"material": material})))
}

/// DELETE /api/warehouse/materials/:id
#[utoipa::path(
    delete,
    path = "/api/warehouse/materials/{id}",
    tag = "Materials",
    params(
        ("id" = Uuid, Path, description = "ID do material"),
    ),
    responses(
        (status = 204, description = "Material excluído com sucesso"),
        (status = 404, description = "Material não encontrado"),
        (status = 409, description = "Material possui estoque ou movimentações"),
    ),
    summary = "Excluir material ou serviço",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    post,
    path = "/api/warehouse/warehouses",
    tag = "Warehouses",
    request_body = CreateWarehousePayload,
    responses(
        (status = 201, description = "Almoxarifado criado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Cidade não encontrada"),
        (status = 409, description = "Código do almoxarifado já existe"),
    ),
    summary = "Criar novo almoxarifado",
    security(("bearer" = []))
)]
pub async fn create_warehouse(
    State(state): State<AppState>,
    _user: CurrentUser,
    Json(payload): Json<CreateWarehousePayload>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    payload.validate()?;

    let warehouse = state.warehouse_service.create_warehouse(payload).await?;

    Ok((StatusCode::CREATED, Json(json!({"warehouse": warehouse}))))
}

/// GET /api/warehouse/warehouses/:id
#[utoipa::path(
    get,
    path = "/api/warehouse/warehouses/{id}",
    tag = "Warehouses",
    params(
        ("id" = Uuid, Path, description = "ID do almoxarifado"),
    ),
    responses(
        (status = 200, description = "Almoxarifado encontrado"),
        (status = 404, description = "Almoxarifado não encontrado"),
    ),
    summary = "Obter almoxarifado por ID"
)]
pub async fn get_warehouse(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let warehouse = state.warehouse_service.get_warehouse_with_city(id).await?;
    Ok(Json(json!({"warehouse": warehouse})))
}

/// PUT /api/warehouse/warehouses/:id
#[utoipa::path(
    put,
    path = "/api/warehouse/warehouses/{id}",
    tag = "Warehouses",
    params(
        ("id" = Uuid, Path, description = "ID do almoxarifado"),
    ),
    request_body = UpdateWarehousePayload,
    responses(
        (status = 200, description = "Almoxarifado atualizado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Almoxarifado ou cidade não encontrados"),
        (status = 409, description = "Código do almoxarifado já existe"),
    ),
    summary = "Atualizar almoxarifado",
    security(("bearer" = []))
)]
pub async fn update_warehouse(
    State(state): State<AppState>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateWarehousePayload>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let warehouse = state.warehouse_service.update_warehouse(id, payload).await?;

    Ok(Json(json!({"warehouse": warehouse})))
}

// ============================
// Stock Movement Handlers
// ============================

/// POST /api/warehouse/stock/entry
#[utoipa::path(
    post,
    path = "/api/warehouse/stock/entry",
    tag = "Stock",
    request_body = StockEntryRequest,
    responses(
        (status = 201, description = "Entrada de estoque registrada com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Almoxarifado ou material não encontrado"),
    ),
    summary = "Registrar entrada de estoque",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    post,
    path = "/api/warehouse/stock/exit",
    tag = "Stock",
    request_body = StockExitRequest,
    responses(
        (status = 201, description = "Saída de estoque registrada com sucesso"),
        (status = 400, description = "Dados inválidos ou estoque insuficiente"),
        (status = 404, description = "Almoxarifado ou material não encontrado"),
    ),
    summary = "Registrar saída de estoque",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    post,
    path = "/api/warehouse/stock/adjustment",
    tag = "Stock",
    request_body = StockAdjustmentRequest,
    responses(
        (status = 201, description = "Ajuste de estoque registrado com sucesso"),
        (status = 400, description = "Dados inválidos ou ajuste resultaria em estoque negativo"),
        (status = 404, description = "Almoxarifado ou material não encontrado"),
    ),
    summary = "Registrar ajuste de estoque (positivo ou negativo)",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    get,
    path = "/api/warehouse/stock/{id}",
    tag = "Stock",
    params(
        ("id" = Uuid, Path, description = "ID do estoque"),
    ),
    responses(
        (status = 200, description = "Informações do estoque retornadas"),
        (status = 404, description = "Estoque não encontrado"),
    ),
    summary = "Obter informações de estoque específico"
)]
pub async fn get_warehouse_stock(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let stock = state.warehouse_service.get_warehouse_stock(id).await?;
    Ok(Json(json!({"stock": stock})))
}

// ============================
// Stock Maintenance Handlers
// ============================

/// PUT /api/warehouse/stock/:id/maintenance
/// Atualiza parâmetros de manutenção do estoque (min stock, resupply days, location)
#[utoipa::path(
    put,
    path = "/api/warehouse/stock/{id}/maintenance",
    tag = "Stock",
    params(
        ("id" = Uuid, Path, description = "ID do estoque"),
    ),
    request_body = UpdateStockMaintenancePayload,
    responses(
        (status = 200, description = "Parâmetros de manutenção atualizados com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Estoque não encontrado"),
    ),
    summary = "Atualizar parâmetros de manutenção do estoque",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    post,
    path = "/api/warehouse/stock/{id}/block",
    tag = "Stock",
    params(
        ("id" = Uuid, Path, description = "ID do estoque"),
    ),
    request_body = BlockMaterialPayload,
    responses(
        (status = 200, description = "Material bloqueado com sucesso"),
        (status = 400, description = "Dados inválidos"),
        (status = 404, description = "Estoque não encontrado"),
        (status = 409, description = "Material já está bloqueado"),
    ),
    summary = "Bloquear material (impede requisições)",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    delete,
    path = "/api/warehouse/stock/{id}/block",
    tag = "Stock",
    params(
        ("id" = Uuid, Path, description = "ID do estoque"),
    ),
    responses(
        (status = 200, description = "Material desbloqueado com sucesso"),
        (status = 404, description = "Estoque não encontrado"),
        (status = 409, description = "Material não está bloqueado"),
    ),
    summary = "Desbloquear material (permite requisições novamente)",
    security(("bearer" = []))
)]
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
#[utoipa::path(
    post,
    path = "/api/warehouse/stock/transfer",
    tag = "Stock",
    request_body = TransferStockPayload,
    responses(
        (status = 201, description = "Transferência realizada com sucesso"),
        (status = 400, description = "Dados inválidos ou estoque insuficiente"),
        (status = 404, description = "Estoque de origem ou destino não encontrado"),
        (status = 409, description = "Materiais não pertencem ao mesmo grupo"),
    ),
    summary = "Transferir estoque entre materiais do mesmo grupo",
    security(("bearer" = []))
)]
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
