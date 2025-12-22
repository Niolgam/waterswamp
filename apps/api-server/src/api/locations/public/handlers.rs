//! Public Handlers for Map Display
//!
//! These handlers provide PUBLIC (unauthenticated) endpoints for the frontend map.
//! They return data with geographic coordinates, counts, and other UI-specific fields.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::infra::{errors::AppError, state::AppState};
use domain::ports::{
    BuildingRepositoryPort, BuildingTypeRepositoryPort, FloorRepositoryPort, SiteRepositoryPort,
    SiteTypeRepositoryPort, SpaceRepositoryPort, SpaceTypeRepositoryPort,
};

use super::contracts::*;

// =============================================================================
// QUERY PARAMETERS
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(rename = "siteId")]
    pub site_id: Option<Uuid>,
}

// =============================================================================
// HELPER FUNCTIONS FOR GeoJSON PARSING
// =============================================================================

/// Parse GeoJSON Point: {"type": "Point", "coordinates": [lng, lat]}
fn parse_geojson_point(json: &JsonValue) -> Option<Coordinates> {
    if let Some(coords) = json.get("coordinates").and_then(|c| c.as_array()) {
        if coords.len() >= 2 {
            let lng = coords[0].as_f64()?;
            let lat = coords[1].as_f64()?;
            return Some(Coordinates { lng, lat });
        }
    }
    None
}

