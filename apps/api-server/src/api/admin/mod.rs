pub mod audit;
pub mod policies;
pub mod users;

use crate::{
    api::locations,
    infra::state::AppState,
    middleware::rate_limit::admin_rate_limiter, // Certifique-se que existe ou use api_rate_limiter
};
use axum::Router;

/// Cria o router principal de administração que agrega as sub-features.
/// Este router deve ser protegido por autenticação E autorização (RBAC) na camada superior.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(users::router())
        .merge(policies::router())
        .merge(audit::router())
        .nest("/locations", locations::router())
        .layer(admin_rate_limiter())
}
