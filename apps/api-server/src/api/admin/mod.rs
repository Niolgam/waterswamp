pub mod audit;
pub mod policies;
pub mod requisitions;
pub mod users;

use crate::{
    api::{budget_classifications, catalog, fleet, geo_regions, organizational, suppliers},
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
        .merge(requisitions::router())
        .nest("/geo_regions", geo_regions::router())
        .nest("/budget-classifications", budget_classifications::router())
        .nest("/catalog", catalog::router())
        .nest("/organizational", organizational::router())
        .nest("/fleet", fleet::router())
        .nest("/suppliers", suppliers::router())
        .layer(admin_rate_limiter())
}
