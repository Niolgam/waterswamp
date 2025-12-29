use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{CatmatCode, MaterialCode, UnitOfMeasure};

// ============================
// Enums
// ============================

/// Tipo de movimentação de estoque
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "movement_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MovementType {
    /// Entrada de material (compra, doação)
    #[sqlx(rename = "ENTRADA")]
    Entry,
    /// Saída de material (requisição atendida)
    #[sqlx(rename = "SAIDA")]
    Exit,
    /// Ajuste de estoque (inventário, correção)
    #[sqlx(rename = "AJUSTE")]
    Adjustment,
    /// Transferência entre almoxarifados (saída de origem)
    #[sqlx(rename = "TRANSFERENCIA_SAIDA")]
    TransferOut,
    /// Transferência entre almoxarifados (entrada em destino)
    #[sqlx(rename = "TRANSFERENCIA_ENTRADA")]
    TransferIn,
    /// Perda ou extravio
    #[sqlx(rename = "PERDA")]
    Loss,
    /// Devolução (material não utilizado)
    #[sqlx(rename = "DEVOLUCAO")]
    Return,
}

/// Status da requisição de materiais
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "requisition_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequisitionStatus {
    /// Requisição criada, aguardando aprovação
    #[sqlx(rename = "PENDENTE")]
    Pending,
    /// Aprovada pelo responsável
    #[sqlx(rename = "APROVADA")]
    Approved,
    /// Rejeitada
    #[sqlx(rename = "REJEITADA")]
    Rejected,
    /// Em atendimento pelo almoxarife
    #[sqlx(rename = "EM_ATENDIMENTO")]
    InProgress,
    /// Atendida (materiais entregues)
    #[sqlx(rename = "ATENDIDA")]
    Fulfilled,
    /// Atendida parcialmente (falta de estoque)
    #[sqlx(rename = "ATENDIDA_PARCIALMENTE")]
    PartiallyFulfilled,
    /// Cancelada
    #[sqlx(rename = "CANCELADA")]
    Cancelled,
}

// ============================
// Material Group Models
// ============================

