pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/policies", get(handlers::list_policies))
        .route("/policies", post(handlers::add_policy))
        .route("/policies", delete(handlers::remove_policy))
}
