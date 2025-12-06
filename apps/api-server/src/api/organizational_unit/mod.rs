pub mod contracts;
pub mod handlers;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::infra::state::AppState;

/// Configures organizational_unit module routes
pub fn routes() -> Router<AppState> {
    Router::new()
        // Create new unit
        .route("/", post(handlers::create_unit))
        // List all units
        .route("/", get(handlers::list_units))
        // Find unit by ID
        .route("/:id", get(handlers::get_unit))
        // Update unit
        .route("/:id", put(handlers::update_unit))
        // Delete unit
        .route("/:id", delete(handlers::delete_unit))
        // List root units
        .route("/root", get(handlers::list_root_units))
        // Find by acronym
        .route("/acronym", get(handlers::get_unit_by_acronym))
        // List by parent
        .route("/by-parent", get(handlers::list_by_parent))
        // List by category
        .route("/by-category", get(handlers::list_by_category))
        // List by campus
        .route("/by-campus", get(handlers::list_by_campus))
}
