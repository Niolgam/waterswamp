use crate::{handlers::auth_handler, rate_limit::login_rate_limiter, state::AppState};
use axum::{routing::post, Router};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", post(auth_handler::handler_login))
        .route("/register", post(auth_handler::handler_register))
        .route("/refresh-token", post(auth_handler::handler_refresh_token))
        .route("/logout", post(auth_handler::handler_logout))
        .route(
            "/forgot-password",
            post(auth_handler::handler_forgot_password),
        )
        .route(
            "/reset-password",
            post(auth_handler::handler_reset_password),
        )
        .layer(login_rate_limiter())
}
