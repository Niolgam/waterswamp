pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_suppliers).post(handlers::create_supplier))
        .route(
            "/{id}",
            get(handlers::get_supplier)
                .put(handlers::update_supplier)
                .delete(handlers::delete_supplier),
        )
}
