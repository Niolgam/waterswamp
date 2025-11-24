use crate::handlers::{admin_handler, audit_handler};
use crate::{middleware::rate_limit::admin_rate_limiter, state::AppState};
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Policy management
        .route("/api/admin/policies", post(admin_handler::add_policy))
        .route("/api/admin/policies", delete(admin_handler::remove_policy))
        // User management
        .route("/api/admin/users", get(admin_handler::list_users))
        .route("/api/admin/users", post(admin_handler::create_user))
        .route("/api/admin/users/{id}", get(admin_handler::get_user))
        .route("/api/admin/users/{id}", put(admin_handler::update_user))
        .route("/api/admin/users/{id}", delete(admin_handler::delete_user))
        // Audit logs
        .route("/api/admin/audit-logs", get(audit_handler::list_audit_logs))
        .route(
            "/api/admin/audit-logs/stats",
            get(audit_handler::get_audit_stats),
        )
        .route(
            "/api/admin/audit-logs/failed-logins",
            get(audit_handler::get_failed_logins),
        )
        .route(
            "/api/admin/audit-logs/suspicious-ips",
            get(audit_handler::get_suspicious_ips),
        )
        .route(
            "/api/admin/audit-logs/user/{user_id}",
            get(audit_handler::get_user_audit_logs),
        )
        .route(
            "/api/admin/audit-logs/{id}",
            get(audit_handler::get_audit_log),
        )
        .route(
            "/api/admin/audit-logs/cleanup",
            post(audit_handler::cleanup_old_logs),
        )
        .layer(admin_rate_limiter())
}
