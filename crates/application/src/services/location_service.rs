use crate::errors::ServiceError;
use domain::models::{
    BuildingTypeDto, BuildingWithRelationsDto, CityDto, CityWithStateDto,
    CreateBuildingPayload, CreateBuildingTypePayload, CreateCityPayload,
    CreateDepartmentCategoryPayload, CreateFloorPayload, CreateSitePayload, CreateSiteTypePayload,
    CreateSpaceTypePayload, CreateStatePayload, DepartmentCategoryDto, FloorWithRelationsDto,
    PaginatedBuildings, PaginatedBuildingTypes, PaginatedCities,
    PaginatedDepartmentCategories, PaginatedFloors, PaginatedSites, PaginatedSiteTypes,
    PaginatedSpaceTypes, PaginatedStates, SiteTypeDto, SiteWithRelationsDto, SpaceTypeDto,
    StateDto, UpdateBuildingPayload, UpdateBuildingTypePayload, UpdateCityPayload,
    UpdateDepartmentCategoryPayload, UpdateFloorPayload, UpdateSitePayload, UpdateSiteTypePayload,
    UpdateSpaceTypePayload, UpdateStatePayload,
};
use domain::ports::{
    BuildingRepositoryPort, BuildingTypeRepositoryPort, CityRepositoryPort,
    DepartmentCategoryRepositoryPort, FloorRepositoryPort, SiteRepositoryPort,
    SiteTypeRepositoryPort, SpaceTypeRepositoryPort, StateRepositoryPort,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct LocationService {
    state_repo: Arc<dyn StateRepositoryPort>,
    city_repo: Arc<dyn CityRepositoryPort>,
    site_type_repo: Arc<dyn SiteTypeRepositoryPort>,
    building_type_repo: Arc<dyn BuildingTypeRepositoryPort>,
    space_type_repo: Arc<dyn SpaceTypeRepositoryPort>,
    department_category_repo: Arc<dyn DepartmentCategoryRepositoryPort>,
    site_repo: Arc<dyn SiteRepositoryPort>,
    building_repo: Arc<dyn BuildingRepositoryPort>,
    floor_repo: Arc<dyn FloorRepositoryPort>,
}

impl LocationService {
    pub fn new(
        state_repo: Arc<dyn StateRepositoryPort>,
        city_repo: Arc<dyn CityRepositoryPort>,
        site_type_repo: Arc<dyn SiteTypeRepositoryPort>,
        building_type_repo: Arc<dyn BuildingTypeRepositoryPort>,
        space_type_repo: Arc<dyn SpaceTypeRepositoryPort>,
        department_category_repo: Arc<dyn DepartmentCategoryRepositoryPort>,
        site_repo: Arc<dyn SiteRepositoryPort>,
        building_repo: Arc<dyn BuildingRepositoryPort>,
    ) -> Self {
        Self {
            state_repo,
            city_repo,
            site_type_repo,
            building_type_repo,
            space_type_repo,
            department_category_repo,
            site_repo,
            building_repo,
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

    // ============================
    // Building Type Operations
    // ============================

    pub async fn create_building_type(
        &self,
        payload: CreateBuildingTypePayload,
    ) -> Result<BuildingTypeDto, ServiceError> {
        if self.building_type_repo.exists_by_name(&payload.name).await? {
            return Err(ServiceError::Conflict(format!(
                "Tipo de edifício com nome '{}' já existe",
                payload.name
            )));
        }

        let building_type = self
            .building_type_repo
            .create(&payload.name, payload.description.as_deref())
            .await?;

        Ok(building_type)
    }

    pub async fn get_building_type(&self, id: Uuid) -> Result<BuildingTypeDto, ServiceError> {
        self.building_type_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Tipo de edifício não encontrado".to_string(),
            ))
    }

    pub async fn update_building_type(
        &self,
        id: Uuid,
        payload: UpdateBuildingTypePayload,
    ) -> Result<BuildingTypeDto, ServiceError> {
        let _ = self.get_building_type(id).await?;

        if let Some(ref new_name) = payload.name {
            if self
                .building_type_repo
                .exists_by_name_excluding(new_name, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Tipo de edifício com nome '{}' já existe",
                    new_name
                )));
            }
        }

        let building_type = self
            .building_type_repo
            .update(id, payload.name.as_ref(), payload.description.as_deref())
            .await?;

        Ok(building_type)
    }

    pub async fn delete_building_type(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.building_type_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Tipo de edifício não encontrado".to_string(),
            ))
        }
    }

    pub async fn list_building_types(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
    ) -> Result<PaginatedBuildingTypes, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (building_types, total) = self.building_type_repo.list(limit, offset, search).await?;

        Ok(PaginatedBuildingTypes {
            building_types,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // Space Type Operations
    // ============================

    pub async fn create_space_type(
        &self,
        payload: CreateSpaceTypePayload,
    ) -> Result<SpaceTypeDto, ServiceError> {
        if self.space_type_repo.exists_by_name(&payload.name).await? {
            return Err(ServiceError::Conflict(format!(
                "Tipo de espaço com nome '{}' já existe",
                payload.name
            )));
        }

        let space_type = self
            .space_type_repo
            .create(&payload.name, payload.description.as_deref())
            .await?;

        Ok(space_type)
    }

    pub async fn get_space_type(&self, id: Uuid) -> Result<SpaceTypeDto, ServiceError> {
        self.space_type_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Tipo de espaço não encontrado".to_string(),
            ))
    }

    pub async fn update_space_type(
        &self,
        id: Uuid,
        payload: UpdateSpaceTypePayload,
    ) -> Result<SpaceTypeDto, ServiceError> {
        let _ = self.get_space_type(id).await?;

        if let Some(ref new_name) = payload.name {
            if self
                .space_type_repo
                .exists_by_name_excluding(new_name, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Tipo de espaço com nome '{}' já existe",
                    new_name
                )));
            }
        }

        let space_type = self
            .space_type_repo
            .update(id, payload.name.as_ref(), payload.description.as_deref())
            .await?;

        Ok(space_type)
    }

    pub async fn delete_space_type(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.space_type_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Tipo de espaço não encontrado".to_string(),
            ))
        }
    }

    pub async fn list_space_types(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
    ) -> Result<PaginatedSpaceTypes, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (space_types, total) = self.space_type_repo.list(limit, offset, search).await?;

        Ok(PaginatedSpaceTypes {
            space_types,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // Department Category Operations
    // ============================

    pub async fn create_department_category(
        &self,
        payload: CreateDepartmentCategoryPayload,
    ) -> Result<DepartmentCategoryDto, ServiceError> {
        if self
            .department_category_repo
            .exists_by_name(&payload.name)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Categoria de departamento com nome '{}' já existe",
                payload.name
            )));
        }

        let department_category = self
            .department_category_repo
            .create(&payload.name, payload.description.as_deref())
            .await?;

        Ok(department_category)
    }

    pub async fn get_department_category(
        &self,
        id: Uuid,
    ) -> Result<DepartmentCategoryDto, ServiceError> {
        self.department_category_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Categoria de departamento não encontrada".to_string(),
            ))
    }

    pub async fn update_department_category(
        &self,
        id: Uuid,
        payload: UpdateDepartmentCategoryPayload,
    ) -> Result<DepartmentCategoryDto, ServiceError> {
        let _ = self.get_department_category(id).await?;

        if let Some(ref new_name) = payload.name {
            if self
                .department_category_repo
                .exists_by_name_excluding(new_name, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Categoria de departamento com nome '{}' já existe",
                    new_name
                )));
            }
        }

        let department_category = self
            .department_category_repo
            .update(id, payload.name.as_ref(), payload.description.as_deref())
            .await?;

        Ok(department_category)
    }

    pub async fn delete_department_category(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.department_category_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Categoria de departamento não encontrada".to_string(),
            ))
        }
    }

    pub async fn list_department_categories(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
    ) -> Result<PaginatedDepartmentCategories, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (department_categories, total) = self
            .department_category_repo
            .list(limit, offset, search)
            .await?;

        Ok(PaginatedDepartmentCategories {
            department_categories,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // Site Operations (Phase 3A)
    // ============================

    pub async fn create_site(
        &self,
        payload: CreateSitePayload,
    ) -> Result<SiteWithRelationsDto, ServiceError> {
        // Validate that city exists
        if self.city_repo.find_by_id(payload.city_id).await?.is_none() {
            return Err(ServiceError::NotFound("Cidade não encontrada".to_string()));
        }

        // Validate that site_type exists
        if self
            .site_type_repo
            .find_by_id(payload.site_type_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound(
                "Tipo de site não encontrado".to_string(),
            ));
        }

        // Create site
        let site = self
            .site_repo
            .create(
                &payload.name,
                payload.city_id,
                payload.site_type_id,
                payload.address.as_deref(),
            )
            .await?;

        // Fetch with relations for response
        let site_with_relations = self
            .site_repo
            .find_with_relations_by_id(site.id)
            .await?
            .ok_or(ServiceError::NotFound("Site não encontrado".to_string()))?;

        Ok(site_with_relations)
    }

    pub async fn get_site(&self, id: Uuid) -> Result<SiteWithRelationsDto, ServiceError> {
        let site = self
            .site_repo
            .find_with_relations_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Site não encontrado".to_string()))?;

        Ok(site)
    }

    pub async fn update_site(
        &self,
        id: Uuid,
        payload: UpdateSitePayload,
    ) -> Result<SiteWithRelationsDto, ServiceError> {
        // Validate site exists
        if self.site_repo.find_by_id(id).await?.is_none() {
            return Err(ServiceError::NotFound("Site não encontrado".to_string()));
        }

        // Validate city if provided
        if let Some(city_id) = payload.city_id {
            if self.city_repo.find_by_id(city_id).await?.is_none() {
                return Err(ServiceError::NotFound("Cidade não encontrada".to_string()));
            }
        }

        // Validate site_type if provided
        if let Some(site_type_id) = payload.site_type_id {
            if self
                .site_type_repo
                .find_by_id(site_type_id)
                .await?
                .is_none()
            {
                return Err(ServiceError::NotFound(
                    "Tipo de site não encontrado".to_string(),
                ));
            }
        }

        // Update site
        let site = self
            .site_repo
            .update(
                id,
                payload.name.as_ref(),
                payload.city_id,
                payload.site_type_id,
                payload.address.as_deref(),
            )
            .await?;

        // Fetch with relations for response
        let site_with_relations = self
            .site_repo
            .find_with_relations_by_id(site.id)
            .await?
            .ok_or(ServiceError::NotFound("Site não encontrado".to_string()))?;

        Ok(site_with_relations)
    }

    pub async fn delete_site(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.site_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Site não encontrado".to_string()))
        }
    }

    pub async fn list_sites(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
        city_id: Option<Uuid>,
        site_type_id: Option<Uuid>,
    ) -> Result<PaginatedSites, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (sites, total) = self
            .site_repo
            .list(limit, offset, search, city_id, site_type_id)
            .await?;

        Ok(PaginatedSites {
            sites,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // Building Operations (Phase 3B)
    // ============================

    pub async fn create_building(
        &self,
        payload: CreateBuildingPayload,
    ) -> Result<BuildingWithRelationsDto, ServiceError> {
        // Validate that site exists
        if self.site_repo.find_by_id(payload.site_id).await?.is_none() {
            return Err(ServiceError::NotFound("Site não encontrado".to_string()));
        }

        // Validate that building_type exists
        if self
            .building_type_repo
            .find_by_id(payload.building_type_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::NotFound(
                "Tipo de edifício não encontrado".to_string(),
            ));
        }

        // Create building
        let building = self
            .building_repo
            .create(
                &payload.name,
                payload.site_id,
                payload.building_type_id,
                payload.description.as_deref(),
            )
            .await?;

        // Fetch with relations for response
        let building_with_relations = self
            .building_repo
            .find_with_relations_by_id(building.id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Edifício não encontrado".to_string(),
            ))?;

        Ok(building_with_relations)
    }

    pub async fn get_building(&self, id: Uuid) -> Result<BuildingWithRelationsDto, ServiceError> {
        self.building_repo
            .find_with_relations_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Edifício não encontrado".to_string(),
            ))
    }

    pub async fn update_building(
        &self,
        id: Uuid,
        payload: UpdateBuildingPayload,
    ) -> Result<BuildingWithRelationsDto, ServiceError> {
        // Validate building exists
        if self.building_repo.find_by_id(id).await?.is_none() {
            return Err(ServiceError::NotFound(
                "Edifício não encontrado".to_string(),
            ));
        }

        // Validate site if provided
        if let Some(site_id) = payload.site_id {
            if self.site_repo.find_by_id(site_id).await?.is_none() {
                return Err(ServiceError::NotFound("Site não encontrado".to_string()));
            }
        }

        // Validate building_type if provided
        if let Some(building_type_id) = payload.building_type_id {
            if self
                .building_type_repo
                .find_by_id(building_type_id)
                .await?
                .is_none()
            {
                return Err(ServiceError::NotFound(
                    "Tipo de edifício não encontrado".to_string(),
                ));
            }
        }

        // Update building
        let building = self
            .building_repo
            .update(
                id,
                payload.name.as_ref(),
                payload.site_id,
                payload.building_type_id,
                payload.description.as_deref(),
            )
            .await?;

        // Fetch with relations for response
        let building_with_relations = self
            .building_repo
            .find_with_relations_by_id(building.id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Edifício não encontrado".to_string(),
            ))?;

        Ok(building_with_relations)
    }

    pub async fn delete_building(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.building_repo.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound(
                "Edifício não encontrado".to_string(),
            ))
        }
    }

    pub async fn list_buildings(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
        site_id: Option<Uuid>,
        building_type_id: Option<Uuid>,
    ) -> Result<PaginatedBuildings, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (buildings, total) = self
            .building_repo
            .list(limit, offset, search, site_id, building_type_id)
            .await?;

        Ok(PaginatedBuildings {
            buildings,
            total,
            limit,
            offset,
        })
    }
}
