//! Auth Feature Module
//!
//! Este módulo encapsula toda a funcionalidade de autenticação:
//! - Login/Logout (JWT e Session-based)
//! - Registro de usuários
//! - Refresh de tokens
//! - Reset de senha
//! - Session management (HttpOnly cookies)
//!
//! # Arquitetura
//!
//! ```text
//! api/auth/
//! ├── mod.rs              # Router + re-exports (este arquivo)
//! ├── handlers.rs         # Handlers HTTP (JWT-based)
//! ├── session_handlers.rs # Handlers HTTP (Cookie-based)
//! └── contracts.rs        # DTOs (Request/Response)
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
pub mod session_handlers;

use axum::{
    routing::{delete, get, post},
    Router,
};

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
/// # Rotas JWT (Token-Based)
///
/// | Método | Path              | Handler           | Descrição                    |
/// |--------|-------------------|-------------------|------------------------------|
/// | POST   | /login            | login             | Autenticar usuário (JWT)     |
/// | POST   | /register         | register          | Registrar novo usuário       |
/// | POST   | /refresh-token    | refresh_token     | Renovar access token         |
/// | POST   | /logout           | logout            | Revogar refresh token        |
/// | POST   | /forgot-password  | forgot_password   | Solicitar reset de senha     |
/// | POST   | /reset-password   | reset_password    | Redefinir senha com token    |
///
/// # Rotas Session (Cookie-Based)
///
/// | Método | Path                  | Handler           | Descrição                    |
/// |--------|-----------------------|-------------------|------------------------------|
/// | POST   | /session/login        | session_login     | Login com cookie HttpOnly    |
/// | POST   | /session/logout       | session_logout    | Logout (limpa cookies)       |
/// | POST   | /session/logout-all   | session_logout_all| Logout de todas as sessões   |
/// | GET    | /session/me           | session_info      | Info da sessão atual         |
/// | GET    | /auth/me              | session_info      | Alias de /session/me         |
/// | GET    | /session/list         | list_sessions     | Lista sessões do usuário     |
/// | DELETE | /session/{id}         | revoke_session    | Revoga sessão específica     |
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
    let jwt_routes = Router::new()
        // Autenticação JWT (Token-Based)
        .route("/login", post(handlers::login))
        .route("/register", post(handlers::register))
        .route("/refresh-token", post(handlers::refresh_token))
        .route("/logout", post(handlers::logout))
        .route("/forgot-password", post(handlers::forgot_password))
        .route("/reset-password", post(handlers::reset_password));

    let session_routes = Router::new()
        // Autenticação Session (Cookie-Based)
        .route("/session/login", post(session_handlers::session_login))
        .route("/session/logout", post(session_handlers::session_logout))
        .route("/session/logout-all", post(session_handlers::session_logout_all))
        .route("/session/me", get(session_handlers::session_info))
        .route("/auth/me", get(session_handlers::session_info))
        .route("/session/list", get(session_handlers::list_sessions))
        .route("/session/{session_id}", delete(session_handlers::revoke_session));

    jwt_routes
        .merge(session_routes)
        .layer(login_rate_limiter())
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
