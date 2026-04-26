pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Routes nested under /warehouses/:id
pub fn warehouse_router() -> Router<AppState> {
    Router::new()
        .route("/inventory-sessions", get(handlers::list_sessions).post(handlers::create_session))
}

/// Routes nested under /inventory-sessions
pub fn session_router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(handlers::get_session))
        .route("/{id}/start-counting", post(handlers::start_counting))
        .route("/{id}/submit-count", post(handlers::submit_count))
        .route("/{id}/start-reconciliation", post(handlers::start_reconciliation))
        .route("/{id}/reconcile", post(handlers::reconcile))
        .route("/{id}/confirm-govbr-signature", post(handlers::confirm_govbr_signature))
        .route("/{id}/cancel", post(handlers::cancel_session))
}
