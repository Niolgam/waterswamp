use crate::errors::RepositoryError;
use crate::models::geo_regions::{
    CityDto, CityWithStateDto, CountryDto, StateDto, StateWithCountryDto,
};
use crate::value_objects::{LocationName, StateCode};
use async_trait::async_trait;
use uuid::Uuid;

// ============================
// Country Repository Port
// ============================

#[async_trait]
pub trait CountryRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CountryDto>, RepositoryError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<CountryDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_code_excluding(
        &self,
        code: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        code: &str,
    ) -> Result<CountryDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        code: Option<&str>,
    ) -> Result<CountryDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<CountryDto>, i64), RepositoryError>;
}

// ============================
// State Repository Port
// ============================

#[async_trait]
pub trait StateRepositoryPort: Send + Sync {
    // Read operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<StateDto>, RepositoryError>;
    async fn find_with_country_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<StateWithCountryDto>, RepositoryError>;
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
        country_id: Uuid,
    ) -> Result<StateDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        code: Option<&StateCode>,
        country_id: Option<Uuid>,
    ) -> Result<StateDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // List operations
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        country_id: Option<Uuid>,
    ) -> Result<(Vec<StateWithCountryDto>, i64), RepositoryError>;
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
