use crate::errors::ServiceError;
use domain::models::{CreateOrganizationalUnitDto, OrganizationalUnit, UpdateOrganizationalUnitDto};
use domain::ports::{OrganizationalUnitRepositoryPort, UnitCategoryRepositoryPort};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Application service for OrganizationalUnit
pub struct OrganizationalUnitService {
    org_unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
    category_repo: Arc<dyn UnitCategoryRepositoryPort>,
}

impl OrganizationalUnitService {
    pub fn new(
        org_unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
        category_repo: Arc<dyn UnitCategoryRepositoryPort>,
    ) -> Self {
        Self {
            org_unit_repo,
            category_repo,
        }
    }

    /// Creates a new organizational unit
    pub async fn create_unit(&self, dto: CreateOrganizationalUnitDto) -> Result<OrganizationalUnit, ServiceError> {
        // Validate DTO
        dto.validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Validate that category exists
        self.category_repo
            .find_by_id(dto.category_id)
            .await?
            .ok_or_else(|| {
                ServiceError::ValidationError(format!(
                    "Category with ID {} does not exist",
                    dto.category_id
                ))
            })?;

        // Validate that parent exists (if provided)
        if let Some(parent_id) = dto.parent_id {
            self.org_unit_repo
                .find_by_id(parent_id)
                .await?
                .ok_or_else(|| {
                    ServiceError::ValidationError(format!(
                        "Parent unit with ID {} does not exist",
                        parent_id
                    ))
                })?;
        }

        // Create unit
        self.org_unit_repo
            .create(&dto)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Finds unit by ID
    pub async fn get_unit(&self, id: Uuid) -> Result<OrganizationalUnit, ServiceError> {
        self.org_unit_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or_else(|| ServiceError::NotFound(format!("Organizational unit with ID {} not found", id)))
    }

    /// Finds unit by acronym
    pub async fn get_unit_by_acronym(&self, acronym: &str) -> Result<OrganizationalUnit, ServiceError> {
        self.org_unit_repo
            .find_by_acronym(acronym)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Organizational unit with acronym '{}' not found", acronym))
            })
    }

    /// Lists all units
    pub async fn list_all_units(&self) -> Result<Vec<OrganizationalUnit>, ServiceError> {
        self.org_unit_repo
            .list_all()
            .await
            .map_err(ServiceError::Repository)
    }

    /// Lists units by parent
    pub async fn list_by_parent(&self, parent_id: Option<Uuid>) -> Result<Vec<OrganizationalUnit>, ServiceError> {
        self.org_unit_repo
            .list_by_parent(parent_id)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Lists root units (units without parent)
    pub async fn list_root_units(&self) -> Result<Vec<OrganizationalUnit>, ServiceError> {
        self.org_unit_repo
            .list_root_units()
            .await
            .map_err(ServiceError::Repository)
    }

    /// Lists units by category
    pub async fn list_by_category(&self, category_id: Uuid) -> Result<Vec<OrganizationalUnit>, ServiceError> {
        self.org_unit_repo
            .list_by_category(category_id)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Lists units by campus
    pub async fn list_by_campus(&self, campus_id: Uuid) -> Result<Vec<OrganizationalUnit>, ServiceError> {
        self.org_unit_repo
            .list_by_campus(campus_id)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Updates existing unit
    pub async fn update_unit(
        &self,
        id: Uuid,
        dto: UpdateOrganizationalUnitDto,
    ) -> Result<OrganizationalUnit, ServiceError> {
        // Validate DTO
        dto.validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Check if unit exists
        self.get_unit(id).await?;

        // Validate that new category exists (if provided)
        if let Some(category_id) = dto.category_id {
            self.category_repo
                .find_by_id(category_id)
                .await?
                .ok_or_else(|| {
                    ServiceError::ValidationError(format!(
                        "Category with ID {} does not exist",
                        category_id
                    ))
                })?;
        }

        // Validate that new parent exists (if provided)
        if let Some(parent_id) = dto.parent_id {
            // Prevent circular reference
            if parent_id == id {
                return Err(ServiceError::ValidationError(
                    "Unit cannot be its own parent".to_string(),
                ));
            }

            self.org_unit_repo
                .find_by_id(parent_id)
                .await?
                .ok_or_else(|| {
                    ServiceError::ValidationError(format!(
                        "Parent unit with ID {} does not exist",
                        parent_id
                    ))
                })?;
        }

        // Update unit
        self.org_unit_repo
            .update(id, &dto)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Deletes unit
    pub async fn delete_unit(&self, id: Uuid) -> Result<(), ServiceError> {
        // Check if unit exists
        self.get_unit(id).await?;

        // Delete
        let deleted = self
            .org_unit_repo
            .delete(id)
            .await
            .map_err(ServiceError::Repository)?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::Internal(anyhow::anyhow!(
                "Failed to delete organizational unit"
            )))
        }
    }

    /// Counts total units
    pub async fn count_units(&self) -> Result<i64, ServiceError> {
        self.org_unit_repo
            .count()
            .await
            .map_err(ServiceError::Repository)
    }
}
