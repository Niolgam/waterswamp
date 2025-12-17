use crate::errors::RepositoryError;
use crate::models::departments::DepartmentCategoryDto;
use crate::value_objects::LocationName;
use async_trait::async_trait;
use uuid::Uuid;

// ============================
// Department Category Repository Port
// ============================

#[async_trait]
pub trait DepartmentCategoryRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DepartmentCategoryDto>, RepositoryError>;
    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<DepartmentCategoryDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_name(&self, name: &LocationName) -> Result<bool, RepositoryError>;
    async fn exists_by_name_excluding(
        &self,
        name: &LocationName,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        description: Option<&str>,
    ) -> Result<DepartmentCategoryDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        description: Option<&str>,
    ) -> Result<DepartmentCategoryDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<DepartmentCategoryDto>, i64), RepositoryError>;
}

// ============================
// Department Repository Port
// ============================
// TODO: Department repository port will be added when Phase 4 is implemented
