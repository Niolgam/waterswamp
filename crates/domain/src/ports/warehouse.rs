use crate::errors::RepositoryError;
use crate::models::{
    MaterialConsumptionReportDto, MaterialDto, MaterialGroupDto, MaterialWithGroupDto,
    MostRequestedMaterialsReportDto, MovementAnalysisReportDto, MovementType, RequisitionDto,
    RequisitionItemDto, RequisitionStatus, RequisitionWithDetailsDto, StockMovementDto,
    StockMovementWithDetailsDto, StockValueDetailDto, StockValueReportDto, WarehouseDto,
    WarehouseStockDto, WarehouseStockWithDetailsDto, WarehouseWithCityDto,
};
use crate::value_objects::{CatmatCode, MaterialCode, UnitOfMeasure};
use async_trait::async_trait;
use uuid::Uuid;

// ============================
// Material Group Repository Port
// ============================

#[async_trait]
pub trait MaterialGroupRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaterialGroupDto>, RepositoryError>;
    async fn find_by_code(
        &self,
        code: &MaterialCode,
    ) -> Result<Option<MaterialGroupDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_code(&self, code: &MaterialCode) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(
        &self,
        code: &MaterialCode,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        code: &MaterialCode,
        name: &str,
        description: Option<&str>,
        expense_element: Option<&str>,
        is_personnel_exclusive: bool,
    ) -> Result<MaterialGroupDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        code: Option<&MaterialCode>,
        name: Option<&str>,
        description: Option<&str>,
        expense_element: Option<&str>,
        is_personnel_exclusive: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<MaterialGroupDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        is_personnel_exclusive: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(Vec<MaterialGroupDto>, i64), RepositoryError>;
}

// ============================
// Material Repository Port
// ============================

#[async_trait]
pub trait MaterialRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaterialDto>, RepositoryError>;
    async fn find_with_group_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<MaterialWithGroupDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_name_in_group(
        &self,
        name: &str,
        material_group_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn exists_by_name_in_group_excluding(
        &self,
        name: &str,
        material_group_id: Uuid,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        material_group_id: Uuid,
        name: &str,
        estimated_value: rust_decimal::Decimal,
        unit_of_measure: &UnitOfMeasure,
        specification: &str,
        search_links: Option<&str>,
        catmat_code: Option<&CatmatCode>,
        photo_url: Option<&str>,
    ) -> Result<MaterialDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        material_group_id: Option<Uuid>,
        name: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>,
        unit_of_measure: Option<&UnitOfMeasure>,
        specification: Option<&str>,
        search_links: Option<&str>,
        catmat_code: Option<&CatmatCode>,
        photo_url: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<MaterialDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        material_group_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<MaterialWithGroupDto>, i64), RepositoryError>;
}

// ============================
// Warehouse Repository Port
// ============================

#[async_trait]
pub trait WarehouseRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseDto>, RepositoryError>;
    async fn find_with_city_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseWithCityDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(
        &self,
        code: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    async fn create(
        &self,
        name: &str,
        code: &str,
        city_id: Uuid,
        responsible_user_id: Option<Uuid>,
        address: Option<&str>,
        phone: Option<&str>,
        email: Option<&str>,
    ) -> Result<WarehouseDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        code: Option<&str>,
        city_id: Option<Uuid>,
        responsible_user_id: Option<Uuid>,
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
        city_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<WarehouseWithCityDto>, i64), RepositoryError>;
}

// ============================
// Warehouse Stock Repository Port
// ============================

#[async_trait]
pub trait WarehouseStockRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseStockDto>, RepositoryError>;
    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WarehouseStockWithDetailsDto>, RepositoryError>;
    async fn find_by_warehouse_and_material(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
    ) -> Result<Option<WarehouseStockDto>, RepositoryError>;

    async fn create(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
        quantity: rust_decimal::Decimal,
        average_unit_value: rust_decimal::Decimal,
        min_stock: Option<rust_decimal::Decimal>,
        max_stock: Option<rust_decimal::Decimal>,
        location: Option<&str>,
    ) -> Result<WarehouseStockDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        min_stock: Option<rust_decimal::Decimal>,
        max_stock: Option<rust_decimal::Decimal>,
        location: Option<&str>,
    ) -> Result<WarehouseStockDto, RepositoryError>;

    // Método crítico para atualizar quantidade e média ponderada
    async fn update_stock_and_average(
        &self,
        id: Uuid,
        new_quantity: rust_decimal::Decimal,
        new_average: rust_decimal::Decimal,
    ) -> Result<WarehouseStockDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        warehouse_id: Option<Uuid>,
        material_id: Option<Uuid>,
        search: Option<String>,
        low_stock: Option<bool>,
    ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// Stock Movement Repository Port
