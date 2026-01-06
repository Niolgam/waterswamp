//! Public Locations Module
//!
//! Provides PUBLIC (unauthenticated) endpoints for the frontend map display.
//! These routes are accessible without authentication.

pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{routing::get, Router};

pub use contracts::*;
pub use handlers::*;

/// Creates the public locations router
///
/// All routes under this router are PUBLIC (no authentication required)
pub fn router() -> Router<AppState> {
    Router::new()
        // Sites
        .route("/sites", get(handlers::list_public_sites))
        .route("/sites/{id}", get(handlers::get_public_site))
        .route(
            "/sites/{siteId}/buildings",
            get(handlers::list_site_buildings),
        )
        .route("/sites/{siteId}/spaces", get(handlers::list_site_spaces))
        // Buildings
        .route("/buildings/{id}", get(handlers::get_public_building))
        // Spaces
        .route("/spaces/{id}", get(handlers::get_public_space))
        // Search
        .route("/search", get(handlers::search_locations))
        // Types (for filters)
        .route("/building-types", get(handlers::list_public_building_types))
        .route("/space-types", get(handlers::list_public_space_types))
}
