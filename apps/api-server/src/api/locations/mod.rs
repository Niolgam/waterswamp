//! Location Management Feature Module
//!
//! Este módulo encapsula funcionalidades de gerenciamento de localizações:
//! - Estados (states)
//! - Cidades (cities)
//! - Tipos de sites (site_types)
//!
//! # Arquitetura
//!
//! ```text
//! api/locations/
//! ├── mod.rs        # Router + re-exports (este arquivo)
//! ├── handlers.rs   # Handlers HTTP
//! └── contracts.rs  # DTOs (Request/Response)
//! ```
//!
//! # Autenticação e Autorização
//!
//! Todas as rotas deste módulo requerem:
//! - Autenticação via JWT
//! - Autorização via RBAC (role: admin)

pub mod contracts;
pub mod handlers;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::infra::state::AppState;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use contracts::{CityResponse, CityWithStateResponse, SiteTypeResponse, StateResponse};

// =============================================================================
// ROUTER
// =============================================================================

/// Cria o router de gerenciamento de localizações.
///
/// # Rotas - States
///
/// | Método | Path                           | Handler       | Descrição                 |
/// |--------|--------------------------------|---------------|---------------------------|
/// | GET    | /admin/locations/states        | list_states   | Listar todos os estados   |
/// | GET    | /admin/locations/states/:id    | get_state     | Obter estado por ID       |
/// | POST   | /admin/locations/states        | create_state  | Criar novo estado         |
/// | PUT    | /admin/locations/states/:id    | update_state  | Atualizar estado          |
/// | DELETE | /admin/locations/states/:id    | delete_state  | Deletar estado            |
///
/// # Rotas - Cities
///
/// | Método | Path                           | Handler       | Descrição                 |
/// |--------|--------------------------------|---------------|---------------------------|
/// | GET    | /admin/locations/cities        | list_cities   | Listar todas as cidades   |
/// | GET    | /admin/locations/cities/:id    | get_city      | Obter cidade por ID       |
/// | POST   | /admin/locations/cities        | create_city   | Criar nova cidade         |
/// | PUT    | /admin/locations/cities/:id    | update_city   | Atualizar cidade          |
/// | DELETE | /admin/locations/cities/:id    | delete_city   | Deletar cidade            |
///
/// # Rotas - Site Types
///
/// | Método | Path                              | Handler           | Descrição                     |
/// |--------|-----------------------------------|-------------------|-------------------------------|
/// | GET    | /admin/locations/site-types       | list_site_types   | Listar tipos de site          |
/// | GET    | /admin/locations/site-types/:id   | get_site_type     | Obter tipo de site por ID     |
/// | POST   | /admin/locations/site-types       | create_site_type  | Criar novo tipo de site       |
/// | PUT    | /admin/locations/site-types/:id   | update_site_type  | Atualizar tipo de site        |
/// | DELETE | /admin/locations/site-types/:id   | delete_site_type  | Deletar tipo de site          |
pub fn router() -> Router<AppState> {
    let states_router = Router::new()
        .route("/", get(handlers::list_states).post(handlers::create_state))
        .route(
            "/:id",
            get(handlers::get_state)
                .put(handlers::update_state)
                .delete(handlers::delete_state),
        );

    let cities_router = Router::new()
        .route("/", get(handlers::list_cities).post(handlers::create_city))
        .route(
            "/:id",
            get(handlers::get_city)
                .put(handlers::update_city)
                .delete(handlers::delete_city),
        );

    let site_types_router = Router::new()
        .route(
            "/",
            get(handlers::list_site_types).post(handlers::create_site_type),
        )
        .route(
            "/:id",
            get(handlers::get_site_type)
                .put(handlers::update_site_type)
                .delete(handlers::delete_site_type),
        );

    Router::new()
        .nest("/states", states_router)
        .nest("/cities", cities_router)
        .nest("/site-types", site_types_router)
}
