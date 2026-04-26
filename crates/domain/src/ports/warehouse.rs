use crate::errors::RepositoryError;
use crate::models::warehouse::*;
use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait WarehouseRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseDto>, RepositoryError>;
    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseWithDetailsDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(
        &self,
        code: &str,
        id: Uuid,
    ) -> Result<bool, RepositoryError>;

    async fn create(
        &self,
        name: &str,
        code: &str,
        warehouse_type: WarehouseType,
        city_id: Uuid,
        responsible_user_id: Option<Uuid>,
        responsible_unit_id: Option<Uuid>,
        allows_transfers: bool,
        is_budgetary: bool,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> Result<WarehouseDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        code: Option<&str>,
        warehouse_type: Option<WarehouseType>,
        city_id: Option<Uuid>,
        responsible_user_id: Option<Uuid>,
        responsible_unit_id: Option<Uuid>,
        allows_transfers: Option<bool>,
        is_budgetary: Option<bool>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<WarehouseDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        warehouse_type: Option<WarehouseType>,
        city_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<WarehouseWithDetailsDto>, i64), RepositoryError>;
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait WarehouseStockRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseStockDto>, RepositoryError>;
    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseStockWithDetailsDto>, RepositoryError>;
    async fn find_by_warehouse_and_item(
        &self,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
    ) -> Result<Option<WarehouseStockDto>, RepositoryError>;

    async fn list_by_warehouse(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        search: Option<String>,
        is_blocked: Option<bool>,
    ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), RepositoryError>;

    async fn update_params(
        &self,
        id: Uuid,
        min_stock: Option<Decimal>,
        max_stock: Option<Decimal>,
        reorder_point: Option<Decimal>,
        resupply_days: Option<i32>,
        location: Option<&str>,
        secondary_location: Option<&str>,
    ) -> Result<WarehouseStockDto, RepositoryError>;

    async fn block(
        &self,
        id: Uuid,
        block_reason: &str,
        blocked_by: Uuid,
    ) -> Result<WarehouseStockDto, RepositoryError>;

    async fn unblock(&self, id: Uuid) -> Result<WarehouseStockDto, RepositoryError>;
}

// ── Disposal Requests (Ticket 1.1 — RN-005) ──────────────────────────────────

#[async_trait]
pub trait DisposalRequestRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DisposalRequestDto>, RepositoryError>;
    async fn find_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<DisposalRequestWithItemsDto>, RepositoryError>;
    async fn list_by_warehouse(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<DisposalRequestStatus>,
    ) -> Result<(Vec<DisposalRequestDto>, i64), RepositoryError>;
    async fn create(
        &self,
        warehouse_id: Uuid,
        sei_process_number: &str,
        justification: &str,
        technical_opinion_url: &str,
        notes: Option<&str>,
        requested_by: Uuid,
    ) -> Result<DisposalRequestDto, RepositoryError>;
    async fn create_item(
        &self,
        disposal_request_id: Uuid,
        catalog_item_id: Uuid,
        unit_raw_id: Uuid,
        unit_conversion_id: Option<Uuid>,
        quantity_raw: rust_decimal::Decimal,
        conversion_factor: rust_decimal::Decimal,
        batch_number: Option<&str>,
        notes: Option<&str>,
    ) -> Result<(), RepositoryError>;
    async fn transition_to_signed(
        &self,
        id: Uuid,
        signed_by: Uuid,
    ) -> Result<DisposalRequestDto, RepositoryError>;
    async fn transition_to_cancelled(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
        cancellation_reason: &str,
    ) -> Result<DisposalRequestDto, RepositoryError>;
    async fn set_item_movement(
        &self,
        item_id: Uuid,
        movement_id: Uuid,
    ) -> Result<(), RepositoryError>;
}

// ── Inventory Sessions (Ticket 1.3 — RF-019) ─────────────────────────────────

#[async_trait]
pub trait InventorySessionRepositoryPort: Send + Sync {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<InventorySessionDto>, RepositoryError>;
    async fn find_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<InventorySessionWithItemsDto>, RepositoryError>;
    async fn list_by_warehouse(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<InventorySessionStatus>,
    ) -> Result<(Vec<InventorySessionDto>, i64), RepositoryError>;
    async fn create(
        &self,
        warehouse_id: Uuid,
        tolerance_percentage: rust_decimal::Decimal,
        notes: Option<&str>,
        created_by: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError>;
    async fn transition_to_counting(&self, id: Uuid) -> Result<InventorySessionDto, RepositoryError>;
    async fn transition_to_reconciling(
        &self,
        id: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError>;
    async fn transition_to_completed(
        &self,
        id: Uuid,
        sei_process_number: Option<&str>,
    ) -> Result<InventorySessionDto, RepositoryError>;
    async fn transition_to_cancelled(
        &self,
        id: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError>;
    async fn confirm_govbr_signature(
        &self,
        id: Uuid,
        signed_by: Uuid,
    ) -> Result<InventorySessionDto, RepositoryError>;
    /// Snapshot: insere todos os itens de warehouse_stocks para o warehouse na sessão.
    async fn snapshot_stock_items(
        &self,
        session_id: Uuid,
        warehouse_id: Uuid,
    ) -> Result<usize, RepositoryError>;
    /// Atualiza ou cria a contagem física de um item.
    async fn upsert_item_count(
        &self,
        session_id: Uuid,
        catalog_item_id: Uuid,
        counted_quantity: rust_decimal::Decimal,
        notes: Option<&str>,
    ) -> Result<InventorySessionItemDto, RepositoryError>;
    async fn list_items(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<InventorySessionItemDto>, RepositoryError>;
    async fn set_item_movement(
        &self,
        item_id: Uuid,
        movement_id: Uuid,
    ) -> Result<(), RepositoryError>;
}
