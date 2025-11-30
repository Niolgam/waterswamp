//! Auth Feature Module
//!
//! Este módulo encapsula toda a funcionalidade de autenticação:
//! - Login/Logout
//! - Registro de usuários
//! - Refresh de tokens
//! - Reset de senha
//!
//! # Arquitetura
//!
//! ```text
//! api/auth/
//! ├── mod.rs        # Router + re-exports (este arquivo)
//! ├── handlers.rs   # Handlers HTTP
//! └── contracts.rs  # DTOs (Request/Response)
//! ```
//!
//! # Uso
//!
//! ```rust,ignore
//! use crate::api::auth;
//!
//! // Obter o router
//! let auth_router = auth::router();
//!
//! // Usar DTOs
//! use crate::api::auth::contracts::{LoginRequest, LoginResponse};
//! ```

pub mod contracts;
pub mod handlers;

use axum::{routing::post, Router};

use crate::{infra::state::AppState, middleware::login_rate_limiter};

// =============================================================================
// RE-EXPORTS
// =============================================================================

// Re-export dos contratos para uso externo
pub use contracts::{
    ForgotPasswordRequest, ForgotPasswordResponse, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, MfaRequiredResponse, RefreshTokenRequest, RefreshTokenResponse,
    RegisterRequest, RegisterResponse, ResetPasswordRequest, ResetPasswordResponse,
};

// =============================================================================
// ROUTER
// =============================================================================

/// Cria o router de autenticação com todas as rotas públicas.
///
/// # Rotas
///
/// | Método | Path              | Handler           | Descrição                    |
/// |--------|-------------------|-------------------|------------------------------|
/// | POST   | /login            | login             | Autenticar usuário           |
/// | POST   | /register         | register          | Registrar novo usuário       |
/// | POST   | /refresh-token    | refresh_token     | Renovar access token         |
/// | POST   | /logout           | logout            | Revogar refresh token        |
/// | POST   | /forgot-password  | forgot_password   | Solicitar reset de senha     |
/// | POST   | /reset-password   | reset_password    | Redefinir senha com token    |
///
/// # Exemplo
///
/// ```rust,ignore
/// use crate::api::auth;
/// use crate::infra::state::AppState;
///
/// let app_state: AppState = /* ... */;
/// let router = auth::router().with_state(app_state);
/// ```
pub fn router() -> Router<AppState> {
    let auth_routes = Router::new()
        // Autenticação Básica
        .route("/login", post(handlers::login))
        .route("/register", post(handlers::register))
        .route("/refresh-token", post(handlers::refresh_token))
        .route("/logout", post(handlers::logout))
        .route("/forgot-password", post(handlers::forgot_password))
        .route("/reset-password", post(handlers::reset_password));

    auth_routes.layer(login_rate_limiter())
}

/// Cria o router para rotas de autenticação protegidas (requerem token de acesso).
///
/// Inclui: Status de Verificação, Configuração e Gestão de MFA.
pub fn protected_router() -> Router<AppState> {
    Router::new()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        // Apenas verifica que o router pode ser criado sem panic
        let _router: Router<AppState> = router();
    }
}
