use crate::errors::RepositoryError;
use crate::models::{MaterialDto, MaterialGroupDto, MaterialWithGroupDto};
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
