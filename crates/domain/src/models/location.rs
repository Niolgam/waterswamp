use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{LocationName, StateCode};

// ============================
// State Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct StateDto {
    pub id: Uuid,
    pub name: LocationName,
    pub code: StateCode,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedStates {
    pub states: Vec<StateDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListStatesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateStatePayload {
    pub name: LocationName,
    pub code: StateCode,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateStatePayload {
    pub name: Option<LocationName>,
    pub code: Option<StateCode>,
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

// ============================
// Site Type Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SiteTypeDto {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedSiteTypes {
    pub site_types: Vec<SiteTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListSiteTypesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateSiteTypePayload {
    pub name: LocationName,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateSiteTypePayload {
    pub name: Option<LocationName>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}
