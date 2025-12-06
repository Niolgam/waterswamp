use crate::errors::ServiceError;
use domain::models::campus::{Campus, CreateCampusDto, UpdateCampusDto};
use domain::ports::CampusRepositoryPort;
use domain::value_objects::Coordinates;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Application service for Campus
pub struct CampusService {
    campus_repo: Arc<dyn CampusRepositoryPort>,
}

impl CampusService {
    pub fn new(campus_repo: Arc<dyn CampusRepositoryPort>) -> Self {
        Self { campus_repo }
    }

    /// Creates a new campus
    pub async fn create_campus(&self, dto: CreateCampusDto) -> Result<Campus, ServiceError> {
        // Validate DTO
        dto.validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Validate coordinates
        Coordinates::new(dto.coordinates.latitude, dto.coordinates.longitude)
            .map_err(|e| ServiceError::ValidationError(e))?;

        // Check if campus with same acronym already exists
        if self.campus_repo.exists_by_acronym(&dto.acronym).await? {
            return Err(ServiceError::ValidationError(format!(
                "Campus with acronym '{}' already exists",
                dto.acronym
            )));
        }

        // Check if campus with same name already exists
        if self.campus_repo.exists_by_name(&dto.name).await? {
            return Err(ServiceError::ValidationError(format!(
                "Campus with name '{}' already exists",
                dto.name
            )));
        }

        // Create campus
        self.campus_repo
            .create(&dto)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Finds campus by ID
    pub async fn get_campus(&self, id: Uuid) -> Result<Campus, ServiceError> {
        self.campus_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or_else(|| ServiceError::NotFound(format!("Campus with ID {} not found", id)))
    }

    /// Finds campus by acronym
    pub async fn get_campus_by_acronym(&self, acronym: &str) -> Result<Campus, ServiceError> {
        self.campus_repo
            .find_by_acronym(acronym)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Campus with acronym '{}' not found", acronym))
            })
    }

    /// Finds campus by name
    pub async fn get_campus_by_name(&self, name: &str) -> Result<Campus, ServiceError> {
        self.campus_repo
            .find_by_name(name)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Campus with name '{}' not found", name))
            })
    }

    /// Lists all campuses
    pub async fn list_all_campuses(&self) -> Result<Vec<Campus>, ServiceError> {
        self.campus_repo
            .list_all()
            .await
            .map_err(ServiceError::Repository)
    }

    /// Lists campuses with pagination
    pub async fn list_campuses_paginated(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Campus>, ServiceError> {
        let limit = limit.unwrap_or(20).min(100); // Max 100 per page
        let offset = offset.unwrap_or(0);

        self.campus_repo
            .list_paginated(limit, offset)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Finds campuses by city ID
    pub async fn find_by_city(&self, city_id: Uuid) -> Result<Vec<Campus>, ServiceError> {
        self.campus_repo
            .find_by_city(city_id)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Updates existing campus
    pub async fn update_campus(
        &self,
        id: Uuid,
        dto: UpdateCampusDto,
    ) -> Result<Campus, ServiceError> {
        // Validate DTO
        dto.validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Check if campus exists
        let existing = self.get_campus(id).await?;

        // If updating coordinates, validate them
        if let Some(ref coord_dto) = dto.coordinates {
            Coordinates::new(coord_dto.latitude, coord_dto.longitude)
                .map_err(|e| ServiceError::ValidationError(e))?;
        }

        // If updating acronym, check for duplicates
        if let Some(ref new_acronym) = dto.acronym {
            if new_acronym != &existing.acronym {
                if self.campus_repo.exists_by_acronym(new_acronym).await? {
                    return Err(ServiceError::ValidationError(format!(
                        "Campus with acronym '{}' already exists",
                        new_acronym
                    )));
                }
            }
        }

        // If updating name, check for duplicates
        if let Some(ref new_name) = dto.name {
            if new_name != &existing.name {
                if self.campus_repo.exists_by_name(new_name).await? {
                    return Err(ServiceError::ValidationError(format!(
                        "Campus with name '{}' already exists",
                        new_name
                    )));
                }
            }
        }

        // Update campus
        self.campus_repo
            .update(id, &dto)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Deletes campus
    pub async fn delete_campus(&self, id: Uuid) -> Result<(), ServiceError> {
        // Check if campus exists
        self.get_campus(id).await?;

        // Delete
        let deleted = self
            .campus_repo
            .delete(id)
            .await
            .map_err(ServiceError::Repository)?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::Internal(anyhow::anyhow!(
                "Failed to delete campus"
            )))
        }
    }

    /// Counts total campuses
    pub async fn count_campuses(&self) -> Result<i64, ServiceError> {
        self.campus_repo
            .count()
            .await
            .map_err(ServiceError::Repository)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::campus::CoordinatesDto;
    use domain::ports::CampusRepositoryPort;
    use mockall::mock;
    use mockall::predicate::*;

    use domain::errors::RepositoryError;

    mock! {
        pub CampusRepo {}

        #[async_trait::async_trait]
        impl CampusRepositoryPort for CampusRepo {
            async fn create(&self, dto: &CreateCampusDto) -> Result<Campus, RepositoryError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Campus>, RepositoryError>;
            async fn find_by_acronym(&self, acronym: &str) -> Result<Option<Campus>, RepositoryError>;
            async fn find_by_name(&self, name: &str) -> Result<Option<Campus>, RepositoryError>;
            async fn list_all(&self) -> Result<Vec<Campus>, RepositoryError>;
            async fn list_paginated(&self, limit: i64, offset: i64) -> Result<Vec<Campus>, RepositoryError>;
            async fn find_by_city(&self, city_id: Uuid) -> Result<Vec<Campus>, RepositoryError>;
            async fn update(&self, id: Uuid, dto: &UpdateCampusDto) -> Result<Campus, RepositoryError>;
            async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
            async fn exists_by_acronym(&self, acronym: &str) -> Result<bool, RepositoryError>;
            async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
            async fn count(&self) -> Result<i64, RepositoryError>;
        }
    }

    fn create_test_campus() -> Campus {
        Campus {
            id: Uuid::new_v4(),
            name: "Central Campus".to_string(),
            acronym: "CC".to_string(),
            city_id: Uuid::new_v4(),
            coordinates: Coordinates::new(-23.5505, -46.6333).unwrap(),
            address: "Av. Paulista, 1000".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_campus_success() {
        let mut mock_repo = MockCampusRepo::new();

        let test_campus = create_test_campus();

        mock_repo
            .expect_exists_by_acronym()
            .with(eq("CC"))
            .returning(|_| Ok(false));

        mock_repo
            .expect_exists_by_name()
            .with(eq("Central Campus"))
            .returning(|_| Ok(false));

        mock_repo
            .expect_create()
            .returning(move |_| Ok(test_campus.clone()));

        let service = CampusService::new(Arc::new(mock_repo));

        let dto = CreateCampusDto {
            name: "Central Campus".to_string(),
            acronym: "CC".to_string(),
            city_id: Uuid::new_v4(),
            coordinates: CoordinatesDto {
                latitude: -23.5505,
                longitude: -46.6333,
            },
            address: "Av. Paulista, 1000".to_string(),
        };

        let result = service.create_campus(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_campus_duplicate_acronym() {
        let mut mock_repo = MockCampusRepo::new();

        mock_repo
            .expect_exists_by_acronym()
            .with(eq("CC"))
            .returning(|_| Ok(true)); // Already exists

        let service = CampusService::new(Arc::new(mock_repo));

        let dto = CreateCampusDto {
            name: "Central Campus".to_string(),
            acronym: "CC".to_string(),
            city_id: Uuid::new_v4(),
            coordinates: CoordinatesDto {
                latitude: -23.5505,
                longitude: -46.6333,
            },
            address: "Av. Paulista, 1000".to_string(),
        };

        let result = service.create_campus(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_campus_success() {
        let mut mock_repo = MockCampusRepo::new();
        let test_campus = create_test_campus();
        let campus_id = test_campus.id;

        mock_repo
            .expect_find_by_id()
            .with(eq(campus_id))
            .returning(move |_| Ok(Some(test_campus.clone())));

        let service = CampusService::new(Arc::new(mock_repo));

        let result = service.get_campus(campus_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_campus_not_found() {
        let mut mock_repo = MockCampusRepo::new();
        let campus_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_id()
            .with(eq(campus_id))
            .returning(|_| Ok(None));

        let service = CampusService::new(Arc::new(mock_repo));

        let result = service.get_campus(campus_id).await;
        assert!(result.is_err());
    }
}
