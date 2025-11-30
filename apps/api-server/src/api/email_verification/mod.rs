pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub use contracts::*;
// Exporta helper para uso no registro de usuários (api/auth)
pub use handlers::create_verification_token;

/// Cria o router para rotas de verificação de email.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/verify-email", post(handlers::verify_email))
        .route("/resend-verification", post(handlers::resend_verification))
}

/// Cria o router para rotas protegidas de verificação (status).
pub fn protected_router() -> Router<AppState> {
    Router::new().route("/verification-status", get(handlers::verification_status))
}
