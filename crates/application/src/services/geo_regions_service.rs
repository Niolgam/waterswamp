use crate::errors::ServiceError;
use domain::models::{
    CityDto, CityWithStateDto, CountryDto, CreateCityPayload, CreateCountryPayload,
    CreateStatePayload, StateDto, StateWithCountryDto, UpdateCityPayload, UpdateCountryPayload,
    UpdateStatePayload,
};
use domain::pagination::Paginated;
use domain::ports::{CityRepositoryPort, CountryRepositoryPort, StateRepositoryPort};
use std::sync::Arc;
use uuid::Uuid;

pub struct GeoRegionsService {
    country_repo: Arc<dyn CountryRepositoryPort>,
    state_repo: Arc<dyn StateRepositoryPort>,
    city_repo: Arc<dyn CityRepositoryPort>,
}

impl GeoRegionsService {
    pub fn new(
        country_repo: Arc<dyn CountryRepositoryPort>,
        state_repo: Arc<dyn StateRepositoryPort>,
        city_repo: Arc<dyn CityRepositoryPort>,
    ) -> Self {
        Self {
            country_repo,
            state_repo,
            city_repo,
        }
    }

    // ============================
    // Country Operations
    // ============================

    pub async fn create_country(
        &self,
        payload: CreateCountryPayload,
    ) -> Result<CountryDto, ServiceError> {
        // Check if country iso2 already exists
        if self.country_repo.exists_by_iso2(&payload.iso2).await? {
            return Err(ServiceError::Conflict(format!(
                "País com código ISO2 '{}' já existe",
                payload.iso2
            )));
        }

        // Check if bacen_code already exists
        if self
            .country_repo
            .exists_by_bacen_code(payload.bacen_code)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "País com código Bacen '{}' já existe",
                payload.bacen_code
            )));
        }

        let country = self
            .country_repo
            .create(
                &payload.name,
                &payload.iso2,
                payload.bacen_code,
                payload.is_active,
            )
            .await?;

        Ok(country)
    }

    pub async fn get_country(&self, id: Uuid) -> Result<CountryDto, ServiceError> {
        self.country_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("País não encontrado".to_string()))
    }

    pub async fn update_country(
        &self,
        id: Uuid,
        payload: UpdateCountryPayload,
    ) -> Result<CountryDto, ServiceError> {
        // Check if country exists
        let _ = self.get_country(id).await?;

        // If updating iso2, check for duplicates
        if let Some(ref new_iso2) = payload.iso2 {
            if self
                .country_repo
                .exists_by_iso2_excluding(new_iso2, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "País com código ISO2 '{}' já existe",
                    new_iso2
                )));
            }
        }

        // If updating bacen_code, check for duplicates
        if let Some(new_bacen_code) = payload.bacen_code {
            if self
                .country_repo
                .exists_by_bacen_code_excluding(new_bacen_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "País com código Bacen '{}' já existe",
                    new_bacen_code
                )));
            }
        }

        let country = self
            .country_repo
            .update(
                id,
                payload.name.as_ref(),
                payload.iso2.as_deref(),
                payload.bacen_code,
                payload.is_active,
            )
            .await?;

        Ok(country)
    }

    pub async fn delete_country(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.country_repo.delete(id).await?;

        if !deleted {
            return Err(ServiceError::NotFound("País não encontrado".to_string()));
        }

        Ok(())
    }

    pub async fn list_countries(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<Paginated<CountryDto>, ServiceError> {
        let (items, total) = self.country_repo.list(limit, offset, search).await?;

        Ok(Paginated::new(items, total, limit, offset))
    }

    // ============================
    // State Operations
    // ============================

    pub async fn create_state(
        &self,
        payload: CreateStatePayload,
    ) -> Result<StateWithCountryDto, ServiceError> {
        // Check if state abbreviation already exists in this country
        if self
            .state_repo
            .exists_by_abbreviation_in_country(&payload.abbreviation, payload.country_id)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Estado com sigla '{}' já existe neste país",
                payload.abbreviation
            )));
        }

        // Check if ibge_code already exists
        if self
            .state_repo
            .exists_by_ibge_code(payload.ibge_code)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Estado com código IBGE '{}' já existe",
                payload.ibge_code
            )));
        }

        // Validate that country exists
        if self
            .country_repo
            .find_by_id(payload.country_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound("País não encontrado".to_string()));
        }

        let state = self
            .state_repo
            .create(
                &payload.name,
                &payload.abbreviation,
                payload.ibge_code,
                payload.country_id,
                payload.is_active.unwrap_or(true),
            )
            .await?;

        // Return state with country information
        let state_with_country = self
            .state_repo
            .find_with_country_by_id(state.id)
            .await?
            .ok_or(ServiceError::NotFound("Estado não encontrado".to_string()))?;

        Ok(state_with_country)
    }

    pub async fn get_state(&self, id: Uuid) -> Result<StateDto, ServiceError> {
        self.state_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Estado não encontrado".to_string()))
    }

    pub async fn get_state_with_country(
        &self,
        id: Uuid,
    ) -> Result<StateWithCountryDto, ServiceError> {
        self.state_repo
            .find_with_country_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Estado não encontrado".to_string()))
    }

    pub async fn update_state(
        &self,
        id: Uuid,
        payload: UpdateStatePayload,
    ) -> Result<StateWithCountryDto, ServiceError> {
        // Check if state exists
        let _ = self.get_state(id).await?;

        // If updating abbreviation, check for duplicates
        if let Some(ref new_abbreviation) = payload.abbreviation {
            if self
                .state_repo
                .exists_by_abbreviation_excluding(new_abbreviation, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Estado com sigla '{}' já existe",
                    new_abbreviation
                )));
            }
        }

        // If updating ibge_code, check for duplicates
        if let Some(new_ibge_code) = payload.ibge_code {
            if self
                .state_repo
                .exists_by_ibge_code_excluding(new_ibge_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Estado com código IBGE '{}' já existe",
                    new_ibge_code
                )));
            }
        }

        // If updating country, validate it exists
        if let Some(country_id) = payload.country_id {
            if self.country_repo.find_by_id(country_id).await?.is_none() {
                return Err(ServiceError::NotFound("País não encontrado".to_string()));
            }
        }

        let state = self
            .state_repo
            .update(
                id,
                payload.name.as_ref(),
                payload.abbreviation.as_ref(),
                payload.ibge_code,
                payload.country_id,
                payload.is_active.unwrap_or(true),
            )
            .await?;

        // Return state with country information
        let state_with_country = self
            .state_repo
            .find_with_country_by_id(state.id)
            .await?
            .ok_or(ServiceError::NotFound("Estado não encontrado".to_string()))?;

        Ok(state_with_country)
    }

    pub async fn delete_state(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.state_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Estado não encontrado".to_string()))
        }
    }

    pub async fn list_states(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
        country_id: Option<Uuid>,
    ) -> Result<Paginated<StateWithCountryDto>, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (items, total) = self
            .state_repo
            .list(limit, offset, search, country_id)
            .await?;

        Ok(Paginated::new(items, total, limit, offset))
    }

    // ============================
    // City Operations
    // ============================

    pub async fn create_city(&self, payload: CreateCityPayload) -> Result<CityDto, ServiceError> {
        // Check if ibge_code already exists
        if self
            .city_repo
            .exists_by_ibge_code(payload.ibge_code)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Cidade com código IBGE '{}' já existe",
                payload.ibge_code
            )));
        }

        // Verify that state exists
        let _ = self.get_state(payload.state_id).await?;

        let city = self
            .city_repo
            .create(
                &payload.name,
                payload.ibge_code,
                payload.siafi_code,
                payload.state_id,
                payload.is_active.unwrap_or(true),
            )
            .await?;

        Ok(city)
    }

    pub async fn get_city(&self, id: Uuid) -> Result<CityWithStateDto, ServiceError> {
        self.city_repo
            .find_with_state_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Cidade não encontrada".to_string()))
    }

    pub async fn update_city(
        &self,
        id: Uuid,
        payload: UpdateCityPayload,
    ) -> Result<CityDto, ServiceError> {
        // Check if city exists
        let _ = self.get_city(id).await?;

        // If updating ibge_code, check for duplicates
        if let Some(new_ibge_code) = payload.ibge_code {
            if self
                .city_repo
                .exists_by_ibge_code_excluding(new_ibge_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Cidade com código IBGE '{}' já existe",
                    new_ibge_code
                )));
            }
        }

        // If updating state_id, verify that new state exists
        if let Some(new_state_id) = payload.state_id {
            let _ = self.get_state(new_state_id).await?;
        }

        let city = self
            .city_repo
            .update(
                id,
                payload.name.as_ref(),
                payload.ibge_code,
                payload.siafi_code,
                payload.state_id,
                payload.is_active.unwrap_or(true),
            )
            .await?;

        Ok(city)
    }

    pub async fn delete_city(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.city_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Cidade não encontrada".to_string()))
        }
    }

    pub async fn list_cities(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
        state_id: Option<Uuid>,
    ) -> Result<Paginated<CityWithStateDto>, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (items, total) = self.city_repo.list(limit, offset, search, state_id).await?;

        Ok(Paginated::new(items, total, limit, offset))
    }
}
