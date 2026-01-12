use crate::models::organizational::*;
use crate::ports::RepositoryError;
use async_trait::async_trait;
use uuid::Uuid;

// ============================================================================
// System Settings Repository Port
// ============================================================================

#[async_trait]
pub trait SystemSettingsRepositoryPort: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<SystemSettingDto>, RepositoryError>;

    async fn list(
        &self,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SystemSettingDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateSystemSettingPayload) -> Result<SystemSettingDto, RepositoryError>;

    async fn update(
        &self,
        key: &str,
        payload: UpdateSystemSettingPayload,
        updated_by: Option<Uuid>,
    ) -> Result<SystemSettingDto, RepositoryError>;

    async fn delete(&self, key: &str) -> Result<(), RepositoryError>;

    async fn get_value<T>(&self, key: &str) -> Result<Option<T>, RepositoryError>
    where
        T: serde::de::DeserializeOwned;
}

// ============================================================================
// Organization Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn find_by_cnpj(&self, cnpj: &str) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn find_main(&self) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn list(
        &self,
        is_active: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateOrganizationPayload) -> Result<OrganizationDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationPayload) -> Result<OrganizationDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organizational Unit Category Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationalUnitCategoryRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError>;

    async fn find_by_name(&self, name: &str) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError>;

    async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitCategoryDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateOrganizationalUnitCategoryPayload) -> Result<OrganizationalUnitCategoryDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationalUnitCategoryPayload) -> Result<OrganizationalUnitCategoryDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organizational Unit Type Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationalUnitTypeRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError>;

    async fn find_by_code(&self, code: &str) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError>;

    async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitTypeDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateOrganizationalUnitTypePayload) -> Result<OrganizationalUnitTypeDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationalUnitTypePayload) -> Result<OrganizationalUnitTypeDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organizational Unit Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationalUnitRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnitDto>, RepositoryError>;

    async fn find_by_id_with_details(&self, id: Uuid) -> Result<Option<OrganizationalUnitWithDetailsDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationalUnitDto>, RepositoryError>;

    async fn list(
        &self,
        organization_id: Option<Uuid>,
        parent_id: Option<Uuid>,
        category_id: Option<Uuid>,
        unit_type_id: Option<Uuid>,
        activity_area: Option<ActivityArea>,
        internal_type: Option<InternalUnitType>,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitWithDetailsDto>, i64), RepositoryError>;

    async fn get_tree(&self, organization_id: Option<Uuid>) -> Result<Vec<OrganizationalUnitTreeNode>, RepositoryError>;

    async fn get_children(&self, parent_id: Uuid) -> Result<Vec<OrganizationalUnitDto>, RepositoryError>;

    async fn has_children(&self, id: Uuid) -> Result<bool, RepositoryError>;

    async fn get_path_to_root(&self, id: Uuid) -> Result<Vec<OrganizationalUnitDto>, RepositoryError>;

    async fn create(&self, payload: CreateOrganizationalUnitPayload) -> Result<OrganizationalUnitDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationalUnitPayload) -> Result<OrganizationalUnitDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    async fn deactivate(&self, id: Uuid, reason: Option<String>) -> Result<(), RepositoryError>;

    async fn activate(&self, id: Uuid) -> Result<(), RepositoryError>;
}
