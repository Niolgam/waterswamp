use crate::errors::RepositoryError;
use crate::models::catalog::*;
use async_trait::async_trait;
use uuid::Uuid;

// ============================
// Unit of Measure Repository Port
// ============================

#[async_trait]
pub trait UnitOfMeasureRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOfMeasureDto>, RepositoryError>;
    async fn find_by_symbol(&self, symbol: &str) -> Result<Option<UnitOfMeasureDto>, RepositoryError>;
    async fn exists_by_symbol(&self, symbol: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_symbol_excluding(&self, symbol: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(
        &self,
        name: &str,
        symbol: &str,
        description: Option<&str>,
        is_base_unit: bool,
    ) -> Result<UnitOfMeasureDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        symbol: Option<&str>,
        description: Option<&str>,
        is_base_unit: Option<bool>,
    ) -> Result<UnitOfMeasureDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<UnitOfMeasureDto>, i64), RepositoryError>;
}

// ============================
// Catalog Group Repository Port
// ============================

#[async_trait]
pub trait CatalogGroupRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatalogGroupDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatalogGroupWithDetailsDto>, RepositoryError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<CatalogGroupDto>, RepositoryError>;
    async fn exists_by_code_in_level(&self, code: &str, parent_id: Option<Uuid>) -> Result<bool, RepositoryError>;
    async fn exists_by_code_in_level_excluding(
        &self,
        code: &str,
        parent_id: Option<Uuid>,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn has_children(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn get_item_count(&self, id: Uuid) -> Result<i64, RepositoryError>;
    async fn create(
        &self,
        parent_id: Option<Uuid>,
        name: &str,
        code: &str,
        item_type: ItemType,
        budget_classification_id: Uuid,
        is_active: bool,
    ) -> Result<CatalogGroupDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        parent_id: Option<Option<Uuid>>,
        name: Option<&str>,
        code: Option<&str>,
        item_type: Option<ItemType>,
        budget_classification_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<CatalogGroupDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        parent_id: Option<Uuid>,
        item_type: Option<ItemType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatalogGroupWithDetailsDto>, i64), RepositoryError>;
    async fn find_children(&self, parent_id: Option<Uuid>) -> Result<Vec<CatalogGroupDto>, RepositoryError>;
    async fn get_tree(&self) -> Result<Vec<CatalogGroupTreeNode>, RepositoryError>;
}

// ============================
// Catalog Item Repository Port
// ============================

#[async_trait]
pub trait CatalogItemRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatalogItemDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatalogItemWithDetailsDto>, RepositoryError>;
    async fn find_by_catmat_code(&self, catmat_code: &str) -> Result<Option<CatalogItemDto>, RepositoryError>;
    async fn exists_by_catmat_code(&self, catmat_code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_catmat_code_excluding(
        &self,
        catmat_code: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn exists_by_name_in_group(&self, name: &str, group_id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_name_in_group_excluding(
        &self,
        name: &str,
        group_id: Uuid,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn create(
        &self,
        group_id: Uuid,
        unit_of_measure_id: Uuid,
        name: &str,
        catmat_code: Option<&str>,
        specification: &str,
        estimated_value: rust_decimal::Decimal,
        search_links: Option<&str>,
        photo_url: Option<&str>,
        is_stockable: bool,
        is_permanent: bool,
        shelf_life_days: Option<i32>,
        requires_batch_control: bool,
        is_active: bool,
    ) -> Result<CatalogItemDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        group_id: Option<Uuid>,
        unit_of_measure_id: Option<Uuid>,
        name: Option<&str>,
        catmat_code: Option<&str>,
        specification: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>,
        search_links: Option<&str>,
        photo_url: Option<&str>,
        is_stockable: Option<bool>,
        is_permanent: Option<bool>,
        shelf_life_days: Option<i32>,
        requires_batch_control: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<CatalogItemDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        group_id: Option<Uuid>,
        is_stockable: Option<bool>,
        is_permanent: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatalogItemWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// Unit Conversion Repository Port
// ============================

#[async_trait]
pub trait UnitConversionRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitConversionDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<UnitConversionWithDetailsDto>, RepositoryError>;
    async fn find_conversion(
        &self,
        from_unit_id: Uuid,
        to_unit_id: Uuid,
    ) -> Result<Option<UnitConversionDto>, RepositoryError>;
    async fn exists_conversion(
        &self,
        from_unit_id: Uuid,
        to_unit_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn create(
        &self,
        from_unit_id: Uuid,
        to_unit_id: Uuid,
        conversion_factor: rust_decimal::Decimal,
    ) -> Result<UnitConversionDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        conversion_factor: rust_decimal::Decimal,
    ) -> Result<UnitConversionDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        from_unit_id: Option<Uuid>,
        to_unit_id: Option<Uuid>,
    ) -> Result<(Vec<UnitConversionWithDetailsDto>, i64), RepositoryError>;
}
