use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::RepositoryError;
use crate::models::{City, CreateCityDto, UpdateCityDto};

#[async_trait]
pub trait CityRepositoryPort: Send + Sync {
    async fn create(&self, dto: &CreateCityDto) -> Result<City, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<City>, RepositoryError>;
    async fn list_by_state(&self, state_id: Uuid) -> Result<Vec<City>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<City>, RepositoryError>;
    async fn update(&self, id: Uuid, dto: &UpdateCityDto) -> Result<City, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
}
