use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Organizational Unit entity - represents a unit in the university's organizational chart
/// Examples: Reitoria, Pro-Reitoria de Ensino, Instituto de Física, etc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationalUnit {
    pub id: Uuid,
    pub name: String,
    pub acronym: Option<String>,
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
    pub is_uorg: bool, // Is it a formal organizational unit?
    pub campus_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating an OrganizationalUnit
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrganizationalUnitDto {
    #[validate(length(min = 2, max = 500, message = "Name must be between 2 and 500 characters"))]
    pub name: String,

    #[validate(length(max = 30, message = "Acronym must be at most 30 characters"))]
    pub acronym: Option<String>,

    pub category_id: Uuid,

    pub parent_id: Option<Uuid>,

    #[validate(length(max = 2000, message = "Description must be at most 2000 characters"))]
    pub description: Option<String>,

    pub is_uorg: Option<bool>,

    pub campus_id: Option<Uuid>,
}

/// DTO for updating an OrganizationalUnit
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateOrganizationalUnitDto {
    #[validate(length(min = 2, max = 500, message = "Name must be between 2 and 500 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 30, message = "Acronym must be at most 30 characters"))]
    pub acronym: Option<String>,

    pub category_id: Option<Uuid>,

    pub parent_id: Option<Uuid>,

    #[validate(length(max = 2000, message = "Description must be at most 2000 characters"))]
    pub description: Option<String>,

    pub is_uorg: Option<bool>,

    pub campus_id: Option<Uuid>,
}
