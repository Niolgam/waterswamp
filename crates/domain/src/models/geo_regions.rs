use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{LocationName, StateCode};

// ============================
// Country Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct CountryDto {
    pub id: Uuid,
    pub name: LocationName,
    pub iso2: String,    // ISO 3166-1 alpha-2 code (BR, US, etc)
    pub bacen_code: i32, // Código Bacen (Brasil é 1058)
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListCountriesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateCountryPayload {
    pub name: LocationName,
    #[validate(length(equal = 2))]
    pub iso2: String, // ISO 3166-1 alpha-2 code
    pub bacen_code: i32, // Código Bacen
    pub is_active: bool,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct UpdateCountryPayload {
    pub name: Option<LocationName>,
    #[validate(length(equal = 2))]
    pub iso2: Option<String>,
    pub bacen_code: Option<i32>,
    pub is_active: bool,
}

// ============================
// State Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct StateDto {
    pub id: Uuid,
    pub name: LocationName,
    pub abbreviation: StateCode,
    pub ibge_code: i32, // Código IBGE (cUF da NF-e)
    pub country_id: Uuid,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct StateWithCountryDto {
    pub id: Uuid,
    pub name: LocationName,
    pub abbreviation: StateCode,
    pub ibge_code: i32,
    pub is_active: bool,
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_iso2: String,
    pub country_bacen_code: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListStatesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub country_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateStatePayload {
    pub name: LocationName,
    pub abbreviation: StateCode,
    pub ibge_code: i32,
    pub country_id: Uuid,
    pub is_active: bool,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct UpdateStatePayload {
    pub name: Option<LocationName>,
    pub abbreviation: Option<StateCode>,
    pub ibge_code: Option<i32>,
    pub country_id: Option<Uuid>,
    pub is_active: bool,
}

// ============================
// City Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct CityDto {
    pub id: Uuid,
    pub name: LocationName,
    pub ibge_code: i32,          // Código IBGE do município
    pub siafi_code: Option<i32>, // Código SIAFI do município
    pub state_id: Uuid,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct CityWithStateDto {
    pub id: Uuid,
    pub name: LocationName,
    pub ibge_code: i32,
    pub siafi_code: Option<i32>,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_abbreviation: StateCode,
    pub state_ibge_code: i32,
    pub is_active: bool,
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_iso2: String,
    pub country_bacen_code: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListCitiesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub state_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateCityPayload {
    pub name: LocationName,
    pub ibge_code: i32,
    pub siafi_code: Option<i32>,
    pub state_id: Uuid,
    pub is_active: bool,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct UpdateCityPayload {
    pub name: Option<LocationName>,
    pub ibge_code: Option<i32>,
    pub siafi_code: Option<i32>,
    pub state_id: Option<Uuid>,
    pub is_active: bool,
}
