use crate::{
    handlers::{auth_handler, email_verification_handler, mfa_handler},
    middleware::login_rate_limiter,
    state::AppState,
};
use axum::routing::get;
use axum::{routing::post, Router};

pub fn router(_state: AppState) -> Router<AppState> {
    // Auth routes (existing + new)
    let auth_routes = Router::new()
        // Existing auth routes
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
        // NEW: Email Verification routes
        .route(
            "/verify-email",
            post(email_verification_handler::handler_verify_email),
        )
        .route(
            "/resend-verification",
            post(email_verification_handler::handler_resend_verification),
        )
        .route("/auth/mfa/verify", post(mfa_handler::handler_mfa_verify));

    // Apply rate limiting to auth routes
    auth_routes.layer(login_rate_limiter())
}

/// Protected auth routes (require authentication)
pub fn protected_auth_router() -> Router<AppState> {
    Router::new()
        // Email verification status (authenticated)
        .route(
            "/verification-status",
            get(email_verification_handler::handler_verification_status),
        )
        // MFA routes (authenticated)
        .route("/auth/mfa/setup", post(mfa_handler::handler_mfa_setup))
        .route(
            "/auth/mfa/verify-setup",
            post(mfa_handler::handler_mfa_verify_setup),
        )
        .route("/auth/mfa/disable", post(mfa_handler::handler_mfa_disable))
        .route(
            "/auth/mfa/regenerate-backup-codes",
            post(mfa_handler::handler_mfa_regenerate_backup_codes),
        )
        .route("/auth/mfa/status", get(mfa_handler::handler_mfa_status))
}
