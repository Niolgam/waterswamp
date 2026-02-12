pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::get,
    Router,
};

pub fn router() -> Router<AppState> {
    let fine_types_router = Router::new()
        .route("/", get(handlers::list_fine_types).post(handlers::create_fine_type))
        .route("/{id}", get(handlers::get_fine_type)
            .put(handlers::update_fine_type)
            .delete(handlers::delete_fine_type));

    let fines_router = Router::new()
        .route("/", get(handlers::list_fines).post(handlers::create_fine))
        .route("/{id}", get(handlers::get_fine)
            .put(handlers::update_fine)
            .delete(handlers::delete_fine))
        .route("/{id}/restore", axum::routing::put(handlers::restore_fine));

    Router::new()
        .nest("/fine-types", fine_types_router)
        .nest("/fines", fines_router)
}
