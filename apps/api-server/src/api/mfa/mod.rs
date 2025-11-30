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
    Router::new().route("/auth/mfa/verify", post(handlers::verify))
}

/// Router para rotas protegidas (gerenciamento)
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/auth/mfa/setup", post(handlers::setup))
        .route("/auth/mfa/verify-setup", post(handlers::verify_setup))
        .route("/auth/mfa/disable", post(handlers::disable))
        .route(
            "/auth/mfa/regenerate-backup-codes",
            post(handlers::regenerate_backup_codes),
        )
        .route("/auth/mfa/status", get(handlers::status))
}
