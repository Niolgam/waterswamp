use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{LocationName, StateCode};

// ============================
// Country Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CountryDto {
    pub id: Uuid,
    pub name: LocationName,
    pub code: String, // ISO 3166-1 alpha-3 code (BRA, USA, etc)
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedCountries {
    pub countries: Vec<CountryDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListCountriesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateCountryPayload {
    pub name: LocationName,
    #[validate(length(equal = 3))]
    pub code: String, // ISO 3166-1 alpha-3 code
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateCountryPayload {
    pub name: Option<LocationName>,
    #[validate(length(equal = 3))]
    pub code: Option<String>,
}

// ============================
// State Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct StateDto {
    pub id: Uuid,
    pub name: LocationName,
    pub code: StateCode,
    pub country_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct StateWithCountryDto {
    pub id: Uuid,
    pub name: LocationName,
    pub code: StateCode,
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedStates {
    pub states: Vec<StateWithCountryDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListStatesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub country_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateStatePayload {
    pub name: LocationName,
    pub code: StateCode,
    pub country_id: Uuid,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateStatePayload {
    pub name: Option<LocationName>,
    pub code: Option<StateCode>,
    pub country_id: Option<Uuid>,
}

// ============================
// City Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CityDto {
    pub id: Uuid,
    pub name: LocationName,
    pub state_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CityWithStateDto {
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

#[derive(Debug, Serialize)]
pub struct PaginatedCities {
    pub cities: Vec<CityWithStateDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListCitiesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub state_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateCityPayload {
    pub name: LocationName,
    pub state_id: Uuid,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateCityPayload {
    pub name: Option<LocationName>,
    pub state_id: Option<Uuid>,
}
