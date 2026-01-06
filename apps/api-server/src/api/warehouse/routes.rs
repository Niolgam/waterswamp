use axum::{
    routing::{get, post, put},
    Router,
};

use crate::infra::state::AppState;

use super::{handlers, reports_handlers};

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
        .route("/stock/transfer", post(handlers::transfer_stock))
        .route("/stock/:id", get(handlers::get_warehouse_stock))
        .route("/stock/:id/maintenance", put(handlers::update_stock_maintenance))
        .route("/stock/:id/block", post(handlers::block_material).delete(handlers::unblock_material))
        // Reports
        .route("/reports/stock-value", get(reports_handlers::get_stock_value_report))
        .route("/reports/stock-value/detail", get(reports_handlers::get_stock_value_detail))
        .route("/reports/consumption", get(reports_handlers::get_consumption_report))
        .route("/reports/most-requested", get(reports_handlers::get_most_requested_materials))
        .route("/reports/movement-analysis", get(reports_handlers::get_movement_analysis))
}
