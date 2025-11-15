use crate::{handlers::protected_handler, state::AppState};
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/users/profile",
            get(protected_handler::handler_user_profile),
        )
        .route(
            "/admin/dashboard",
            get(protected_handler::handler_admin_dashboard),
        )
}
