use crate::errors::RepositoryError;
use crate::models::{
    BuildingDto, BuildingTypeDto, BuildingWithRelationsDto, CityDto, CityWithStateDto,
    DepartmentCategoryDto, SiteDto, SiteTypeDto, SiteWithRelationsDto, SpaceTypeDto, StateDto,
};
use crate::value_objects::{LocationName, StateCode};
use async_trait::async_trait;
use uuid::Uuid;

// ============================
// State Repository Port
// ============================

#[async_trait]
pub trait StateRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StateDto>, RepositoryError>;
    async fn find_by_code(&self, code: &StateCode) -> Result<Option<StateDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_code(&self, code: &StateCode) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(
        &self,
        code: &StateCode,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        code: &StateCode,
    ) -> Result<StateDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        code: Option<&StateCode>,
    ) -> Result<StateDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<StateDto>, i64), RepositoryError>;
}

// ============================
// City Repository Port
// ============================

#[async_trait]
pub trait CityRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CityDto>, RepositoryError>;
    async fn find_with_state_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<CityWithStateDto>, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        state_id: Uuid,
    ) -> Result<CityDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        state_id: Option<Uuid>,
    ) -> Result<CityDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        state_id: Option<Uuid>,
    ) -> Result<(Vec<CityWithStateDto>, i64), RepositoryError>;
}

// ============================
// Site Type Repository Port
// ============================

#[async_trait]
pub trait SiteTypeRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiteTypeDto>, RepositoryError>;
    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<SiteTypeDto>, RepositoryError>;

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
    ) -> Result<SiteTypeDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        description: Option<&str>,
    ) -> Result<SiteTypeDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<SiteTypeDto>, i64), RepositoryError>;
}

// ============================
// Building Type Repository Port
// ============================

#[async_trait]
pub trait BuildingTypeRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BuildingTypeDto>, RepositoryError>;
    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<BuildingTypeDto>, RepositoryError>;

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
    ) -> Result<BuildingTypeDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        description: Option<&str>,
    ) -> Result<BuildingTypeDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<BuildingTypeDto>, i64), RepositoryError>;
}

// ============================
// Space Type Repository Port
// ============================

#[async_trait]
pub trait SpaceTypeRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpaceTypeDto>, RepositoryError>;
    async fn find_by_name(
        &self,
        name: &LocationName,
    ) -> Result<Option<SpaceTypeDto>, RepositoryError>;

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
    ) -> Result<SpaceTypeDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        description: Option<&str>,
    ) -> Result<SpaceTypeDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<SpaceTypeDto>, i64), RepositoryError>;
}

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
// Site Repository Port (Phase 3A)
// ============================

#[async_trait]
pub trait SiteRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiteDto>, RepositoryError>;
    async fn find_with_relations_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SiteWithRelationsDto>, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        city_id: Uuid,
        site_type_id: Uuid,
        address: Option<&str>,
    ) -> Result<SiteDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        city_id: Option<Uuid>,
        site_type_id: Option<Uuid>,
        address: Option<&str>,
    ) -> Result<SiteDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        city_id: Option<Uuid>,
        site_type_id: Option<Uuid>,
    ) -> Result<(Vec<SiteWithRelationsDto>, i64), RepositoryError>;
}

// ============================
// Building Repository Port (Phase 3B)
// ============================

#[async_trait]
pub trait BuildingRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BuildingDto>, RepositoryError>;
    async fn find_with_relations_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<BuildingWithRelationsDto>, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        site_id: Uuid,
        building_type_id: Uuid,
        description: Option<&str>,
    ) -> Result<BuildingDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        site_id: Option<Uuid>,
        building_type_id: Option<Uuid>,
        description: Option<&str>,
    ) -> Result<BuildingDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        site_id: Option<Uuid>,
        building_type_id: Option<Uuid>,
    ) -> Result<(Vec<BuildingWithRelationsDto>, i64), RepositoryError>;
}
