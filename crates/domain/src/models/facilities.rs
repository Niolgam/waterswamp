use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{LocationName, StateCode};

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

// ============================
// Building Type Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BuildingTypeDto {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedBuildingTypes {
    pub building_types: Vec<BuildingTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListBuildingTypesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateBuildingTypePayload {
    pub name: LocationName,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateBuildingTypePayload {
    pub name: Option<LocationName>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// ============================
// Space Type Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SpaceTypeDto {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedSpaceTypes {
    pub space_types: Vec<SpaceTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListSpaceTypesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateSpaceTypePayload {
    pub name: LocationName,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateSpaceTypePayload {
    pub name: Option<LocationName>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// ============================
// Site Models (Phase 3A)
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SiteDto {
    pub id: Uuid,
    pub name: LocationName,
    pub city_id: Uuid,
    pub site_type_id: Uuid,
    pub address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO with joined data from city, state, country, and site_type
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SiteWithRelationsDto {
    pub id: Uuid,
    pub name: LocationName,
    pub city_id: Uuid,
    pub city_name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
    pub site_type_id: Uuid,
    pub site_type_name: LocationName,
    pub address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedSites {
    pub sites: Vec<SiteWithRelationsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListSitesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub city_id: Option<Uuid>,
    pub site_type_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateSitePayload {
    pub name: LocationName,
    pub city_id: Uuid,
    pub site_type_id: Uuid,
    #[validate(length(max = 500))]
    pub address: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateSitePayload {
    pub name: Option<LocationName>,
    pub city_id: Option<Uuid>,
    pub site_type_id: Option<Uuid>,
    #[validate(length(max = 500))]
    pub address: Option<String>,
}

// ============================
// Building Models (Phase 3B)
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BuildingDto {
    pub id: Uuid,
    pub name: LocationName,
    pub site_id: Uuid,
    pub building_type_id: Uuid,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO with joined data from site, city, state, country, and building_type
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BuildingWithRelationsDto {
    pub id: Uuid,
    pub name: LocationName,
    pub site_id: Uuid,
    pub site_name: LocationName,
    pub city_id: Uuid,
    pub city_name: LocationName,
    pub state_id: Uuid,
    pub state_name: LocationName,
    pub state_code: StateCode,
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
    pub building_type_id: Uuid,
    pub building_type_name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedBuildings {
    pub buildings: Vec<BuildingWithRelationsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListBuildingsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub site_id: Option<Uuid>,
    pub building_type_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateBuildingPayload {
    pub name: LocationName,
    pub site_id: Uuid,
    pub building_type_id: Uuid,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateBuildingPayload {
    pub name: Option<LocationName>,
    pub site_id: Option<Uuid>,
    pub building_type_id: Option<Uuid>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// ============================
// Floor Models (Phase 3C)
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct FloorDto {
    pub id: Uuid,
    pub floor_number: i32,
    pub building_id: Uuid,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO with joined data from building, site, city, state, country
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct FloorWithRelationsDto {
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
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedFloors {
    pub floors: Vec<FloorWithRelationsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListFloorsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub building_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateFloorPayload {
    pub floor_number: i32,
    pub building_id: Uuid,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateFloorPayload {
    pub floor_number: Option<i32>,
    pub building_id: Option<Uuid>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// ============================
// Space Models (Phase 3D)
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SpaceDto {
    pub id: Uuid,
    pub name: LocationName,
    pub floor_id: Uuid,
    pub space_type_id: Uuid,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO with joined data from floor, building, site, city, state, country, and space_type
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SpaceWithRelationsDto {
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
    pub country_id: Uuid,
    pub country_name: LocationName,
    pub country_code: String,
    pub space_type_id: Uuid,
    pub space_type_name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedSpaces {
    pub spaces: Vec<SpaceWithRelationsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListSpacesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub floor_id: Option<Uuid>,
    pub space_type_id: Option<Uuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateSpacePayload {
    pub name: LocationName,
    pub floor_id: Uuid,
    pub space_type_id: Uuid,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateSpacePayload {
    pub name: Option<LocationName>,
    pub floor_id: Option<Uuid>,
    pub space_type_id: Option<Uuid>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}