/// Parse GeoJSON Polygon and extract bounds
fn parse_geojson_polygon_bounds(json: &JsonValue) -> Option<Bounds> {
    if let Some(coords) = json
        .get("coordinates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|ring| ring.as_array())
    {
        let mut min_lng = f64::MAX;
        let mut min_lat = f64::MAX;
        let mut max_lng = f64::MIN;
        let mut max_lat = f64::MIN;

        for point in coords {
            if let Some(point_arr) = point.as_array() {
                if point_arr.len() >= 2 {
                    let lng = point_arr[0].as_f64()?;
                    let lat = point_arr[1].as_f64()?;

                    min_lng = min_lng.min(lng);
                    min_lat = min_lat.min(lat);
                    max_lng = max_lng.max(lng);
                    max_lat = max_lat.max(lat);
                }
            }
        }

        return Some(Bounds {
            min_lng,
            min_lat,
            max_lng,
            max_lat,
        });
    }
    None
}

/// Parse GeoJSON Polygon to coordinate array [[lng, lat], ...]
fn parse_geojson_polygon_coordinates(json: &JsonValue) -> Option<Vec<Vec<f64>>> {
    if let Some(coords) = json
        .get("coordinates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|ring| ring.as_array())
    {
        let mut result = Vec::new();
        for point in coords {
            if let Some(point_arr) = point.as_array() {
                if point_arr.len() >= 2 {
                    let lng = point_arr[0].as_f64()?;
                    let lat = point_arr[1].as_f64()?;
                    result.push(vec![lng, lat]);
                }
            }
        }
        return Some(result);
    }
    None
}

/// Parse GeoJSON to SpaceCoordinates (Point or Polygon)
fn parse_space_coordinates(json: &JsonValue, location_type: &str) -> Option<SpaceCoordinates> {
    match location_type {
        "point" => parse_geojson_point(json).map(|c| SpaceCoordinates::Point(vec![c.lng, c.lat])),
        "polygon" => parse_geojson_polygon_coordinates(json).map(SpaceCoordinates::Polygon),
        _ => None,
    }
}

// =============================================================================
// SITE ENDPOINTS
// =============================================================================

/// GET /api/locations/public/sites
///
/// Lists all available sites for the map
pub async fn list_public_sites(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PublicSiteListResponse>>>, AppError> {
    let site_repo = &state.site_repository;

    // Query all sites with relations
    let (sites, _total) = site_repo
        .list(1000, 0, None, None, None)
        .await?;

    let mut results = Vec::new();

    for site in sites {
        // Count buildings and spaces for this site
        let (buildings, _) = state
            .building_repository
            .list(10000, 0, None, Some(site.id), None)
            .await?;

        let building_count = buildings.len() as i64;

        // Count spaces across all buildings
        let mut space_count = 0i64;
        for building in &buildings {
            let (floors, _) = state
                .floor_repository
                .list(10000, 0, None, Some(building.id))
                .await?;

            for floor in floors {
                let (spaces, _) = state
                    .space_repository
                    .list(10000, 0, None, Some(floor.id), None)
                    .await?;
                space_count += spaces.len() as i64;
            }
        }

        // Parse geographic fields
        let center = site
            .center
            .as_ref()
            .and_then(parse_geojson_point)
            .unwrap_or(Coordinates {
                lng: -122.4194,
                lat: 37.7749,
            });

        let bounds = site
            .bounds
            .as_ref()
            .and_then(parse_geojson_polygon_bounds)
            .unwrap_or(Bounds {
                min_lng: -122.4194,
                min_lat: 37.7749,
                max_lng: -122.4000,
                max_lat: 37.7900,
            });

        results.push(PublicSiteListResponse {
            id: site.id,
            name: site.name.to_string(),
            description: None,
            address: site.address.clone(),
            code: site.code.clone(),
            building_count,
            space_count,
            bounds,
            center,
            default_zoom: site.default_zoom.unwrap_or(15),
        });
    }

    Ok(Json(ApiResponse {
        success: true,
        data: results,
        timestamp: chrono::Utc::now(),
    }))
}

/// GET /api/locations/public/sites/:id
///
/// Get detailed information about a specific site
pub async fn get_public_site(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PublicSiteDetailResponse>>, AppError> {
    let site_repo = &state.site_repository;

    let site = site_repo
        .find_with_relations_by_id(id)
        .await
        ?
        .ok_or_else(|| AppError::NotFound(format!("Site {} not found", id)))?;

    // Count buildings and spaces
    let (buildings, _) = state
        .building_repository
        .list(10000, 0, None, Some(site.id), None)
        .await
        ?;

    let building_count = buildings.len() as i64;

    let mut space_count = 0i64;
    for building in &buildings {
        let (floors, _) = state
            .floor_repository
            .list(10000, 0, None, Some(building.id))
            .await
            ?;

        for floor in floors {
            let (spaces, _) = state
                .space_repository
                .list(10000, 0, None, Some(floor.id), None)
                .await
                ?;
            space_count += spaces.len() as i64;
        }
    }

    // Parse geographic fields
    let center = site
        .center
        .as_ref()
        .and_then(parse_geojson_point)
        .unwrap_or(Coordinates {
            lng: -122.4194,
            lat: 37.7749,
        });

    let bounds = site
        .bounds
        .as_ref()
        .and_then(parse_geojson_polygon_bounds)
        .unwrap_or(Bounds {
            min_lng: -122.4194,
            min_lat: 37.7749,
            max_lng: -122.4000,
            max_lat: 37.7900,
        });

    Ok(Json(ApiResponse {
        success: true,
        data: PublicSiteDetailResponse {
            id: site.id,
            name: site.name.to_string(),
            description: None,
            address: site.address.clone(),
            code: site.code.clone(),
            city_id: site.city_id,
            site_type_id: site.site_type_id,
            building_count,
            space_count,
            bounds,
            center,
            default_zoom: site.default_zoom.unwrap_or(15),
            created_at: site.created_at,
            updated_at: site.updated_at,
        },
        timestamp: chrono::Utc::now(),
    }))
}

// =============================================================================
// BUILDING ENDPOINTS
// =============================================================================

/// GET /api/locations/public/sites/:siteId/buildings
///
/// Lists all buildings for a specific site
pub async fn list_site_buildings(
    State(state): State<AppState>,
    Path(site_id): Path<Uuid>,
) -> Result<Json<PaginatedApiResponse<PublicBuildingResponse>>, AppError> {
    let (buildings, total) = state
        .building_repository
        .list(1000, 0, None, Some(site_id), None)
        .await
        ?;

    let mut results = Vec::new();

    for building in buildings {
        // Get building type info
        let building_type = state
            .building_type_repository
            .find_by_id(building.building_type_id)
            .await
            ?
            .ok_or_else(|| {
                AppError::NotFound(format!("Building type {} not found", building.building_type_id))
            })?;

        // Count floors and spaces
        let (floors, _) = state
            .floor_repository
            .list(10000, 0, None, Some(building.id))
            .await
            ?;

        let floor_count = floors.len() as i64;

        let mut space_count = 0i64;
        for floor in &floors {
            let (spaces, _) = state
                .space_repository
                .list(10000, 0, None, Some(floor.id), None)
                .await
                ?;
            space_count += spaces.len() as i64;
        }

        // Parse coordinates
        let coordinates = building
            .coordinates
            .as_ref()
            .and_then(parse_geojson_polygon_coordinates)
            .unwrap_or_else(|| {
                vec![
                    vec![-122.4100, 37.7800],
                    vec![-122.4090, 37.7800],
                    vec![-122.4090, 37.7790],
                    vec![-122.4100, 37.7790],
                    vec![-122.4100, 37.7800],
                ]
            });

        results.push(PublicBuildingResponse {
            id: building.id,
            name: building.name.to_string(),
            code: building.code.clone(),
            site_id: building.site_id,
            building_type_id: building.building_type_id,
            building_type: BuildingTypeInfo {
                id: building_type.id,
                name: building_type.name.to_string(),
                icon: building_type.icon.unwrap_or_else(|| "ki-outline ki-building".to_string()),
                color: building_type.color.unwrap_or_else(|| "#FF9955".to_string()),
                description: building_type.description,
            },
            total_floors: building.total_floors,
            floor_count,
            space_count,
            address: None,
            coordinates,
            created_at: building.created_at,
            updated_at: building.updated_at,
        });
    }

    Ok(Json(PaginatedApiResponse {
        success: true,
        data: results,
        meta: PaginationMeta {
            total,
            page: 1,
            limit: 1000,
        },
    }))
}

/// GET /api/locations/public/buildings/:id
///
/// Get detailed information about a specific building
pub async fn get_public_building(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PublicBuildingDetailResponse>>, AppError> {
    let building = state
        .building_repository
        .find_with_relations_by_id(id)
        .await
        ?
        .ok_or_else(|| AppError::NotFound(format!("Building {} not found", id)))?;

    // Get building type info
    let building_type = state
        .building_type_repository
        .find_by_id(building.building_type_id)
        .await
        ?
        .ok_or_else(|| {
            AppError::NotFound(format!("Building type {} not found", building.building_type_id))
        })?;

    // Get floors with space counts
    let (floors, _) = state
        .floor_repository
        .list(10000, 0, None, Some(building.id))
        .await
        ?;

    let floor_count = floors.len() as i64;
    let mut total_space_count = 0i64;

    let mut floor_infos = Vec::new();
    for floor in floors {
        let (spaces, _) = state
            .space_repository
            .list(10000, 0, None, Some(floor.id), None)
            .await
            ?;

        let space_count = spaces.len() as i64;
        total_space_count += space_count;

        floor_infos.push(FloorInfo {
            id: floor.id,
            name: format!("Floor {}", floor.floor_number),
            floor_number: floor.floor_number,
            space_count,
        });
    }

    // Parse coordinates
    let coordinates = building
        .coordinates
        .as_ref()
        .and_then(parse_geojson_polygon_coordinates)
        .unwrap_or_else(|| {
            vec![
                vec![-122.4100, 37.7800],
                vec![-122.4090, 37.7800],
                vec![-122.4090, 37.7790],
                vec![-122.4100, 37.7790],
                vec![-122.4100, 37.7800],
            ]
        });

    Ok(Json(ApiResponse {
        success: true,
        data: PublicBuildingDetailResponse {
            id: building.id,
            name: building.name.to_string(),
            code: building.code.clone(),
            site_id: building.site_id,
            building_type_id: building.building_type_id,
            building_type: BuildingTypeInfo {
                id: building_type.id,
                name: building_type.name.to_string(),
                icon: building_type.icon.unwrap_or_else(|| "ki-outline ki-building".to_string()),
                color: building_type.color.unwrap_or_else(|| "#FF9955".to_string()),
                description: building_type.description,
            },
            total_floors: building.total_floors,
            floor_count,
            space_count: total_space_count,
            address: None,
            coordinates,
            floors: floor_infos,
            created_at: building.created_at,
            updated_at: building.updated_at,
        },
        timestamp: chrono::Utc::now(),
    }))
}

// =============================================================================
// SPACE ENDPOINTS
// =============================================================================

/// GET /api/locations/public/sites/:siteId/spaces
///
/// Lists all spaces for a specific site
pub async fn list_site_spaces(
    State(state): State<AppState>,
    Path(site_id): Path<Uuid>,
) -> Result<Json<PaginatedApiResponse<PublicSpaceResponse>>, AppError> {
    // First get all buildings for this site
    let (buildings, _) = state
        .building_repository
        .list(10000, 0, None, Some(site_id), None)
        .await
        ?;

    let mut results = Vec::new();
    let mut total_count = 0i64;

    for building in buildings {
        let (floors, _) = state
            .floor_repository
            .list(10000, 0, None, Some(building.id))
            .await
            ?;

        for floor in floors {
            let (spaces, _) = state
                .space_repository
                .list(10000, 0, None, Some(floor.id), None)
                .await
                ?;

            total_count += spaces.len() as i64;

            for space in spaces {
                // Get space type
                let space_type = state
                    .space_type_repository
                    .find_by_id(space.space_type_id)
                    .await
                    ?
                    .ok_or_else(|| {
                        AppError::NotFound(format!("Space type {} not found", space.space_type_id))
                    })?;

                // Parse coordinates
                let location_type = space.location_type.clone().unwrap_or_else(|| "point".to_string());
                let coordinates = space
                    .coordinates
                    .as_ref()
                    .and_then(|c| parse_space_coordinates(c, &location_type))
                    .unwrap_or_else(|| SpaceCoordinates::Point(vec![-122.4095, 37.7795]));

                results.push(PublicSpaceResponse {
                    id: space.id,
                    name: space.name.to_string(),
                    code: space.code.clone(),
                    floor_id: space.floor_id,
                    space_type_id: space.space_type_id,
                    space_type: SpaceTypeInfo {
                        id: space_type.id,
                        name: space_type.name.to_string(),
                        icon: space_type.icon.unwrap_or_else(|| "ki-outline ki-briefcase".to_string()),
                        color: space_type.color.unwrap_or_else(|| "#3B82F6".to_string()),
                        description: space_type.description,
                    },
                    location_type,
                    coordinates,
                    capacity: space.capacity,
                    area: space.area,
                    description: space.description.clone(),
                    floor: SpaceFloorInfo {
                        id: floor.id,
                        name: format!("Floor {}", floor.floor_number),
                        floor_number: floor.floor_number,
                        building_id: building.id,
                        building: Some(SpaceBuildingInfo {
                            id: building.id,
                            name: building.name.to_string(),
                            site_id: building.site_id,
                        }),
                    },
                    created_at: space.created_at,
                    updated_at: space.updated_at,
                });
            }
        }
    }

    Ok(Json(PaginatedApiResponse {
        success: true,
        data: results,
        meta: PaginationMeta {
            total: total_count,
            page: 1,
            limit: 10000,
        },
    }))
}

/// GET /api/locations/public/spaces/:id
///
/// Get detailed information about a specific space
pub async fn get_public_space(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PublicSpaceResponse>>, AppError> {
    let space = state
        .space_repository
        .find_with_relations_by_id(id)
        .await
        ?
        .ok_or_else(|| AppError::NotFound(format!("Space {} not found", id)))?;

    // Get space type
    let space_type = state
        .space_type_repository
        .find_by_id(space.space_type_id)
        .await
        ?
        .ok_or_else(|| {
            AppError::NotFound(format!("Space type {} not found", space.space_type_id))
        })?;

    // Parse coordinates
    let location_type = space.location_type.clone().unwrap_or_else(|| "point".to_string());
    let coordinates = space
        .coordinates
        .as_ref()
        .and_then(|c| parse_space_coordinates(c, &location_type))
        .unwrap_or_else(|| SpaceCoordinates::Point(vec![-122.4095, 37.7795]));

    Ok(Json(ApiResponse {
        success: true,
        data: PublicSpaceResponse {
            id: space.id,
            name: space.name.to_string(),
            code: space.code.clone(),
            floor_id: space.floor_id,
            space_type_id: space.space_type_id,
            space_type: SpaceTypeInfo {
                id: space_type.id,
                name: space_type.name.to_string(),
                icon: space_type.icon.unwrap_or_else(|| "ki-outline ki-briefcase".to_string()),
                color: space_type.color.unwrap_or_else(|| "#3B82F6".to_string()),
                description: space_type.description,
            },
            location_type,
            coordinates,
            capacity: space.capacity,
            area: space.area,
            description: space.description.clone(),
            floor: SpaceFloorInfo {
                id: space.floor_id,
                name: format!("Floor {}", space.floor_number),
                floor_number: space.floor_number,
                building_id: space.building_id,
                building: Some(SpaceBuildingInfo {
                    id: space.building_id,
                    name: space.building_name.to_string(),
                    site_id: space.site_id,
                }),
            },
            created_at: space.created_at,
            updated_at: space.updated_at,
        },
        timestamp: chrono::Utc::now(),
    }))
}

// =============================================================================
// SEARCH ENDPOINT
// =============================================================================

/// GET /api/locations/public/search?q=term&siteId=uuid
///
/// Search for buildings and spaces by name or code
pub async fn search_locations(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut results = Vec::new();

    // Search buildings
    let (buildings, _) = state
        .building_repository
        .list(50, 0, Some(params.q.clone()), params.site_id, None)
        .await
        ?;

    for building in buildings {
        // Get building type
        let building_type = state
            .building_type_repository
            .find_by_id(building.building_type_id)
            .await
            ?
            .ok_or_else(|| {
                AppError::NotFound(format!("Building type {} not found", building.building_type_id))
            })?;

        let coordinates = building
            .coordinates
            .as_ref()
            .and_then(parse_geojson_polygon_coordinates)
            .unwrap_or_else(|| {
                vec![
                    vec![-122.4100, 37.7800],
                    vec![-122.4090, 37.7800],
                    vec![-122.4090, 37.7790],
                    vec![-122.4100, 37.7790],
                    vec![-122.4100, 37.7800],
                ]
            });

        results.push(SearchResultItem::Building {
            id: building.id,
            name: building.name.to_string(),
            code: building.code.clone(),
            site_id: building.site_id,
            building_type_id: building.building_type_id,
            building_type: BuildingTypeInfo {
                id: building_type.id,
                name: building_type.name.to_string(),
                icon: building_type.icon.unwrap_or_else(|| "ki-outline ki-building".to_string()),
                color: building_type.color.unwrap_or_else(|| "#FF9955".to_string()),
                description: building_type.description,
            },
            coordinates,
            match_type: "name".to_string(),
        });
    }

    // Search spaces (limit to first 50 - params.site_id results)
    let space_limit = 50 - results.len() as i64;
    if space_limit > 0 {
        let (spaces, _) = state
            .space_repository
            .list(space_limit, 0, Some(params.q.clone()), None, None)
            .await
            ?;

        for space in spaces {
            // Filter by site_id if provided
            if let Some(site_id) = params.site_id {
                if space.site_id != site_id {
                    continue;
                }
            }

            // Get space type
            let space_type = state
                .space_type_repository
                .find_by_id(space.space_type_id)
                .await
                ?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Space type {} not found", space.space_type_id))
                })?;

            let location_type = space.location_type.clone().unwrap_or_else(|| "point".to_string());
            let coordinates = space
                .coordinates
                .as_ref()
                .and_then(|c| parse_space_coordinates(c, &location_type))
                .unwrap_or_else(|| SpaceCoordinates::Point(vec![-122.4095, 37.7795]));

            results.push(SearchResultItem::Space {
                id: space.id,
                name: space.name.to_string(),
                code: space.code.clone(),
                floor_id: space.floor_id,
                space_type_id: space.space_type_id,
                space_type: SpaceTypeInfo {
                    id: space_type.id,
                    name: space_type.name.to_string(),
                    icon: space_type.icon.unwrap_or_else(|| "ki-outline ki-briefcase".to_string()),
                    color: space_type.color.unwrap_or_else(|| "#3B82F6".to_string()),
                    description: space_type.description,
                },
                location_type,
                coordinates,
                floor: SpaceFloorInfo {
                    id: space.floor_id,
                    name: format!("Floor {}", space.floor_number),
                    floor_number: space.floor_number,
                    building_id: space.building_id,
                    building: None,
                },
                match_type: "name".to_string(),
            });
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": results,
        "meta": {
            "query": params.q,
            "total": results.len(),
            "limit": 50
        }
    })))
}

// =============================================================================
// TYPE ENDPOINTS (for filters)
// =============================================================================

/// GET /api/locations/public/building-types
///
/// Lists all building types with counts
pub async fn list_public_building_types(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PublicBuildingTypeResponse>>>, AppError> {
    let (building_types, _) = state
        .building_type_repository
        .list(1000, 0, None)
        .await
        ?;

    let mut results = Vec::new();

    for building_type in building_types {
        // Count buildings of this type
        let (buildings, total) = state
            .building_repository
            .list(1, 0, None, None, Some(building_type.id))
            .await
            ?;

        results.push(PublicBuildingTypeResponse {
            id: building_type.id,
            name: building_type.name.to_string(),
            icon: building_type.icon.unwrap_or_else(|| "ki-outline ki-building".to_string()),
            color: building_type.color.unwrap_or_else(|| "#FF9955".to_string()),
            description: building_type.description,
            count: total,
        });
    }

    Ok(Json(ApiResponse {
        success: true,
        data: results,
        timestamp: chrono::Utc::now(),
    }))
}

/// GET /api/locations/public/space-types
///
/// Lists all space types with counts
pub async fn list_public_space_types(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PublicSpaceTypeResponse>>>, AppError> {
    let (space_types, _) = state
        .space_type_repository
        .list(1000, 0, None)
        .await
        ?;

    let mut results = Vec::new();

    for space_type in space_types {
        // Count spaces of this type
        let (spaces, total) = state
            .space_repository
            .list(1, 0, None, None, Some(space_type.id))
            .await
            ?;

        results.push(PublicSpaceTypeResponse {
            id: space_type.id,
            name: space_type.name.to_string(),
            icon: space_type.icon.unwrap_or_else(|| "ki-outline ki-briefcase".to_string()),
            color: space_type.color.unwrap_or_else(|| "#3B82F6".to_string()),
            description: space_type.description,
            count: total,
        });
    }

    Ok(Json(ApiResponse {
        success: true,
        data: results,
        timestamp: chrono::Utc::now(),
    }))
}
