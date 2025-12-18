//! Facilities Module
//!
//! Handles Site, Building, Floor, Space entities and their Type classifications.

pub mod contracts;
pub mod handlers;

pub use contracts::{
    BuildingResponse, BuildingTypeResponse, FloorResponse, SiteResponse, SiteTypeResponse,
    SpaceResponse, SpaceTypeResponse,
};

use crate::infra::state::AppState;
use axum::{routing::get, Router};

/// Creates the facilities router with all Type and Entity routes
pub fn router() -> Router<AppState> {
    let site_types_router = Router::new()
        .route(
            "/",
            get(handlers::list_site_types).post(handlers::create_site_type),
        )
        .route(
            "/{id}",
            get(handlers::get_site_type)
                .put(handlers::update_site_type)
                .delete(handlers::delete_site_type),
        );

    let building_types_router = Router::new()
        .route(
            "/",
            get(handlers::list_building_types).post(handlers::create_building_type),
        )
        .route(
            "/{id}",
            get(handlers::get_building_type)
                .put(handlers::update_building_type)
                .delete(handlers::delete_building_type),
        );

    let space_types_router = Router::new()
        .route(
            "/",
            get(handlers::list_space_types).post(handlers::create_space_type),
        )
        .route(
            "/{id}",
            get(handlers::get_space_type)
                .put(handlers::update_space_type)
                .delete(handlers::delete_space_type),
        );

    let sites_router = Router::new()
        .route("/", get(handlers::list_sites).post(handlers::create_site))
        .route(
            "/{id}",
            get(handlers::get_site)
                .put(handlers::update_site)
                .delete(handlers::delete_site),
        );

    let buildings_router = Router::new()
        .route(
            "/",
            get(handlers::list_buildings).post(handlers::create_building),
        )
        .route(
            "/{id}",
            get(handlers::get_building)
                .put(handlers::update_building)
                .delete(handlers::delete_building),
        );

    let floors_router = Router::new()
        .route("/", get(handlers::list_floors).post(handlers::create_floor))
        .route(
            "/{id}",
            get(handlers::get_floor)
                .put(handlers::update_floor)
                .delete(handlers::delete_floor),
        );

    let spaces_router = Router::new()
        .route("/", get(handlers::list_spaces).post(handlers::create_space))
        .route(
            "/{id}",
            get(handlers::get_space)
                .put(handlers::update_space)
                .delete(handlers::delete_space),
        );

    Router::new()
        .nest("/site-types", site_types_router)
        .nest("/building-types", building_types_router)
        .nest("/space-types", space_types_router)
        .nest("/sites", sites_router)
        .nest("/buildings", buildings_router)
        .nest("/floors", floors_router)
        .nest("/spaces", spaces_router)
}
