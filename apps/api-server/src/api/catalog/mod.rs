pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

/// Creates the catalog router with all CRUD routes
pub fn router() -> Router<AppState> {
    // Units of Measure routes
    let units_router = Router::new()
        .route(
            "/",
            get(handlers::list_units_of_measure).post(handlers::create_unit_of_measure),
        )
        .route(
            "/{id}",
            get(handlers::get_unit_of_measure)
                .put(handlers::update_unit_of_measure)
                .delete(handlers::delete_unit_of_measure),
        );

    // Catalog Groups routes
    let groups_router = Router::new()
        .route(
            "/",
            get(handlers::list_catalog_groups).post(handlers::create_catalog_group),
        )
        .route("/tree", get(handlers::get_catalog_group_tree))
        .route(
            "/{id}",
            get(handlers::get_catalog_group)
                .put(handlers::update_catalog_group)
                .delete(handlers::delete_catalog_group),
        );

    // Catalog Items routes
    let items_router = Router::new()
        .route(
            "/",
            get(handlers::list_catalog_items).post(handlers::create_catalog_item),
        )
        .route(
            "/{id}",
            get(handlers::get_catalog_item)
                .put(handlers::update_catalog_item)
                .delete(handlers::delete_catalog_item),
        );

    // Unit Conversions routes
    let conversions_router = Router::new()
        .route(
            "/",
            get(handlers::list_unit_conversions).post(handlers::create_unit_conversion),
        )
        .route(
            "/{id}",
            get(handlers::get_unit_conversion)
                .put(handlers::update_unit_conversion)
                .delete(handlers::delete_unit_conversion),
        );

    Router::new()
        .nest("/units-of-measure", units_router)
        .nest("/groups", groups_router)
        .nest("/items", items_router)
        .nest("/conversions", conversions_router)
}
