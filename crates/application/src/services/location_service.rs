use crate::errors::ServiceError;
use domain::models::{
    CityDto, CityWithStateDto, CreateCityPayload, CreateSiteTypePayload, CreateStatePayload,
    PaginatedCities, PaginatedSiteTypes, PaginatedStates, SiteTypeDto, StateDto,
    UpdateCityPayload, UpdateSiteTypePayload, UpdateStatePayload,
};
use domain::ports::{CityRepositoryPort, SiteTypeRepositoryPort, StateRepositoryPort};
use std::sync::Arc;
use uuid::Uuid;

pub struct LocationService {
    state_repo: Arc<dyn StateRepositoryPort>,
    city_repo: Arc<dyn CityRepositoryPort>,
    site_type_repo: Arc<dyn SiteTypeRepositoryPort>,
}

impl LocationService {
    pub fn new(
        state_repo: Arc<dyn StateRepositoryPort>,
        city_repo: Arc<dyn CityRepositoryPort>,
        site_type_repo: Arc<dyn SiteTypeRepositoryPort>,
    ) -> Self {
        Self {
            state_repo,
            city_repo,
            site_type_repo,
        }
    }

    // ============================
    // State Operations
    // ============================

    pub async fn create_state(&self, payload: CreateStatePayload) -> Result<StateDto, ServiceError> {
        // Check if state code already exists
        if self.state_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!(
                "Estado com código '{}' já existe",
                payload.code
            )));
        }

        let state = self
            .state_repo
            .create(&payload.name, &payload.code)
            .await?;

        Ok(state)
    }

    pub async fn get_state(&self, id: Uuid) -> Result<StateDto, ServiceError> {
        self.state_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Estado não encontrado".to_string()))
    }

    pub async fn update_state(
        &self,
        id: Uuid,
        payload: UpdateStatePayload,
    ) -> Result<StateDto, ServiceError> {
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

        let state = self
            .state_repo
            .update(id, payload.name.as_ref(), payload.code.as_ref())
            .await?;

        Ok(state)
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
    ) -> Result<PaginatedStates, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (states, total) = self.state_repo.list(limit, offset, search).await?;

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

    // ============================
    // Site Type Operations
    // ============================

    pub async fn create_site_type(
        &self,
        payload: CreateSiteTypePayload,
    ) -> Result<SiteTypeDto, ServiceError> {
        // Check if site type name already exists
        if self.site_type_repo.exists_by_name(&payload.name).await? {
            return Err(ServiceError::Conflict(format!(
                "Tipo de site com nome '{}' já existe",
                payload.name
            )));
        }

        let site_type = self
            .site_type_repo
            .create(&payload.name, payload.description.as_deref())
            .await?;

        Ok(site_type)
    }

    pub async fn get_site_type(&self, id: Uuid) -> Result<SiteTypeDto, ServiceError> {
        self.site_type_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Tipo de site não encontrado".to_string(),
            ))
    }

    pub async fn update_site_type(
        &self,
        id: Uuid,
        payload: UpdateSiteTypePayload,
    ) -> Result<SiteTypeDto, ServiceError> {
        // Check if site type exists
        let _ = self.get_site_type(id).await?;

        // If updating name, check for duplicates
        if let Some(ref new_name) = payload.name {
            if self
                .site_type_repo
                .exists_by_name_excluding(new_name, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Tipo de site com nome '{}' já existe",
                    new_name
                )));
            }
        }

        let site_type = self
            .site_type_repo
            .update(id, payload.name.as_ref(), payload.description.as_deref())
            .await?;

        Ok(site_type)
    }

    pub async fn delete_site_type(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.site_type_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Tipo de site não encontrado".to_string(),
            ))
        }
    }

    pub async fn list_site_types(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
    ) -> Result<PaginatedSiteTypes, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (site_types, total) = self.site_type_repo.list(limit, offset, search).await?;

        Ok(PaginatedSiteTypes {
            site_types,
            total,
            limit,
            offset,
        })
    }
}
