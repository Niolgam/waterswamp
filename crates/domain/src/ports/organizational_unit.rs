use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::RepositoryError;
use crate::models::{CreateOrganizationalUnitDto, OrganizationalUnit, UpdateOrganizationalUnitDto};

#[async_trait]
pub trait OrganizationalUnitRepositoryPort: Send + Sync {
    async fn create(&self, dto: &CreateOrganizationalUnitDto) -> Result<OrganizationalUnit, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnit>, RepositoryError>;
    async fn find_by_acronym(&self, acronym: &str) -> Result<Option<OrganizationalUnit>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<OrganizationalUnit>, RepositoryError>;
    async fn list_by_parent(&self, parent_id: Option<Uuid>) -> Result<Vec<OrganizationalUnit>, RepositoryError>;
    async fn list_by_category(&self, category_id: Uuid) -> Result<Vec<OrganizationalUnit>, RepositoryError>;
    async fn list_by_campus(&self, campus_id: Uuid) -> Result<Vec<OrganizationalUnit>, RepositoryError>;
    async fn list_root_units(&self) -> Result<Vec<OrganizationalUnit>, RepositoryError>;
    async fn update(&self, id: Uuid, dto: &UpdateOrganizationalUnitDto) -> Result<OrganizationalUnit, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn count(&self) -> Result<i64, RepositoryError>;
}
