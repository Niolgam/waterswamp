use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "warehouse_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarehouseType {
    Central,
    Sector,
}

// ============================
// Warehouse DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub warehouse_type: WarehouseType,
    pub city_id: Uuid,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: bool,
    pub is_budgetary: bool,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse with city and state names joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseWithDetailsDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub warehouse_type: WarehouseType,
    pub city_id: Uuid,
    pub city_name: Option<String>,
    pub state_abbreviation: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: bool,
    pub is_budgetary: bool,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// Warehouse Stock DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseStockDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,

    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    pub average_unit_value: Decimal,

    pub min_stock: Option<Decimal>,
    pub max_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub resupply_days: Option<i32>,

    pub location: Option<String>,
    pub secondary_location: Option<String>,

    pub is_blocked: bool,
    pub block_reason: Option<String>,
    pub blocked_at: Option<DateTime<Utc>>,
    pub blocked_by: Option<Uuid>,

    pub last_entry_at: Option<DateTime<Utc>>,
    pub last_exit_at: Option<DateTime<Utc>>,
    pub last_inventory_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse stock with catalog item name, unit, and warehouse name joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct WarehouseStockWithDetailsDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: Option<String>,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub catalog_item_code: Option<String>,
    pub unit_symbol: Option<String>,
    pub unit_name: Option<String>,

    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    /// quantity - reserved_quantity (when not blocked)
    pub available_quantity: Decimal,
    pub average_unit_value: Decimal,
    pub total_value: Decimal,

    pub min_stock: Option<Decimal>,
    pub max_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub resupply_days: Option<i32>,

    pub location: Option<String>,
    pub secondary_location: Option<String>,

    pub is_blocked: bool,
    pub block_reason: Option<String>,
    pub blocked_at: Option<DateTime<Utc>>,
    pub blocked_by: Option<Uuid>,

    pub last_entry_at: Option<DateTime<Utc>>,
    pub last_exit_at: Option<DateTime<Utc>>,
    pub last_inventory_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// Request Payloads
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWarehousePayload {
    pub name: String,
    pub code: String,
    pub warehouse_type: WarehouseType,
    pub city_id: Uuid,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: Option<bool>,
    pub is_budgetary: Option<bool>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateWarehousePayload {
    pub name: Option<String>,
    pub code: Option<String>,
    pub warehouse_type: Option<WarehouseType>,
    pub city_id: Option<Uuid>,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_unit_id: Option<Uuid>,
    pub allows_transfers: Option<bool>,
    pub is_budgetary: Option<bool>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

/// Update stock control parameters (min/max/reorder/location) — does NOT affect quantity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateStockParamsPayload {
    pub min_stock: Option<Decimal>,
    pub max_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
    pub resupply_days: Option<i32>,
    pub location: Option<String>,
    pub secondary_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockStockPayload {
    pub block_reason: String,
}

// ============================
// Stock Movement DTOs
// ============================

/// Tipo de movimento de estoque (espelha stock_movement_type_enum)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "stock_movement_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockMovementTypeDto {
    Entry,
    Exit,
    Loss,
    Return,
    TransferIn,
    TransferOut,
    AdjustmentAdd,
    AdjustmentSub,
    DonationIn,
    DonationOut,
}

/// DTO para listagem de movimentações de estoque
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct StockMovementDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub warehouse_name: Option<String>,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub catalog_item_code: Option<String>,
    pub movement_type: StockMovementTypeDto,
    pub movement_date: DateTime<Utc>,
    pub quantity_raw: Decimal,
    pub quantity_base: Decimal,
    pub unit_price_base: Decimal,
    pub total_value: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub average_before: Decimal,
    pub average_after: Decimal,
    pub invoice_id: Option<Uuid>,
    pub requisition_id: Option<Uuid>,
    pub related_warehouse_id: Option<Uuid>,
    pub related_warehouse_name: Option<String>,
    pub document_number: Option<String>,
    pub notes: Option<String>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub requires_review: bool,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================
// Standalone Entry (RF-009: Entrada Avulsa)
// ============================

/// Tipo de entrada avulsa permitida
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StandaloneEntryType {
    /// Entrada por doação (DONATION_IN)
    Donation,
    /// Ajuste de inventário — sobra (ADJUSTMENT_ADD)
    InventoryAdjustment,
}

/// Item de uma entrada avulsa
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StandaloneEntryItemPayload {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    /// Quantidade na unidade do documento
    pub quantity_raw: Decimal,
    /// Fator de conversão para unidade base (1.0 se não houver conversão)
    pub conversion_factor: Decimal,
    /// Preço unitário na unidade base
    pub unit_price_base: Decimal,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    /// Obrigatório quando preço diverge > 20% do custo médio
    pub divergence_justification: Option<String>,
    pub item_notes: Option<String>,
}

/// Payload para registrar entrada avulsa (RF-009)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StandaloneEntryPayload {
    pub entry_type: StandaloneEntryType,
    /// Origem: CPF/CNPJ do doador ou descrição da fonte (RF-009)
    pub origin_description: String,
    pub document_number: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<StandaloneEntryItemPayload>,
}

