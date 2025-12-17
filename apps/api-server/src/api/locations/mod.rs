//! Location Management Feature Module
//!
//! Este módulo encapsula funcionalidades de gerenciamento de localizações:
//! - Estados (states)
//! - Cidades (cities)
//! - Tipos de sites (site_types)
//! - Tipos de edifícios (building_types)
//! - Tipos de espaços (space_types)
//! - Categorias de departamentos (department_categories)
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

pub use contracts::{
    BuildingResponse, BuildingTypeResponse, CityResponse, CityWithStateResponse,
    DepartmentCategoryResponse, SiteResponse, SiteTypeResponse, SpaceTypeResponse,
    StateResponse,
};

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
///
/// # Rotas - Building Types
///
/// | Método | Path                                  | Handler               | Descrição                         |
/// |--------|---------------------------------------|-----------------------|-----------------------------------|
/// | GET    | /admin/locations/building-types       | list_building_types   | Listar tipos de edifício          |
/// | GET    | /admin/locations/building-types/:id   | get_building_type     | Obter tipo de edifício por ID     |
/// | POST   | /admin/locations/building-types       | create_building_type  | Criar novo tipo de edifício       |
/// | PUT    | /admin/locations/building-types/:id   | update_building_type  | Atualizar tipo de edifício        |
/// | DELETE | /admin/locations/building-types/:id   | delete_building_type  | Deletar tipo de edifício          |
///
/// # Rotas - Space Types
///
/// | Método | Path                              | Handler           | Descrição                     |
/// |--------|-----------------------------------|-------------------|-------------------------------|
/// | GET    | /admin/locations/space-types      | list_space_types  | Listar tipos de espaço        |
/// | GET    | /admin/locations/space-types/:id  | get_space_type    | Obter tipo de espaço por ID   |
/// | POST   | /admin/locations/space-types      | create_space_type | Criar novo tipo de espaço     |
/// | PUT    | /admin/locations/space-types/:id  | update_space_type | Atualizar tipo de espaço      |
/// | DELETE | /admin/locations/space-types/:id  | delete_space_type | Deletar tipo de espaço        |
///
/// # Rotas - Department Categories
///
/// | Método | Path                                         | Handler                      | Descrição                              |
/// |--------|----------------------------------------------|------------------------------|----------------------------------------|
/// | GET    | /admin/locations/department-categories       | list_department_categories   | Listar categorias de departamento      |
/// | GET    | /admin/locations/department-categories/:id   | get_department_category      | Obter categoria de departamento por ID |
/// | POST   | /admin/locations/department-categories       | create_department_category   | Criar nova categoria de departamento   |
/// | PUT    | /admin/locations/department-categories/:id   | update_department_category   | Atualizar categoria de departamento    |
/// | DELETE | /admin/locations/department-categories/:id   | delete_department_category   | Deletar categoria de departamento      |
///
/// # Rotas - Sites (Phase 3A)
///
/// | Método | Path                         | Handler      | Descrição                                    |
/// |--------|------------------------------|--------------|----------------------------------------------|
/// | GET    | /admin/locations/sites       | list_sites   | Listar sites (com filtros city_id, site_type_id) |
/// | GET    | /admin/locations/sites/:id   | get_site     | Obter site por ID (com dados relacionados)   |
/// | POST   | /admin/locations/sites       | create_site  | Criar novo site                              |
/// | PUT    | /admin/locations/sites/:id   | update_site  | Atualizar site                               |
/// | DELETE | /admin/locations/sites/:id   | delete_site  | Deletar site                                 |
///
/// # Rotas - Buildings (Phase 3B)
///
/// | Método | Path                            | Handler         | Descrição                                               |
/// |--------|----------------------------------|-----------------|--------------------------------------------------------|
/// | GET    | /admin/locations/buildings       | list_buildings  | Listar edifícios (com filtros site_id, building_type_id) |
/// | GET    | /admin/locations/buildings/:id   | get_building    | Obter edifício por ID (com dados relacionados)          |
/// | POST   | /admin/locations/buildings       | create_building | Criar novo edifício                                     |
/// | PUT    | /admin/locations/buildings/:id   | update_building | Atualizar edifício                                      |
/// | DELETE | /admin/locations/buildings/:id   | delete_building | Deletar edifício                                        |
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

    let building_types_router = Router::new()
        .route(
            "/",
            get(handlers::list_building_types).post(handlers::create_building_type),
        )
        .route(
            "/:id",
            get(handlers::get_building_type)
                .put(handlers::update_building_type)
                .delete(handlers::delete_building_type),
        );

    let space_types_router = Router::new()
        .route(
            "/",
            get(handlers::list_space_types).post(handlers::create_space_type),
        )
        .route(
            "/:id",
            get(handlers::get_space_type)
                .put(handlers::update_space_type)
                .delete(handlers::delete_space_type),
        );

    let department_categories_router = Router::new()
        .route(
            "/",
            get(handlers::list_department_categories).post(handlers::create_department_category),
        )
        .route(
            "/:id",
            get(handlers::get_department_category)
                .put(handlers::update_department_category)
                .delete(handlers::delete_department_category),
        );

    let sites_router = Router::new()
        .route("/", get(handlers::list_sites).post(handlers::create_site))
        .route(
            "/:id",
            get(handlers::get_site)
                .put(handlers::update_site)
                .delete(handlers::delete_site),
        );

    let buildings_router = Router::new()
        .route(
            "/",
            get(handlers::list_buildings).post(handlers::create_building),
        )
        .route(
            "/:id",
            get(handlers::get_building)
                .put(handlers::update_building)
                .delete(handlers::delete_building),
        );

    Router::new()
        .nest("/states", states_router)
        .nest("/cities", cities_router)
        .nest("/site-types", site_types_router)
        .nest("/building-types", building_types_router)
        .nest("/space-types", space_types_router)
        .nest("/department-categories", department_categories_router)
        .nest("/sites", sites_router)
        .nest("/buildings", buildings_router)
}
