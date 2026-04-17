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