/// Resultado de uma entrada avulsa
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StandaloneEntryResult {
    pub movements_created: usize,
    pub entry_type: String,
    pub origin_description: String,
    pub warehouse_id: Uuid,
}

// ============================
// Return Entry (RF-011: Devolução de Requisição)
// ============================

/// Item de devolução de requisição
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReturnEntryItemPayload {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub item_notes: Option<String>,
}

/// Payload para registrar devolução de itens de requisição ao almoxarifado (RF-011)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReturnEntryPayload {
    /// ID da requisição original (obrigatório para rastreabilidade)
    pub requisition_id: Uuid,
    pub notes: Option<String>,
    pub items: Vec<ReturnEntryItemPayload>,
}

/// Resultado de uma devolução
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReturnEntryResult {
    pub movements_created: usize,
    pub requisition_id: Uuid,
    pub warehouse_id: Uuid,
}

// ============================
// Disposal Exit (RF-016: Saída por Desfazimento/Baixa)
// ============================

/// Item de saída por desfazimento
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisposalExitItemPayload {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub item_notes: Option<String>,
}

/// Payload para registrar saída por desfazimento/baixa (RF-016)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisposalExitPayload {
    /// Justificativa obrigatória (RN-005)
    pub justification: String,
    /// Número de processo SEI (formato validado por regex — RF-039)
    pub sei_process_number: String,
    /// URL do arquivo PDF do Parecer Técnico (RF-016)
    pub technical_opinion_url: String,
    pub notes: Option<String>,
    pub items: Vec<DisposalExitItemPayload>,
}

/// Resultado de saída por desfazimento
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisposalExitResult {
    pub movements_created: usize,
    pub sei_process_number: String,
    pub warehouse_id: Uuid,
}

// ============================
// Manual/OS Exit (RF-017: Saída por Ordem de Serviço)
// ============================

/// Item de saída manual / por OS
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManualExitItemPayload {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub item_notes: Option<String>,
}

/// Payload para registrar saída manual ou por Ordem de Serviço (RF-017)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManualExitPayload {
    /// Número da OS ou documento associado
    pub document_number: String,
    pub justification: String,
    pub notes: Option<String>,
    pub items: Vec<ManualExitItemPayload>,
}

/// Resultado de saída manual
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ManualExitResult {
    pub movements_created: usize,
    pub document_number: String,
    pub warehouse_id: Uuid,
}

// ============================
// Stock Transfer (RF-018: Transferência entre Almoxarifados)
// ============================

/// Status de uma transferência
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "stock_transfer_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockTransferStatus {
    Pending,
    /// RF-049: Aguardando assinatura digital Gov.br antes de efetivar o TRANSFER_IN.
    AwaitingGovbrSignature,
    Confirmed,
    Rejected,
    Cancelled,
    Expired,
}

/// Item de uma transferência
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct StockTransferItemDto {
    pub id: Uuid,
    pub transfer_id: Uuid,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub catalog_item_code: Option<String>,
    pub quantity_requested: Decimal,
    pub quantity_confirmed: Option<Decimal>,
    pub unit_raw_id: Uuid,
    pub unit_symbol: Option<String>,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub source_movement_id: Option<Uuid>,
    pub destination_movement_id: Option<Uuid>,
}

/// DTO de uma transferência entre almoxarifados
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct StockTransferDto {
    pub id: Uuid,
    pub transfer_number: String,
    pub source_warehouse_id: Uuid,
    pub source_warehouse_name: Option<String>,
    pub destination_warehouse_id: Uuid,
    pub destination_warehouse_name: Option<String>,
    pub status: StockTransferStatus,
    pub notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub cancellation_reason: Option<String>,
    pub initiated_by: Uuid,
    pub initiated_by_name: Option<String>,
    pub confirmed_by: Option<Uuid>,
    pub rejected_by: Option<Uuid>,
    pub cancelled_by: Option<Uuid>,
    pub initiated_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    /// RF-049: se verdadeiro, a confirmação do destino requer assinatura Gov.br antes do TRANSFER_IN.
    pub requires_govbr_signature: bool,
    pub govbr_signed_at: Option<DateTime<Utc>>,
    pub govbr_signed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO completo com itens
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StockTransferWithItemsDto {
    #[serde(flatten)]
    pub transfer: StockTransferDto,
    pub items: Vec<StockTransferItemDto>,
}

/// Item para iniciar transferência
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InitiateTransferItemPayload {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

/// Payload para iniciar transferência (RF-018 — passo 1)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InitiateTransferPayload {
    pub destination_warehouse_id: Uuid,
    /// Prazo em horas para o destino confirmar (None = sem prazo)
    pub expires_in_hours: Option<i64>,
    pub notes: Option<String>,
    /// RF-049: exige assinatura Gov.br antes de efetivar o TRANSFER_IN no destino.
    pub requires_govbr_signature: Option<bool>,
    pub items: Vec<InitiateTransferItemPayload>,
}

/// Item confirmado pelo destino (pode confirmar quantidade diferente)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfirmTransferItemInput {
    pub transfer_item_id: Uuid,
    /// Quantidade efetivamente recebida (pode ser <= quantity_requested)
    pub quantity_confirmed: Decimal,
}

