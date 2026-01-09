use crate::errors::ServiceError;
use domain::models::{
    CityDto, CityWithStateDto, CountryDto, CreateCityPayload, CreateCountryPayload,
    CreateStatePayload, PaginatedCities, PaginatedCountries, PaginatedStates, StateDto,
    StateWithCountryDto, UpdateCityPayload, UpdateCountryPayload, UpdateStatePayload,
};
use domain::ports::{CityRepositoryPort, CountryRepositoryPort, StateRepositoryPort};
use std::sync::Arc;
use uuid::Uuid;

pub struct LocationService {
    country_repo: Arc<dyn CountryRepositoryPort>,
    state_repo: Arc<dyn StateRepositoryPort>,
    city_repo: Arc<dyn CityRepositoryPort>,
}

impl LocationService {
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
        // Check if country code already exists
        if self.country_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!(
                "País com código '{}' já existe",
                payload.code
            )));
        }

        let country = self
            .country_repo
            .create(&payload.name, &payload.code)
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

        // If updating code, check for duplicates
        if let Some(ref new_code) = payload.code {
            if self
                .country_repo
                .exists_by_code_excluding(new_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "País com código '{}' já existe",
                    new_code
                )));
            }
        }

        let country = self
            .country_repo
            .update(id, payload.name.as_ref(), payload.code.as_deref())
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
    ) -> Result<PaginatedCountries, ServiceError> {
        let (countries, total) = self.country_repo.list(limit, offset, search).await?;

        Ok(PaginatedCountries {
            countries,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // State Operations
    // ============================

    pub async fn create_state(
        &self,
        payload: CreateStatePayload,
    ) -> Result<StateWithCountryDto, ServiceError> {
        // Check if state code already exists
        if self.state_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!(
                "Estado com código '{}' já existe",
                payload.code
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
            .create(&payload.name, &payload.code, payload.country_id)
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

        // If updating code, check for duplicates
        if let Some(ref new_code) = payload.code {
            if self
                .state_repo
                .exists_by_code_excluding(new_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Estado com código '{}' já existe",
                    new_code
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
                payload.code.as_ref(),
                payload.country_id,
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
    ) -> Result<PaginatedStates, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (states, total) = self
            .state_repo
            .list(limit, offset, search, country_id)
            .await?;

        Ok(PaginatedStates {
            states,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // City Operations
    // ============================

    pub async fn create_city(&self, payload: CreateCityPayload) -> Result<CityDto, ServiceError> {
        // Verify that state exists
        let _ = self.get_state(payload.state_id).await?;

        let city = self
            .city_repo
            .create(&payload.name, payload.state_id)
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

        // If updating state_id, verify that new state exists
        if let Some(new_state_id) = payload.state_id {
            let _ = self.get_state(new_state_id).await?;
        }

        let city = self
            .city_repo
            .update(id, payload.name.as_ref(), payload.state_id)
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
    ) -> Result<PaginatedCities, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (cities, total) = self.city_repo.list(limit, offset, search, state_id).await?;

        Ok(PaginatedCities {
            cities,
            total,
            limit,
            offset,
        })
    }
}
