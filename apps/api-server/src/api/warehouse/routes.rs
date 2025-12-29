use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::infra::state::AppState;

use super::handlers;

pub fn warehouse_routes() -> Router<AppState> {
    Router::new()
        // Material Groups
        .route(
            "/material-groups",
            get(handlers::list_material_groups).post(handlers::create_material_group),
        )
        .route(
            "/material-groups/:id",
            get(handlers::get_material_group)
                .put(handlers::update_material_group)
                .delete(handlers::delete_material_group),
        )
        // Materials
        .route(
            "/materials",
            get(handlers::list_materials).post(handlers::create_material),
        )
        .route(
            "/materials/:id",
            get(handlers::get_material)
                .put(handlers::update_material)
                .delete(handlers::delete_material),
        )
        // Warehouses
        .route(
            "/warehouses",
            post(handlers::create_warehouse),
        )
        .route(
            "/warehouses/:id",
            get(handlers::get_warehouse)
                .put(handlers::update_warehouse),
        )
        // Stock Movements
        .route("/stock/entry", post(handlers::register_stock_entry))
        .route("/stock/exit", post(handlers::register_stock_exit))
        .route("/stock/adjustment", post(handlers::register_stock_adjustment))
        .route("/stock/:id", get(handlers::get_warehouse_stock))
}
