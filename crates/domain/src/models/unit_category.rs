use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Unit Category entity - represents a category of organizational units
/// Examples: "Administrative", "Institute", "Department", "Pro-Rectory"
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UnitCategory {
    pub id: Uuid,
    pub name: String,
    pub color_hex: String, // For UI visualization (e.g., "#3B82F6")
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a UnitCategory
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUnitCategoryDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(length(equal = 7, message = "Color must be a valid hex color (#RRGGBB)"))]
    pub color_hex: String,
}

/// DTO for updating a UnitCategory
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUnitCategoryDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,

    #[validate(length(equal = 7, message = "Color must be a valid hex color (#RRGGBB)"))]
    pub color_hex: Option<String>,
}
