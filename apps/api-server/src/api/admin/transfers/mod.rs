pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Collection: list + initiate from a specific source warehouse
        .route("/warehouses/{warehouse_id}/transfers", post(handlers::initiate_transfer))
        // Transfer lifecycle
        .route("/transfers", get(handlers::list_transfers))
        .route("/transfers/{id}", get(handlers::get_transfer))
        .route("/transfers/{id}/confirm", post(handlers::confirm_transfer))
        .route("/transfers/{id}/reject", post(handlers::reject_transfer))
        .route("/transfers/{id}/cancel", post(handlers::cancel_transfer))
}
