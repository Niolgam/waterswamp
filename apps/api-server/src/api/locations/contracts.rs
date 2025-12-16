use domain::value_objects::{LocationName, StateCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================
// State Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct StateResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub code: StateCode,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// City Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct CityResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub state_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CityWithStateResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Site Type Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteTypeResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
