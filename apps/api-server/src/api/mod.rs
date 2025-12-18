//! Cada feature é autocontida e exporta:
//! - `router()` - Router Axum com todas as rotas da feature
//! - `contracts` - DTOs de request/response
//!

pub mod admin;
pub mod auth;
pub mod email_verification;
pub mod locations;
pub mod mfa;
pub mod users;

use axum::Router;

use crate::infra::state::AppState;

// =============================================================================
// ROUTER PRINCIPAL
// =============================================================================

/// Cria o router principal da API, combinando todas as features.
///
/// # Features Incluídas
///
/// - **auth**: Autenticação (login, registro, tokens, password reset)
/// - **users**: Self-service de usuários (perfil, alteração de senha)
///
/// # Nota
///
/// Este router NÃO inclui:
/// - Middlewares de autenticação/autorização
/// - Rate limiting
/// - Rotas administrativas (ver `routes/admin.rs`)
///
/// Esses são aplicados na camada de routes (`routes/mod.rs`).
///
/// # Exemplo
///
/// ```rust,ignore
/// use crate::api;
/// use crate::infra::state::AppState;
///
/// let state: AppState = /* ... */;
/// let router = api::router(state);
/// ```
pub fn router(state: AppState) -> Router {
    Router::new()
        // Auth routes (públicas)
        .merge(auth::router())
        // User self-service routes (requer autenticação - aplicada em routes/)
        .merge(users::router())
        .with_state(state)
}

/// Cria apenas o router de rotas públicas (sem autenticação necessária).
///
/// Útil para separar rotas públicas das protegidas na configuração.
pub fn public_router() -> Router<AppState> {
    auth::router()
}

/// Cria apenas o router de rotas que requerem autenticação.
///
/// Útil para aplicar middleware de autenticação seletivamente.
pub fn authenticated_router() -> Router<AppState> {
    users::router()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_router_creation() {
        let _router: Router<AppState> = public_router();
    }

    #[test]
    fn test_authenticated_router_creation() {
        let _router: Router<AppState> = authenticated_router();
    }
}
