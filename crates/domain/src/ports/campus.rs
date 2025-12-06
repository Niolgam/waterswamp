use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::RepositoryError;
use crate::models::campus::{Campus, CreateCampusDto, UpdateCampusDto};

/// Port (trait) for Campus repository
/// Defines persistence operations for Campus
#[async_trait]
pub trait CampusRepositoryPort: Send + Sync {
    /// Creates a new campus
    async fn create(&self, dto: &CreateCampusDto) -> Result<Campus, RepositoryError>;

    /// Finds campus by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Campus>, RepositoryError>;

    /// Finds campus by acronym
    async fn find_by_acronym(&self, acronym: &str) -> Result<Option<Campus>, RepositoryError>;

    /// Finds campus by name
    async fn find_by_name(&self, name: &str) -> Result<Option<Campus>, RepositoryError>;

    /// Lists all campuses
    async fn list_all(&self) -> Result<Vec<Campus>, RepositoryError>;

    /// Lists campuses with pagination
    async fn list_paginated(&self, limit: i64, offset: i64) -> Result<Vec<Campus>, RepositoryError>;

    /// Finds campuses by city ID
    async fn find_by_city(&self, city_id: Uuid) -> Result<Vec<Campus>, RepositoryError>;

    /// Updates existing campus
    async fn update(&self, id: Uuid, dto: &UpdateCampusDto) -> Result<Campus, RepositoryError>;

    /// Deletes campus by ID
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    /// Checks if campus with the specified acronym exists
    async fn exists_by_acronym(&self, acronym: &str) -> Result<bool, RepositoryError>;

    /// Checks if campus with the specified name exists
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;

    /// Counts total campuses
    async fn count(&self) -> Result<i64, RepositoryError>;
}