/// Payload para confirmar recebimento (RF-018 — passo 2a)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfirmTransferPayload {
    pub items: Vec<ConfirmTransferItemInput>,
    pub notes: Option<String>,
}

/// Payload para rejeitar transferência (RF-018 — passo 2b)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RejectTransferPayload {
    pub rejection_reason: String,
}

/// Payload para cancelar transferência pela origem
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CancelTransferPayload {
    pub cancellation_reason: String,
}

/// RF-049 — Confirma a assinatura Gov.br de uma transferência em AWAITING_GOVBR_SIGNATURE.
/// Após confirmação, o TRANSFER_IN é gerado e o status passa para CONFIRMED.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfirmGovbrSignatureTransferPayload {
    pub notes: Option<String>,
}

// ============================
// Disposal Requests (Ticket 1.1 — RN-005)
// ============================

/// Status do pedido de desfazimento
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "disposal_request_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisposalRequestStatus {
    AwaitingSignature,
    Signed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct DisposalRequestDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub sei_process_number: String,
    pub justification: String,
    pub technical_opinion_url: String,
    pub status: DisposalRequestStatus,
    pub notes: Option<String>,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub signed_by: Option<Uuid>,
    pub signed_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct DisposalRequestItemDto {
    pub id: Uuid,
    pub disposal_request_id: Uuid,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub catalog_item_code: Option<String>,
    pub unit_raw_id: Uuid,
    pub unit_symbol: Option<String>,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub notes: Option<String>,
    pub movement_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisposalRequestWithItemsDto {
    #[serde(flatten)]
    pub request: DisposalRequestDto,
    pub warehouse_name: Option<String>,
    pub items: Vec<DisposalRequestItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisposalRequestItemInput {
    pub catalog_item_id: Uuid,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub batch_number: Option<String>,
    pub notes: Option<String>,
}

/// Cria um pedido de desfazimento em AWAITING_SIGNATURE (estoque não é deduzido ainda).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDisposalRequestPayload {
    pub justification: String,
    pub sei_process_number: String,
    pub technical_opinion_url: String,
    pub notes: Option<String>,
    pub items: Vec<DisposalRequestItemInput>,
}

/// Confirma que a assinatura Gov.br foi obtida — dispara a dedução de estoque.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfirmDisposalSignaturePayload {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CancelDisposalRequestPayload {
    pub cancellation_reason: String,
}

// ============================
// Inventory Sessions (Ticket 1.3 — RF-019 + Ticket 1.2 — RF-049)
// ============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "inventory_session_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InventorySessionStatus {
    Open,
    Counting,
    Reconciling,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InventorySessionDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub status: InventorySessionStatus,
    pub tolerance_percentage: Decimal,
    pub sei_process_number: Option<String>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub counting_started_at: Option<DateTime<Utc>>,
    pub reconciliation_started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub govbr_signed_at: Option<DateTime<Utc>>,
    pub govbr_signed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InventorySessionItemDto {
    pub id: Uuid,
    pub session_id: Uuid,
    pub catalog_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub catalog_item_code: Option<String>,
    pub unit_raw_id: Uuid,
    pub unit_symbol: Option<String>,
    pub system_quantity: Decimal,
    pub counted_quantity: Option<Decimal>,
    /// counted_quantity - system_quantity (calculado no SELECT)
    pub divergence: Option<Decimal>,
    /// |divergence| / system_quantity (calculado no SELECT; NULL se system_quantity = 0)
    pub divergence_percentage: Option<Decimal>,
    pub movement_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventorySessionWithItemsDto {
    #[serde(flatten)]
    pub session: InventorySessionDto,
    pub warehouse_name: Option<String>,
    pub items: Vec<InventorySessionItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInventorySessionPayload {
    /// Percentual de tolerância; se None, usa `inventory.tolerance_percentage` de system_settings.
    pub tolerance_percentage: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubmitCountPayload {
    pub catalog_item_id: Uuid,
    pub counted_quantity: Decimal,
    pub notes: Option<String>,
}

/// RN-012: sei_process_number é obrigatório se algum item tiver divergência > tolerance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReconcileInventoryPayload {
    pub sei_process_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CancelInventorySessionPayload {
    pub reason: Option<String>,
}

/// RF-049: Confirma assinatura Gov.br do documento de conciliação de inventário.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfirmGovbrSignatureInventoryPayload {
    pub notes: Option<String>,
}
