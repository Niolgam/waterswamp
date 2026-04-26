pub mod audit;
pub mod policies;
pub mod requisitions;
pub mod transfers;
pub mod users;

pub mod invoices;
pub mod inventory_sessions;
pub mod warehouses;

use crate::{
    api::{
        budget_classifications, catalog, drivers, fleet, fuelings, geo_regions, maintenance,
        organizational, reports, suppliers, trips, vehicle_fines,
    },
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
        .nest("/trips", trips::router())
        .nest("/maintenance", maintenance::router())
        .nest("/suppliers", suppliers::router())
        .nest("/drivers", drivers::router())
        .nest("/fuelings", fuelings::router())
        .nest("/vehicle-fines", vehicle_fines::router())
        .nest("/reports", reports::router())
        .nest("/invoices", invoices::router())
        .nest("/warehouses", warehouses::router())
        .nest("/warehouses", inventory_sessions::warehouse_router())
        .nest("/inventory-sessions", inventory_sessions::session_router())
        .nest("/disposal-requests", warehouses::disposal_requests_router())
        .merge(transfers::router())
        .layer(admin_rate_limiter())
}
