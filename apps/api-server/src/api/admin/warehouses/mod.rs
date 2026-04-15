pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_warehouses).post(handlers::create_warehouse))
        .route("/{id}", get(handlers::get_warehouse).put(handlers::update_warehouse).delete(handlers::delete_warehouse))
        .route("/{id}/stocks", get(handlers::list_warehouse_stocks))
        // Stock management routes (stock id in path)
        .route("/stocks/{stock_id}", get(handlers::get_stock).patch(handlers::update_stock_params))
        .route("/stocks/{stock_id}/block", post(handlers::block_stock))
        .route("/stocks/{stock_id}/unblock", post(handlers::unblock_stock))
        // Stock movement routes (RF-009, RF-011, RF-016, RF-017)
        .route("/{id}/movements", get(handlers::list_stock_movements))
        .route("/{id}/entries", post(handlers::create_standalone_entry))
        .route("/{id}/returns", post(handlers::create_return_entry))
        .route("/{id}/disposals", post(handlers::create_disposal_exit))
        .route("/{id}/manual-exits", post(handlers::create_manual_exit))
}
