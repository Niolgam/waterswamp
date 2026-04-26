pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Routes nested under /warehouses/:id
pub fn warehouse_batch_router() -> Router<AppState> {
    Router::new()
        .route("/batch-stocks/{catalog_item_id}", get(handlers::list_batches))
        .route("/fefo-exit", post(handlers::fefo_exit))
}

/// Routes at the root admin level
pub fn batch_router() -> Router<AppState> {
    Router::new()
        .route("/batch-quality-occurrences", get(handlers::list_occurrences).post(handlers::create_occurrence))
        .route("/batch-quality-occurrences/{id}", get(handlers::get_occurrence))
        .route("/batch-quality-occurrences/{id}/resolve", post(handlers::resolve_occurrence))
        .route("/batch-quality-occurrences/{id}/close", post(handlers::close_occurrence))
        .route("/batch-stocks/near-expiry", get(handlers::list_near_expiry))
}
