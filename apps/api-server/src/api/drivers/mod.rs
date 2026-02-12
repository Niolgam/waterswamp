pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::get,
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_drivers).post(handlers::create_driver))
        .route("/{id}", get(handlers::get_driver)
            .put(handlers::update_driver)
            .delete(handlers::delete_driver))
}
