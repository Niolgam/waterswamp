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

// ============================
// CATMAT Group Repository Port
// ============================

#[async_trait]
pub trait CatmatGroupRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatGroupDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, code: &str, name: &str, is_active: bool) -> Result<CatmatGroupDto, RepositoryError>;
    async fn update(&self, id: Uuid, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatmatGroupDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatmatGroupDto>, i64), RepositoryError>;
    async fn get_tree(&self) -> Result<Vec<CatmatGroupTreeNode>, RepositoryError>;
}

// ============================
// CATMAT Class Repository Port
// ============================

#[async_trait]
pub trait CatmatClassRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatClassDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatmatClassWithDetailsDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, group_id: Uuid, code: &str, name: &str, budget_classification_id: Option<Uuid>, is_active: bool) -> Result<CatmatClassDto, RepositoryError>;
    async fn update(&self, id: Uuid, group_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, budget_classification_id: Option<Uuid>, is_active: Option<bool>) -> Result<CatmatClassDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>, group_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatmatClassWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// CATMAT Item Repository Port
// ============================

#[async_trait]
pub trait CatmatItemRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatItemDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatmatItemWithDetailsDto>, RepositoryError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<CatmatItemDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(
        &self,
        class_id: Uuid,
        unit_of_measure_id: Uuid,
        code: &str,
        description: &str,
        supplementary_description: Option<&str>,
        is_sustainable: bool,
        specification: Option<&str>,
        estimated_value: rust_decimal::Decimal,
        search_links: Option<&str>,
        photo_url: Option<&str>,
        is_permanent: bool,
        shelf_life_days: Option<i32>,
        requires_batch_control: bool,
        is_active: bool,
    ) -> Result<CatmatItemDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        class_id: Option<Uuid>,
        unit_of_measure_id: Option<Uuid>,
        code: Option<&str>,
        description: Option<&str>,
        supplementary_description: Option<&str>,
        is_sustainable: Option<bool>,
        specification: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>,
        search_links: Option<&str>,
        photo_url: Option<&str>,
        is_permanent: Option<bool>,
        shelf_life_days: Option<i32>,
        requires_batch_control: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<CatmatItemDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        class_id: Option<Uuid>,
        is_sustainable: Option<bool>,
        is_permanent: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatmatItemWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// CATSER Group Repository Port
// ============================

#[async_trait]
pub trait CatserGroupRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserGroupDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, code: &str, name: &str, is_active: bool) -> Result<CatserGroupDto, RepositoryError>;
    async fn update(&self, id: Uuid, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatserGroupDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatserGroupDto>, i64), RepositoryError>;
    async fn get_tree(&self) -> Result<Vec<CatserGroupTreeNode>, RepositoryError>;
}

// ============================
// CATSER Class Repository Port
// ============================

#[async_trait]
pub trait CatserClassRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserClassDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatserClassWithDetailsDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(&self, group_id: Uuid, code: &str, name: &str, budget_classification_id: Option<Uuid>, is_active: bool) -> Result<CatserClassDto, RepositoryError>;
    async fn update(&self, id: Uuid, group_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, budget_classification_id: Option<Uuid>, is_active: Option<bool>) -> Result<CatserClassDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(&self, limit: i64, offset: i64, search: Option<String>, group_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatserClassWithDetailsDto>, i64), RepositoryError>;
}

// ============================
// CATSER Item Repository Port
// ============================

#[async_trait]
pub trait CatserItemRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserItemDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatserItemWithDetailsDto>, RepositoryError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<CatserItemDto>, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError>;
    async fn create(
        &self,
        class_id: Uuid,
        unit_of_measure_id: Uuid,
        code: &str,
        description: &str,
        supplementary_description: Option<&str>,
        specification: Option<&str>,
        estimated_value: rust_decimal::Decimal,
        search_links: Option<&str>,
        is_active: bool,
    ) -> Result<CatserItemDto, RepositoryError>;
    async fn update(
        &self,
        id: Uuid,
        class_id: Option<Uuid>,
        unit_of_measure_id: Option<Uuid>,
        code: Option<&str>,
        description: Option<&str>,
        supplementary_description: Option<&str>,
        specification: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>,
        search_links: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<CatserItemDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        class_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatserItemWithDetailsDto>, i64), RepositoryError>;
}
