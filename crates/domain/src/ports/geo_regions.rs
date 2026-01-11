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
    async fn find_by_iso2(&self, iso2: &str) -> Result<Option<CountryDto>, RepositoryError>;
    async fn find_by_bacen_code(&self, bacen_code: i32) -> Result<Option<CountryDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_iso2(&self, iso2: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_iso2_excluding(
        &self,
        iso2: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn exists_by_bacen_code(&self, bacen_code: i32) -> Result<bool, RepositoryError>;
    async fn exists_by_bacen_code_excluding(
        &self,
        bacen_code: i32,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        iso2: &str,
        bacen_code: i32,
    ) -> Result<CountryDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        iso2: Option<&str>,
        bacen_code: Option<i32>,
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
    async fn find_by_abbreviation(&self, abbreviation: &StateCode) -> Result<Option<StateDto>, RepositoryError>;
    async fn find_by_ibge_code(&self, ibge_code: i32) -> Result<Option<StateDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_abbreviation(&self, abbreviation: &StateCode) -> Result<bool, RepositoryError>;
    async fn exists_by_abbreviation_in_country(
        &self,
        abbreviation: &StateCode,
        country_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn exists_by_abbreviation_excluding(
        &self,
        abbreviation: &StateCode,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn exists_by_ibge_code(&self, ibge_code: i32) -> Result<bool, RepositoryError>;
    async fn exists_by_ibge_code_excluding(
        &self,
        ibge_code: i32,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        abbreviation: &StateCode,
        ibge_code: i32,
        country_id: Uuid,
    ) -> Result<StateDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        abbreviation: Option<&StateCode>,
        ibge_code: Option<i32>,
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
    async fn find_by_ibge_code(&self, ibge_code: i32) -> Result<Option<CityDto>, RepositoryError>;

    // Validation checks
    async fn exists_by_ibge_code(&self, ibge_code: i32) -> Result<bool, RepositoryError>;
    async fn exists_by_ibge_code_excluding(
        &self,
        ibge_code: i32,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Write operations
    async fn create(
        &self,
        name: &LocationName,
        ibge_code: i32,
        state_id: Uuid,
    ) -> Result<CityDto, RepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        name: Option<&LocationName>,
        ibge_code: Option<i32>,
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
