use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::RepositoryError;
use crate::models::{CreateUnitCategoryDto, UnitCategory, UpdateUnitCategoryDto};

#[async_trait]
pub trait UnitCategoryRepositoryPort: Send + Sync {
    async fn create(&self, dto: &CreateUnitCategoryDto) -> Result<UnitCategory, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitCategory>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<UnitCategory>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<UnitCategory>, RepositoryError>;
    async fn update(&self, id: Uuid, dto: &UpdateUnitCategoryDto) -> Result<UnitCategory, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
}
