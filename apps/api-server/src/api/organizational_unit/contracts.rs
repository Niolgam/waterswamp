use domain::models::OrganizationalUnit;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// RESPONSES
// =============================================================================

#[derive(Serialize)]
pub struct OrganizationalUnitResponse {
    pub id: String,
    pub name: String,
    pub acronym: Option<String>,
    pub category_id: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
    pub is_uorg: bool,
    pub campus_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<OrganizationalUnit> for OrganizationalUnitResponse {
    fn from(unit: OrganizationalUnit) -> Self {
        Self {
            id: unit.id.to_string(),
            name: unit.name,
            acronym: unit.acronym,
            category_id: unit.category_id.to_string(),
            parent_id: unit.parent_id.map(|id| id.to_string()),
            description: unit.description,
            is_uorg: unit.is_uorg,
            campus_id: unit.campus_id.map(|id| id.to_string()),
            created_at: unit.created_at.to_rfc3339(),
            updated_at: unit.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct OrganizationalUnitListResponse {
    pub units: Vec<OrganizationalUnitResponse>,
    pub total: i64,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// =============================================================================
// REQUESTS
// =============================================================================

#[derive(Deserialize, Validate)]
pub struct CreateOrganizationalUnitRequest {
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

#[derive(Deserialize, Validate)]
pub struct UpdateOrganizationalUnitRequest {
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
