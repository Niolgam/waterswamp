use domain::value_objects::{LocationName, StateCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================
// Country Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub code: String, // ISO 3166-1 alpha-3 code
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// State Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct StateResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub code: StateCode,
    pub country_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateWithCountryResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub code: StateCode,
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
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
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
