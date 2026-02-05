pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Collection route
        .route("/requisitions", get(handlers::list_requisitions))
        // Specific requisition routes
        .route("/requisitions/{id}", get(handlers::get_requisition))
        .route("/requisitions/{id}/approve", post(handlers::approve_requisition))
        .route("/requisitions/{id}/reject", post(handlers::reject_requisition))
        .route("/requisitions/{id}/cancel", post(handlers::cancel_requisition))
        // Audit/History routes
        .route("/requisitions/{id}/history", get(handlers::get_requisition_history))
        .route("/requisitions/{id}/rollback-points", get(handlers::get_rollback_points))
        .route("/requisitions/{id}/rollback", post(handlers::rollback_requisition))
        // Item routes
        .route("/requisitions/{id}/items", get(handlers::get_requisition_items))
        .route(
            "/requisitions/{req_id}/items/{item_id}/delete",
            post(handlers::delete_requisition_item),
        )
        .route(
            "/requisitions/{req_id}/items/{item_id}/restore",
            post(handlers::restore_requisition_item),
        )
}
