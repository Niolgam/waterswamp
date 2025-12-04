pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub use contracts::*;

/// Router para rota de verificação (parte do login)
pub fn router() -> Router<AppState> {
    // CORREÇÃO: handlers::verify -> handlers::verify_login
    Router::new().route("/auth/mfa/verify", post(handlers::verify_login))
}

/// Router para rotas protegidas (gerenciamento)
pub fn protected_router() -> Router<AppState> {
    Router::new()
        // CORREÇÃO: handlers::setup -> handlers::initiate_setup
        .route("/auth/mfa/setup", post(handlers::initiate_setup))
        .route("/auth/mfa/verify-setup", post(handlers::verify_setup))
        // CORREÇÃO: handlers::disable -> handlers::disable_mfa
        .route("/auth/mfa/disable", post(handlers::disable_mfa))
        .route(
            "/auth/mfa/regenerate-backup-codes",
            post(handlers::regenerate_backup_codes),
        )
        // CORREÇÃO: handlers::status -> handlers::get_status
        .route("/auth/mfa/status", get(handlers::get_status))
}
