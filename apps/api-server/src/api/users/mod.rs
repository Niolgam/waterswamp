//! User Self-Service Feature Module
//!
//! Este módulo encapsula funcionalidades de self-service para usuários:
//! - Visualização e atualização de perfil
//! - Alteração de senha
//!
//! # Arquitetura
//!
//! ```text
//! api/users/
//! ├── mod.rs        # Router + re-exports (este arquivo)
//! ├── handlers.rs   # Handlers HTTP
//! └── contracts.rs  # DTOs (Request/Response)
//! ```
//!
//! # Autenticação
//!
//! Todas as rotas deste módulo requerem autenticação.
//! O middleware de autenticação deve ser aplicado na camada de routes.

pub mod contracts;
pub mod handlers;

use axum::{
    routing::{get, put},
    Router,
};

use crate::infra::state::AppState;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use contracts::{
    ChangePasswordRequest, ChangePasswordResponse, ProfileResponse, UpdateProfileRequest,
    UpdateProfileResponse,
};

// =============================================================================
// ROUTER
// =============================================================================

/// Cria o router de self-service de usuários.
///
/// # Rotas
///
/// | Método | Path              | Handler          | Descrição                    |
/// |--------|-------------------|------------------|------------------------------|
/// | GET    | /users/profile    | get_profile      | Obter perfil do usuário      |
/// | PUT    | /users/profile    | update_profile   | Atualizar perfil             |
/// | PUT    | /users/password   | change_password  | Alterar senha                |
///
/// # Autenticação
///
/// Todas as rotas requerem autenticação JWT.
/// O middleware deve ser aplicado externamente.
///
/// # Exemplo
///
/// ```rust,ignore
/// use crate::api::users;
/// use crate::middleware::mw_authenticate;
///
/// let router = users::router()
///     .layer(middleware::from_fn_with_state(state, mw_authenticate));
/// ```
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/profile", get(handlers::get_profile))
        .route("/users/profile", put(handlers::update_profile))
        .route("/users/password", put(handlers::change_password))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }
}
