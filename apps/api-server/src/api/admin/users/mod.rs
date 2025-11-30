pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, put},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/users",
            get(handlers::list_users).post(handlers::create_user),
        )
        .route(
            "/users/{id}",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .route("/users/{id}/ban", put(handlers::ban_user))
        .route("/users/{id}/unban", put(handlers::unban_user))
}
