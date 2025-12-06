use domain::models::UnitCategory;
use serde::{Deserialize, Serialize};
use validator::Validate;

// =============================================================================
// RESPONSES
// =============================================================================

#[derive(Serialize)]
pub struct UnitCategoryResponse {
    pub id: String,
    pub name: String,
    pub color_hex: String,
    pub created_at: String,
}

impl From<UnitCategory> for UnitCategoryResponse {
    fn from(category: UnitCategory) -> Self {
        Self {
            id: category.id.to_string(),
            name: category.name,
            color_hex: category.color_hex,
            created_at: category.created_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct UnitCategoryListResponse {
    pub categories: Vec<UnitCategoryResponse>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// =============================================================================
// REQUESTS
// =============================================================================

#[derive(Deserialize, Validate)]
pub struct CreateUnitCategoryRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(length(equal = 7, message = "Color must be a valid hex color (#RRGGBB)"))]
    pub color_hex: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateUnitCategoryRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,

    #[validate(length(equal = 7, message = "Color must be a valid hex color (#RRGGBB)"))]
    pub color_hex: Option<String>,
}