// ============================

#[async_trait]
pub trait StockMovementRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StockMovementDto>, RepositoryError>;
    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<StockMovementWithDetailsDto>, RepositoryError>;

    async fn create(
        &self,
        warehouse_stock_id: Uuid,
        movement_type: MovementType,
        quantity: rust_decimal::Decimal,
        unit_value: rust_decimal::Decimal,
        total_value: rust_decimal::Decimal,
        balance_before: rust_decimal::Decimal,
        balance_after: rust_decimal::Decimal,
        average_before: rust_decimal::Decimal,
        average_after: rust_decimal::Decimal,
        movement_date: chrono::DateTime<chrono::Utc>,
        document_number: Option<&str>,
        requisition_id: Option<Uuid>,
        user_id: Uuid,
        notes: Option<&str>,
    ) -> Result<StockMovementDto, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        warehouse_id: Option<Uuid>,
        material_id: Option<Uuid>,
        movement_type: Option<MovementType>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<StockMovementWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// Requisition Repository Port
// ============================

#[async_trait]
pub trait RequisitionRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RequisitionDto>, RepositoryError>;
    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<RequisitionWithDetailsDto>, RepositoryError>;

    async fn create(
        &self,
        warehouse_id: Uuid,
        requester_id: Uuid,
        total_value: rust_decimal::Decimal,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError>;

    async fn update_status(
        &self,
        id: Uuid,
        status: RequisitionStatus,
    ) -> Result<RequisitionDto, RepositoryError>;

    async fn approve(
        &self,
        id: Uuid,
        approved_by: Uuid,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError>;

    async fn reject(
        &self,
        id: Uuid,
        rejection_reason: &str,
    ) -> Result<RequisitionDto, RepositoryError>;

    async fn fulfill(
        &self,
        id: Uuid,
        fulfilled_by: Uuid,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        warehouse_id: Option<Uuid>,
        requester_id: Option<Uuid>,
        status: Option<RequisitionStatus>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<RequisitionWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// Requisition Item Repository Port
// ============================

#[async_trait]
pub trait RequisitionItemRepositoryPort: Send + Sync {
    async fn create(
        &self,
        requisition_id: Uuid,
        material_id: Uuid,
        requested_quantity: rust_decimal::Decimal,
        unit_value: rust_decimal::Decimal,
        total_value: rust_decimal::Decimal,
    ) -> Result<RequisitionItemDto, RepositoryError>;

    async fn update_fulfilled_quantity(
        &self,
        id: Uuid,
        fulfilled_quantity: rust_decimal::Decimal,
    ) -> Result<RequisitionItemDto, RepositoryError>;

    async fn find_by_requisition_id(
        &self,
        requisition_id: Uuid,
    ) -> Result<Vec<RequisitionItemDto>, RepositoryError>;
}

// ============================
// Warehouse Reports Port
// ============================

#[async_trait]
pub trait WarehouseReportsPort: Send + Sync {
    /// Relatório de valor total de estoque por almoxarifado
    async fn get_stock_value_report(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<StockValueReportDto>, RepositoryError>;

    /// Relatório detalhado de valor de estoque por material em um almoxarifado
    async fn get_stock_value_detail(
        &self,
        warehouse_id: Uuid,
        material_group_id: Option<Uuid>,
    ) -> Result<Vec<StockValueDetailDto>, RepositoryError>;

    /// Relatório de consumo de materiais por período
    async fn get_material_consumption_report(
        &self,
        warehouse_id: Option<Uuid>,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<MaterialConsumptionReportDto>, RepositoryError>;

    /// Relatório de materiais mais requisitados
    async fn get_most_requested_materials(
        &self,
        warehouse_id: Option<Uuid>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
    ) -> Result<Vec<MostRequestedMaterialsReportDto>, RepositoryError>;

    /// Análise de movimentações por tipo e período
    async fn get_movement_analysis(
        &self,
        warehouse_id: Option<Uuid>,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<MovementAnalysisReportDto>, RepositoryError>;
}
