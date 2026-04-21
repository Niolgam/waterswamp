pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        // Listagem global + abertura por veículo
        .route("/", get(handlers::list_orders))
        .route("/vehicles/{vehicle_id}", axum::routing::post(handlers::open_order))
        .route("/vehicles/{vehicle_id}/cost", get(handlers::get_cost_summary))
        // Operações por OS
        .route("/{id}", get(handlers::get_order))
        .route("/{id}/advance", axum::routing::put(handlers::advance_order))
        .route("/{id}/items", axum::routing::post(handlers::add_item).get(handlers::list_items))
}
