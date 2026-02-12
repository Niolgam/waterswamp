pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::get,
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_fuelings).post(handlers::create_fueling))
        .route("/{id}", get(handlers::get_fueling)
            .put(handlers::update_fueling)
            .delete(handlers::delete_fueling))
}
