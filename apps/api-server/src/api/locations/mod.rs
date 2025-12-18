//! Location Management Feature Module
//!
//! Este módulo foi refatorado em três submódulos organizados por domínio:
//! - **geo_regions**: Entidades geográficas (Country, State, City)
//! - **facilities**: Instalações físicas e seus tipos (Site, Building, Floor, Space + Types)
//! - **departments**: Categorias departamentais (DepartmentCategory)
//!
//! # Arquitetura
//!
//! ```text
//! api/locations/
//! ├── mod.rs              # Router principal (este arquivo)
//! ├── geo_regions/
//! │   ├── mod.rs          # Router de regiões geográficas
//! │   ├── handlers.rs     # Handlers HTTP
//! │   └── contracts.rs    # DTOs
//! ├── facilities/
//! │   ├── mod.rs          # Router de instalações
//! │   ├── handlers.rs     # Handlers HTTP
//! │   └── contracts.rs    # DTOs
//! └── departments/
//!     ├── mod.rs          # Router de departamentos
//!     ├── handlers.rs     # Handlers HTTP
//!     └── contracts.rs    # DTOs
//! ```
//!
//! # Autenticação e Autorização
//!
//! Todas as rotas deste módulo requerem:
//! - Autenticação via JWT
//! - Autorização via RBAC (role: admin)

pub mod departments;
pub mod facilities;
pub mod geo_regions;

use axum::Router;
use crate::infra::state::AppState;

// =============================================================================
// RE-EXPORTS
// =============================================================================

// Geographic Regions
pub use geo_regions::{
    CityResponse, CityWithStateResponse, CountryResponse, StateResponse, StateWithCountryResponse,
};

// Facilities
pub use facilities::{
    BuildingResponse, BuildingTypeResponse, FloorResponse, SiteResponse, SiteTypeResponse,
    SpaceResponse, SpaceTypeResponse,
};

// Departments
pub use departments::DepartmentCategoryResponse;

// =============================================================================
// ROUTER
// =============================================================================

/// Cria o router principal de gerenciamento de localizações.
///
/// Agrega os routers dos três submódulos:
/// - `/geo_regions/*` - Countries, States, Cities
/// - `/facilities/*` - Sites, Buildings, Floors, Spaces + Types
/// - `/departments/*` - Department Categories
pub fn router() -> Router<AppState> {
    geo_regions::router()
        .merge(facilities::router())
        .nest("/department-categories", departments::router())
}
