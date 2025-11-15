use crate::handlers::admin_handler;
use crate::{rate_limit::admin_rate_limiter, state::AppState};
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/policies", post(admin_handler::add_policy))
        .route("/api/admin/policies", delete(admin_handler::remove_policy))
        .route("/api/admin/users", get(admin_handler::list_users))
        .route("/api/admin/users", post(admin_handler::create_user))
        .route("/api/admin/users/{id}", get(admin_handler::get_user))
        .route("/api/admin/users/{id}", put(admin_handler::update_user))
        .route("/api/admin/users/{id}", delete(admin_handler::delete_user))
        .layer(admin_rate_limiter())
}
