pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_invoices).post(handlers::create_invoice))
        .route(
            "/{id}",
            get(handlers::get_invoice)
                .put(handlers::update_invoice)
                .delete(handlers::delete_invoice),
        )
        .route("/{id}/items", get(handlers::get_invoice_items))
        .route("/{id}/start-checking", axum::routing::post(handlers::start_checking))
        .route("/{id}/finish-checking", axum::routing::post(handlers::finish_checking))
        .route("/{id}/post", axum::routing::post(handlers::post_invoice))
        .route("/{id}/reject", axum::routing::post(handlers::reject_invoice))
        .route("/{id}/cancel", axum::routing::post(handlers::cancel_invoice))
        .route(
            "/{id}/adjustments",
            get(handlers::list_invoice_adjustments)
                .post(handlers::create_invoice_adjustment),
        )
}
