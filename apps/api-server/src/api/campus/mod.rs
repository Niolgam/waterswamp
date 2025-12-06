pub mod contracts;
pub mod handlers;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::infra::state::AppState;

/// Configures campus module routes
pub fn routes() -> Router<AppState> {
    Router::new()
        // Create new campus
        .route("/", post(handlers::create_campus))
        // List all campuses (with optional pagination)
        .route("/", get(handlers::list_campuses))
        // Find campus by ID
        .route("/:id", get(handlers::get_campus))
        // Update campus
        .route("/:id", put(handlers::update_campus))
        // Delete campus
        .route("/:id", delete(handlers::delete_campus))
        // Find campus by acronym
        .route("/acronym/:acronym", get(handlers::get_campus_by_acronym))
        // Find campus by name
        .route("/name/:name", get(handlers::get_campus_by_name))
        // Find campuses by city
        .route("/search/city", get(handlers::search_by_city))
}
