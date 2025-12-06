use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::RepositoryError;
use crate::models::{CreateStateDto, State, UpdateStateDto};

#[async_trait]
pub trait StateRepositoryPort: Send + Sync {
    async fn create(&self, dto: &CreateStateDto) -> Result<State, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<State>, RepositoryError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<State>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<State>, RepositoryError>;
    async fn update(&self, id: Uuid, dto: &UpdateStateDto) -> Result<State, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
}
