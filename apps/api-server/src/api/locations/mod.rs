pub mod public;

use crate::infra::state::AppState;
use axum::Router;

// pub fn router() -> Router<AppState> {}

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
