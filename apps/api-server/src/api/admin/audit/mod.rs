pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Static specific routes first
        .route("/audit-logs/stats", get(handlers::get_stats))
        .route(
            "/audit-logs/failed-logins",
            get(handlers::get_failed_logins),
        )
        .route(
            "/audit-logs/suspicious-ips",
            get(handlers::get_suspicious_ips),
        )
        .route("/audit-logs/cleanup", post(handlers::cleanup_logs))
        // Dynamic routes next
        .route("/audit-logs/user/{user_id}", get(handlers::get_user_logs))
        .route("/audit-logs/{id}", get(handlers::get_log))
        // Root collection route last
        .route("/audit-logs", get(handlers::list_logs))
}
