pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

/// Trips router — RF-USO programação e uso de veículos.
pub fn router() -> Router<AppState> {
    Router::new()
        // Listagem e criação
        .route("/", get(handlers::list_trips).post(handlers::create_trip))
        // Operações por viagem
        .route("/{id}", get(handlers::get_trip))
        .route("/{id}/review", axum::routing::put(handlers::review_trip))
        .route("/{id}/checkin", axum::routing::put(handlers::checkin))
        .route("/{id}/checkout", axum::routing::put(handlers::checkout))
        .route("/{id}/cancel", axum::routing::put(handlers::cancel_trip))
}
