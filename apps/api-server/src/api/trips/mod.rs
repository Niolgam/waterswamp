pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

/// Trips router — RF-USO programação e uso de veículos.
///
/// FSM: SOLICITADA → APROVADA → ALOCADA → EM_CURSO → AGUARDANDO_PC → CONCLUIDA
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_trips).post(handlers::create_trip))
        .route("/{id}", get(handlers::get_trip))
        .route("/{id}/review",    axum::routing::put(handlers::review_trip))
        .route("/{id}/allocate",  axum::routing::put(handlers::allocate_trip))
        .route("/{id}/checkout",  axum::routing::put(handlers::checkout))
        .route("/{id}/checkin",   axum::routing::put(handlers::checkin))
        .route("/{id}/finalize",  axum::routing::put(handlers::finalize_trip))
        .route("/{id}/conflict",  axum::routing::put(handlers::set_conflict))
        .route("/{id}/cancel",    axum::routing::put(handlers::cancel_trip))
}
