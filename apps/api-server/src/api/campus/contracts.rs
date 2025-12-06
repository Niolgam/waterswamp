use domain::models::campus::{Campus, CoordinatesDto};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// RESPONSES
// =============================================================================

#[derive(Serialize)]
pub struct CampusResponse {
    pub id: String,
    pub name: String,
    pub acronym: String,
    pub city_id: String,
    pub coordinates: CoordinatesDto,
    pub address: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Campus> for CampusResponse {
    fn from(campus: Campus) -> Self {
        Self {
            id: campus.id.to_string(),
            name: campus.name,
            acronym: campus.acronym,
            city_id: campus.city_id.to_string(),
            coordinates: campus.coordinates.into(),
            address: campus.address,
            created_at: campus.created_at.to_rfc3339(),
            updated_at: campus.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct CampusListResponse {
    pub campuses: Vec<CampusResponse>,
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
pub struct CreateCampusRequest {
    #[validate(length(min = 3, max = 200, message = "Name must be between 3 and 200 characters"))]
    pub name: String,

    #[validate(length(min = 2, max = 10, message = "Acronym must be between 2 and 10 characters"))]
    pub acronym: String,

    pub city_id: Uuid,

    pub coordinates: CoordinatesDto,

    #[validate(length(min = 10, max = 500, message = "Address must be between 10 and 500 characters"))]
    pub address: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateCampusRequest {
    #[validate(length(min = 3, max = 200, message = "Name must be between 3 and 200 characters"))]
    pub name: Option<String>,

    #[validate(length(min = 2, max = 10, message = "Acronym must be between 2 and 10 characters"))]
    pub acronym: Option<String>,

    pub city_id: Option<Uuid>,

    pub coordinates: Option<CoordinatesDto>,

    #[validate(length(min = 10, max = 500, message = "Address must be between 10 and 500 characters"))]
    pub address: Option<String>,
}
