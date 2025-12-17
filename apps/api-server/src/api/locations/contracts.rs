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

// ============================
// Building Type Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingTypeResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Space Type Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceTypeResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Department Category Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentCategoryResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Site Response DTOs (Phase 3A)
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub city_id: Uuid,
    pub city_name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub site_type_id: Uuid,
    pub site_type_name: LocationName,
    pub address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Building Response DTOs (Phase 3B)
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub site_id: Uuid,
    pub site_name: LocationName,
    pub city_id: Uuid,
    pub city_name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub building_type_id: Uuid,
    pub building_type_name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Floor Response DTOs (Phase 3C)
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct FloorResponse {
    pub id: Uuid,
    pub floor_number: i32,
    pub building_id: Uuid,
    pub building_name: LocationName,
    pub site_id: Uuid,
    pub site_name: LocationName,
    pub city_id: Uuid,
    pub city_name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Space Response DTOs (Phase 3D)
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub floor_id: Uuid,
    pub floor_number: i32,
    pub building_id: Uuid,
    pub building_name: LocationName,
    pub site_id: Uuid,
    pub site_name: LocationName,
    pub city_id: Uuid,
    pub city_name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub space_type_id: Uuid,
    pub space_type_name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
