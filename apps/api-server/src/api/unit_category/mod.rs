pub mod contracts;
pub mod handlers;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::infra::state::AppState;

/// Configures unit_category module routes
pub fn routes() -> Router<AppState> {
    Router::new()
        // Create new category
        .route("/", post(handlers::create_category))
        // List all categories
        .route("/", get(handlers::list_categories))
        // Find category by ID
        .route("/:id", get(handlers::get_category))
        // Update category
        .route("/:id", put(handlers::update_category))
        // Delete category
        .route("/:id", delete(handlers::delete_category))
}
