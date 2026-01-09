//! Location Management Feature Module
//!
//! Este módulo contém os endpoints para gerenciamento de regiões geográficas:
//! - **geo_regions**: Entidades geográficas (Country, State, City)
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
//! └── public/
//!     ├── mod.rs          # Router público
//!     ├── handlers.rs     # Handlers HTTP públicos
//!     └── contracts.rs    # DTOs
//! ```
//!
//! # Autenticação e Autorização
//!
//! As rotas de geo_regions requerem:
//! - Autenticação via JWT
//! - Autorização via RBAC (role: admin)

pub mod geo_regions;
pub mod public;

use axum::Router;
use crate::infra::state::AppState;

// =============================================================================
// RE-EXPORTS
// =============================================================================

// Geographic Regions
pub use geo_regions::{
    CityResponse, CityWithStateResponse, CountryResponse, StateResponse, StateWithCountryResponse,
};

// =============================================================================
// ROUTER
// =============================================================================

/// Cria o router principal de gerenciamento de localizações (ADMIN).
///
/// Agrega os routers dos submódulos:
/// - `/countries/*`, `/states/*`, `/cities/*` - Geographic regions
pub fn router() -> Router<AppState> {
    geo_regions::router()
}

/// Cria o router público de localizações (SEM AUTENTICAÇÃO).
///
/// Este router fornece endpoints públicos para visualização do mapa:
/// - `/public/sites` - Listar sites
/// - `/public/buildings` - Obter buildings
/// - `/public/spaces` - Obter spaces
/// - `/public/search` - Buscar localizações
/// - `/public/building-types` - Tipos de building
/// - `/public/space-types` - Tipos de space
pub fn public_router() -> Router<AppState> {
    public::router()
}