/// DTO completo do Grupo de Material retornado do banco de dados
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MaterialGroupDto {
    pub id: Uuid,
    pub code: MaterialCode,
    pub name: String,
    pub description: Option<String>,
    pub expense_element: Option<String>,
    pub is_personnel_exclusive: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resposta paginada de grupos de materiais
#[derive(Debug, Serialize)]
pub struct PaginatedMaterialGroups {
    pub material_groups: Vec<MaterialGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query params para listagem de grupos de materiais
#[derive(Debug, Deserialize)]
pub struct ListMaterialGroupsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub is_personnel_exclusive: Option<bool>,
    pub is_active: Option<bool>,
}

/// Payload para criação de grupo de material
#[derive(Debug, Validate, Deserialize)]
pub struct CreateMaterialGroupPayload {
    pub code: MaterialCode,
    #[validate(length(min = 3, max = 200, message = "Nome deve ter entre 3 e 200 caracteres"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Descrição deve ter no máximo 1000 caracteres"))]
    pub description: Option<String>,
    #[validate(length(max = 200, message = "Elemento de despesa deve ter no máximo 200 caracteres"))]
    pub expense_element: Option<String>,
    pub is_personnel_exclusive: Option<bool>,
}

/// Payload para atualização de grupo de material
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateMaterialGroupPayload {
    pub code: Option<MaterialCode>,
    #[validate(length(min = 3, max = 200, message = "Nome deve ter entre 3 e 200 caracteres"))]
    pub name: Option<String>,
    #[validate(length(max = 1000, message = "Descrição deve ter no máximo 1000 caracteres"))]
    pub description: Option<String>,
    #[validate(length(max = 200, message = "Elemento de despesa deve ter no máximo 200 caracteres"))]
    pub expense_element: Option<String>,
    pub is_personnel_exclusive: Option<bool>,
    pub is_active: Option<bool>,
}

// ============================
// Material Models
// ============================

/// DTO completo do Material/Serviço retornado do banco de dados
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MaterialDto {
    pub id: Uuid,
    pub material_group_id: Uuid,
    pub name: String,
    pub estimated_value: rust_decimal::Decimal,
    pub unit_of_measure: UnitOfMeasure,
    pub specification: String,
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    pub photo_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO com informações do grupo de material incluídas
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MaterialWithGroupDto {
    pub id: Uuid,
    pub material_group_id: Uuid,
    pub material_group_code: MaterialCode,
    pub material_group_name: String,
    pub name: String,
    pub estimated_value: rust_decimal::Decimal,
    pub unit_of_measure: UnitOfMeasure,
    pub specification: String,
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    pub photo_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resposta paginada de materiais
#[derive(Debug, Serialize)]
pub struct PaginatedMaterials {
    pub materials: Vec<MaterialWithGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query params para listagem de materiais
#[derive(Debug, Deserialize)]
pub struct ListMaterialsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub material_group_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

/// Payload para criação de material/serviço
#[derive(Debug, Validate, Deserialize)]
pub struct CreateMaterialPayload {
    pub material_group_id: Uuid,
    #[validate(length(min = 3, max = 200, message = "Denominação deve ter entre 3 e 200 caracteres"))]
    pub name: String,
    pub estimated_value: rust_decimal::Decimal, // Validated in service layer
    pub unit_of_measure: UnitOfMeasure,
    #[validate(length(min = 10, max = 2000, message = "Especificação deve ter entre 10 e 2000 caracteres"))]
    pub specification: String,
    #[validate(length(max = 1000, message = "Links de busca devem ter no máximo 1000 caracteres"))]
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    #[validate(url(message = "URL da foto inválida"))]
    pub photo_url: Option<String>,
}

/// Payload para atualização de material/serviço
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateMaterialPayload {
    pub material_group_id: Option<Uuid>,
    #[validate(length(min = 3, max = 200, message = "Denominação deve ter entre 3 e 200 caracteres"))]
    pub name: Option<String>,
    pub estimated_value: Option<rust_decimal::Decimal>, // Validated in service layer
    pub unit_of_measure: Option<UnitOfMeasure>,
    #[validate(length(min = 10, max = 2000, message = "Especificação deve ter entre 10 e 2000 caracteres"))]
    pub specification: Option<String>,
    #[validate(length(max = 1000, message = "Links de busca devem ter no máximo 1000 caracteres"))]
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    #[validate(url(message = "URL da foto inválida"))]
    pub photo_url: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Warehouse Models (Almoxarifado)
// ============================

/// DTO completo do Almoxarifado
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WarehouseDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub city_id: Uuid,
    pub responsible_user_id: Option<Uuid>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO do Almoxarifado com informações da cidade
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WarehouseWithCityDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub city_id: Uuid,
    pub city_name: String,
    pub state_code: String,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_username: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedWarehouses {
    pub warehouses: Vec<WarehouseWithCityDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListWarehousesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub city_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateWarehousePayload {
    #[validate(length(min = 3, max = 200, message = "Nome deve ter entre 3 e 200 caracteres"))]
    pub name: String,
    #[validate(length(min = 2, max = 50, message = "Código deve ter entre 2 e 50 caracteres"))]
    pub code: String,
    pub city_id: Uuid,
    pub responsible_user_id: Option<Uuid>,
    #[validate(length(max = 500, message = "Endereço deve ter no máximo 500 caracteres"))]
    pub address: Option<String>,
    #[validate(length(max = 20, message = "Telefone deve ter no máximo 20 caracteres"))]
    pub phone: Option<String>,
    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateWarehousePayload {
    #[validate(length(min = 3, max = 200, message = "Nome deve ter entre 3 e 200 caracteres"))]
    pub name: Option<String>,
    #[validate(length(min = 2, max = 50, message = "Código deve ter entre 2 e 50 caracteres"))]
    pub code: Option<String>,
    pub city_id: Option<Uuid>,
    pub responsible_user_id: Option<Uuid>,
    #[validate(length(max = 500, message = "Endereço deve ter no máximo 500 caracteres"))]
    pub address: Option<String>,
    #[validate(length(max = 20, message = "Telefone deve ter no máximo 20 caracteres"))]
    pub phone: Option<String>,
    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

// ============================
// Warehouse Stock Models (Estoque por Almoxarifado)
// ============================

/// DTO completo do Estoque por Almoxarifado
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WarehouseStockDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub quantity: rust_decimal::Decimal,
    pub average_unit_value: rust_decimal::Decimal,
    pub min_stock: Option<rust_decimal::Decimal>,
    pub max_stock: Option<rust_decimal::Decimal>,
    pub location: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO com informações completas (warehouse + material)
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WarehouseStockWithDetailsDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub material_id: Uuid,
    pub material_name: String,
    pub material_group_name: String,
    pub unit_of_measure: UnitOfMeasure,
    pub quantity: rust_decimal::Decimal,
    pub average_unit_value: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal, // quantity * average_unit_value
    pub min_stock: Option<rust_decimal::Decimal>,
    pub max_stock: Option<rust_decimal::Decimal>,
    pub location: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedWarehouseStocks {
    pub stocks: Vec<WarehouseStockWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListWarehouseStocksQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub warehouse_id: Option<Uuid>,
    pub material_id: Option<Uuid>,
    pub search: Option<String>,
    pub low_stock: Option<bool>, // Filtrar apenas itens com estoque baixo
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateWarehouseStockPayload {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub quantity: rust_decimal::Decimal, // Validated in service layer (>= 0)
    pub average_unit_value: rust_decimal::Decimal, // Validated in service layer (>= 0)
    pub min_stock: Option<rust_decimal::Decimal>,
    pub max_stock: Option<rust_decimal::Decimal>,
    #[validate(length(max = 100, message = "Localização deve ter no máximo 100 caracteres"))]
    pub location: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateWarehouseStockPayload {
    pub min_stock: Option<rust_decimal::Decimal>,
    pub max_stock: Option<rust_decimal::Decimal>,
    #[validate(length(max = 100, message = "Localização deve ter no máximo 100 caracteres"))]
    pub location: Option<String>,
}

// ============================
// Stock Movement Models (Movimentações de Estoque)
// ============================

/// DTO completo da Movimentação de Estoque
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct StockMovementDto {
    pub id: Uuid,
    pub warehouse_stock_id: Uuid,
    pub movement_type: MovementType,
    pub quantity: rust_decimal::Decimal,
    pub unit_value: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub balance_before: rust_decimal::Decimal,
    pub balance_after: rust_decimal::Decimal,
    pub average_before: rust_decimal::Decimal,
    pub average_after: rust_decimal::Decimal,
    pub movement_date: chrono::DateTime<chrono::Utc>,
    pub document_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub user_id: Uuid,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// DTO com informações completas
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct StockMovementWithDetailsDto {
    pub id: Uuid,
    pub warehouse_stock_id: Uuid,
    pub warehouse_name: String,
    pub material_name: String,
    pub unit_of_measure: UnitOfMeasure,
    pub movement_type: MovementType,
    pub quantity: rust_decimal::Decimal,
    pub unit_value: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub balance_before: rust_decimal::Decimal,
    pub balance_after: rust_decimal::Decimal,
    pub average_before: rust_decimal::Decimal,
    pub average_after: rust_decimal::Decimal,
    pub movement_date: chrono::DateTime<chrono::Utc>,
    pub document_number: Option<String>,
    pub requisition_id: Option<Uuid>,
    pub user_id: Uuid,
    pub username: String,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedStockMovements {
    pub movements: Vec<StockMovementWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListStockMovementsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub warehouse_id: Option<Uuid>,
    pub material_id: Option<Uuid>,
    pub movement_type: Option<MovementType>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateStockMovementPayload {
    pub warehouse_id: Uuid,
    pub material_id: Uuid,
    pub movement_type: MovementType,
    pub quantity: rust_decimal::Decimal,
    pub unit_value: rust_decimal::Decimal, // Validated in service layer (>= 0)
    pub movement_date: Option<chrono::DateTime<chrono::Utc>>,
    #[validate(length(max = 100, message = "Número do documento deve ter no máximo 100 caracteres"))]
    pub document_number: Option<String>,
    #[validate(length(max = 1000, message = "Observações devem ter no máximo 1000 caracteres"))]
    pub notes: Option<String>,
}

// ============================
// Requisition Models (Requisições de Materiais)
// ============================

/// DTO completo da Requisição
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct RequisitionDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub requester_id: Uuid,
    pub status: RequisitionStatus,
    pub total_value: rust_decimal::Decimal,
    pub request_date: chrono::DateTime<chrono::Utc>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub fulfilled_by: Option<Uuid>,
    pub fulfilled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO com informações completas
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequisitionWithDetailsDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub requester_id: Uuid,
    pub requester_username: String,
    pub status: RequisitionStatus,
    pub total_value: rust_decimal::Decimal,
    pub request_date: chrono::DateTime<chrono::Utc>,
    pub approved_by: Option<Uuid>,
    pub approved_by_username: Option<String>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub fulfilled_by: Option<Uuid>,
    pub fulfilled_by_username: Option<String>,
    pub fulfilled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<RequisitionItemDto>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedRequisitions {
    pub requisitions: Vec<RequisitionWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListRequisitionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub warehouse_id: Option<Uuid>,
    pub requester_id: Option<Uuid>,
    pub status: Option<RequisitionStatus>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateRequisitionPayload {
    pub warehouse_id: Uuid,
    #[validate(length(min = 1, message = "Requisição deve ter pelo menos um item"))]
    pub items: Vec<CreateRequisitionItemPayload>,
    #[validate(length(max = 1000, message = "Observações devem ter no máximo 1000 caracteres"))]
    pub notes: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ApproveRequisitionPayload {
    #[validate(length(max = 1000, message = "Observações devem ter no máximo 1000 caracteres"))]
    pub notes: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct RejectRequisitionPayload {
    #[validate(length(min = 10, max = 1000, message = "Motivo da rejeição deve ter entre 10 e 1000 caracteres"))]
    pub rejection_reason: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct FulfillRequisitionPayload {
    pub items: Vec<FulfillRequisitionItemPayload>,
    #[validate(length(max = 1000, message = "Observações devem ter no máximo 1000 caracteres"))]
    pub notes: Option<String>,
}

// ============================
// Requisition Item Models (Itens da Requisição)
// ============================

/// DTO completo do Item da Requisição
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct RequisitionItemDto {
    pub id: Uuid,
    pub requisition_id: Uuid,
    pub material_id: Uuid,
    pub material_name: String,
    pub unit_of_measure: UnitOfMeasure,
    pub requested_quantity: rust_decimal::Decimal,
    pub fulfilled_quantity: Option<rust_decimal::Decimal>,
    pub unit_value: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateRequisitionItemPayload {
    pub material_id: Uuid,
    pub requested_quantity: rust_decimal::Decimal, // Validated in service layer (> 0)
}

#[derive(Debug, Validate, Deserialize)]
pub struct FulfillRequisitionItemPayload {
    pub requisition_item_id: Uuid,
    pub fulfilled_quantity: rust_decimal::Decimal, // Validated in service layer (>= 0)
}
