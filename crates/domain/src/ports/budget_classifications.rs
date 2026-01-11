use crate::errors::RepositoryError;
use crate::models::{BudgetClassificationDto, BudgetClassificationWithParentDto};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait BudgetClassificationRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BudgetClassificationDto>, RepositoryError>;

    async fn find_with_parent_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<BudgetClassificationWithParentDto>, RepositoryError>;

    async fn find_by_full_code(
        &self,
        full_code: &str,
    ) -> Result<Option<BudgetClassificationDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_full_code(&self, full_code: &str) -> Result<bool, RepositoryError>;

    async fn exists_by_full_code_excluding(
        &self,
        full_code: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        parent_id: Option<Uuid>,
        code_part: &str,
        name: &str,
        is_active: bool,
    ) -> Result<BudgetClassificationDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        parent_id: Option<Option<Uuid>>,
        code_part: Option<&str>,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<BudgetClassificationDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        parent_id: Option<Uuid>,
        level: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<(Vec<BudgetClassificationWithParentDto>, i64), RepositoryError>;

    // Hierarchical operations
    async fn find_children(&self, parent_id: Option<Uuid>) -> Result<Vec<BudgetClassificationDto>, RepositoryError>;

    async fn find_by_level(&self, level: i32) -> Result<Vec<BudgetClassificationDto>, RepositoryError>;
}
