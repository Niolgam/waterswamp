//! Public API Contracts for Map Display
//!
//! These DTOs are specifically designed for the frontend map display
//! and include geographic coordinates, bounds, counts, and other UI-specific fields.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// COMMON STRUCTURES
// =============================================================================

/// Geographic coordinates in GeoJSON format [longitude, latitude]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coordinates {
    pub lng: f64,
    pub lat: f64,
}

/// Geographic bounds for a site
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bounds {
    #[serde(rename = "minLng")]
    pub min_lng: f64,
    #[serde(rename = "minLat")]
    pub min_lat: f64,
    #[serde(rename = "maxLng")]
    pub max_lng: f64,
    #[serde(rename = "maxLat")]
    pub max_lat: f64,
}

// =============================================================================
// TYPE RESPONSES (for filters and legends)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicBuildingTypeResponse {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub description: Option<String>,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicSpaceTypeResponse {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub description: Option<String>,
    pub count: i64,
}

// =============================================================================
// SITE RESPONSES
// =============================================================================

/// Simplified site info for site listing
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicSiteListResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub code: Option<String>,
    #[serde(rename = "buildingCount")]
    pub building_count: i64,
    #[serde(rename = "spaceCount")]
    pub space_count: i64,
    pub bounds: Bounds,
    pub center: Coordinates,
    #[serde(rename = "defaultZoom")]
    pub default_zoom: i32,
}

/// Detailed site info for single site view
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicSiteDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub code: Option<String>,
    #[serde(rename = "cityId")]
    pub city_id: Uuid,
    #[serde(rename = "siteTypeId")]
    pub site_type_id: Uuid,
    #[serde(rename = "buildingCount")]
    pub building_count: i64,
    #[serde(rename = "spaceCount")]
    pub space_count: i64,
    pub bounds: Bounds,
    pub center: Coordinates,
    #[serde(rename = "defaultZoom")]
    pub default_zoom: i32,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// BUILDING RESPONSES
// =============================================================================

/// Nested building type info for building response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildingTypeInfo {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub description: Option<String>,
}

/// Building info for listing (includes coordinates as polygon)
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicBuildingResponse {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    #[serde(rename = "siteId")]
    pub site_id: Uuid,
    #[serde(rename = "buildingTypeId")]
    pub building_type_id: Uuid,
    #[serde(rename = "buildingType")]
    pub building_type: BuildingTypeInfo,
    #[serde(rename = "totalFloors")]
    pub total_floors: Option<i32>,
    #[serde(rename = "floorCount")]
    pub floor_count: i64,
    #[serde(rename = "spaceCount")]
    pub space_count: i64,
    pub address: Option<String>,
    /// Polygon coordinates in GeoJSON format: [[lng, lat], [lng, lat], ...]
    pub coordinates: Vec<Vec<f64>>,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Building detail with floors list
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicBuildingDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    #[serde(rename = "siteId")]
    pub site_id: Uuid,
    #[serde(rename = "buildingTypeId")]
    pub building_type_id: Uuid,
    #[serde(rename = "buildingType")]
    pub building_type: BuildingTypeInfo,
    #[serde(rename = "totalFloors")]
    pub total_floors: Option<i32>,
    #[serde(rename = "floorCount")]
    pub floor_count: i64,
    #[serde(rename = "spaceCount")]
    pub space_count: i64,
    pub address: Option<String>,
    pub coordinates: Vec<Vec<f64>>,
    pub floors: Vec<FloorInfo>,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FloorInfo {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "floorNumber")]
    pub floor_number: i32,
    #[serde(rename = "spaceCount")]
    pub space_count: i64,
}

// =============================================================================
// SPACE RESPONSES
// =============================================================================

/// Nested space type info for space response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpaceTypeInfo {
    pub id: Uuid,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub description: Option<String>,
}

/// Floor info for space response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpaceFloorInfo {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "floorNumber")]
    pub floor_number: i32,
    #[serde(rename = "buildingId")]
    pub building_id: Uuid,
    pub building: Option<SpaceBuildingInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpaceBuildingInfo {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "siteId")]
    pub site_id: Uuid,
}

/// Space coordinates - either point or polygon
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SpaceCoordinates {
    Point(Vec<f64>),        // [lng, lat]
    Polygon(Vec<Vec<f64>>), // [[lng, lat], [lng, lat], ...]
}

/// Space info for listing
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicSpaceResponse {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    #[serde(rename = "floorId")]
    pub floor_id: Uuid,
    #[serde(rename = "spaceTypeId")]
    pub space_type_id: Uuid,
    #[serde(rename = "spaceType")]
    pub space_type: SpaceTypeInfo,
    #[serde(rename = "locationType")]
    pub location_type: String, // "point" or "polygon"
    pub coordinates: SpaceCoordinates,
    pub capacity: Option<i32>,
    pub area: Option<f64>,
    pub description: Option<String>,
    pub floor: SpaceFloorInfo,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// SEARCH RESPONSE
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchResultItem {
    #[serde(rename = "building")]
    Building {
        id: Uuid,
        name: String,
        code: Option<String>,
        #[serde(rename = "siteId")]
        site_id: Uuid,
        #[serde(rename = "buildingTypeId")]
        building_type_id: Uuid,
        #[serde(rename = "buildingType")]
        building_type: BuildingTypeInfo,
        coordinates: Vec<Vec<f64>>,
        #[serde(rename = "matchType")]
        match_type: String,
    },
    #[serde(rename = "space")]
    Space {
        id: Uuid,
        name: String,
        code: Option<String>,
        #[serde(rename = "floorId")]
        floor_id: Uuid,
        #[serde(rename = "spaceTypeId")]
        space_type_id: Uuid,
        #[serde(rename = "spaceType")]
        space_type: SpaceTypeInfo,
        #[serde(rename = "locationType")]
        location_type: String,
        coordinates: SpaceCoordinates,
        floor: SpaceFloorInfo,
        #[serde(rename = "matchType")]
        match_type: String,
    },
}

// =============================================================================
// WRAPPER RESPONSES (standardized API format)
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedApiResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMeta {
    pub query: String,
    pub total: i64,
    pub limit: i64,
}
