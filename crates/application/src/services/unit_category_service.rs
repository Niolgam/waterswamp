use crate::errors::ServiceError;
use domain::models::{CreateUnitCategoryDto, UnitCategory, UpdateUnitCategoryDto};
use domain::ports::UnitCategoryRepositoryPort;
use domain::validation::HEX_COLOR_REGEX;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Application service for UnitCategory
pub struct UnitCategoryService {
    unit_category_repo: Arc<dyn UnitCategoryRepositoryPort>,
}

impl UnitCategoryService {
    pub fn new(unit_category_repo: Arc<dyn UnitCategoryRepositoryPort>) -> Self {
        Self { unit_category_repo }
    }

    /// Creates a new unit category
    pub async fn create_category(&self, dto: CreateUnitCategoryDto) -> Result<UnitCategory, ServiceError> {
        // Validate DTO
        dto.validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Validate hex color format
        if !HEX_COLOR_REGEX.is_match(&dto.color_hex) {
            return Err(ServiceError::ValidationError(
                "Color must be in hex format (#RRGGBB)".to_string(),
            ));
        }

        // Check if category with same name already exists
        if self.unit_category_repo.exists_by_name(&dto.name).await? {
            return Err(ServiceError::ValidationError(format!(
                "Category with name '{}' already exists",
                dto.name
            )));
        }

        // Create category
        self.unit_category_repo
            .create(&dto)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Finds category by ID
    pub async fn get_category(&self, id: Uuid) -> Result<UnitCategory, ServiceError> {
        self.unit_category_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or_else(|| ServiceError::NotFound(format!("Category with ID {} not found", id)))
    }

    /// Lists all categories
    pub async fn list_all_categories(&self) -> Result<Vec<UnitCategory>, ServiceError> {
        self.unit_category_repo
            .list_all()
            .await
            .map_err(ServiceError::Repository)
    }

    /// Updates existing category
    pub async fn update_category(
        &self,
        id: Uuid,
        dto: UpdateUnitCategoryDto,
    ) -> Result<UnitCategory, ServiceError> {
        // Validate DTO
        dto.validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Validate hex color format if provided
        if let Some(ref color) = dto.color_hex {
            if !HEX_COLOR_REGEX.is_match(color) {
                return Err(ServiceError::ValidationError(
                    "Color must be in hex format (#RRGGBB)".to_string(),
                ));
            }
        }

        // Check if category exists
        let existing = self.get_category(id).await?;

        // If updating name, check for duplicates
        if let Some(ref new_name) = dto.name {
            if new_name != &existing.name {
                if self.unit_category_repo.exists_by_name(new_name).await? {
                    return Err(ServiceError::ValidationError(format!(
                        "Category with name '{}' already exists",
                        new_name
                    )));
                }
            }
        }

        // Update category
        self.unit_category_repo
            .update(id, &dto)
            .await
            .map_err(ServiceError::Repository)
    }

    /// Deletes category
    pub async fn delete_category(&self, id: Uuid) -> Result<(), ServiceError> {
        // Check if category exists
        self.get_category(id).await?;

        // Delete
        let deleted = self
            .unit_category_repo
            .delete(id)
            .await
            .map_err(ServiceError::Repository)?;

        if deleted {
            Ok(())
        } else {
            Err(ServiceError::Internal(anyhow::anyhow!(
                "Failed to delete category"
            )))
        }
    }
}
