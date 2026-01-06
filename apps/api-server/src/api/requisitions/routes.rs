use axum::{
    routing::{get, post},
    Router,
};

use crate::infra::state::AppState;

use super::handlers;

pub fn requisition_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handlers::list_requisitions).post(handlers::create_requisition),
        )
        .route(
            "/{id}",
            get(handlers::get_requisition).delete(handlers::cancel_requisition),
        )
        .route("/{id}/approve", post(handlers::approve_requisition))
        .route("/{id}/reject", post(handlers::reject_requisition))
        .route("/{id}/fulfill", post(handlers::fulfill_requisition))
}
