pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

/// Creates the catalog router with all CRUD routes for CATMAT, CATSER, units, and conversions
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

    // CATMAT Groups routes
    let catmat_groups_router = Router::new()
        .route(
            "/",
            get(handlers::list_catmat_groups).post(handlers::create_catmat_group),
        )
        .route("/tree", get(handlers::get_catmat_tree))
        .route(
            "/{id}",
            get(handlers::get_catmat_group)
                .put(handlers::update_catmat_group)
                .delete(handlers::delete_catmat_group),
        );

    // CATMAT Classes routes
    let catmat_classes_router = Router::new()
        .route(
            "/",
            get(handlers::list_catmat_classes).post(handlers::create_catmat_class),
        )
        .route(
            "/{id}",
            get(handlers::get_catmat_class)
                .put(handlers::update_catmat_class)
                .delete(handlers::delete_catmat_class),
        );

    // CATMAT Items (PDM) routes
    let catmat_items_router = Router::new()
        .route(
            "/",
            get(handlers::list_catmat_items).post(handlers::create_catmat_item),
        )
        .route(
            "/{id}",
            get(handlers::get_catmat_item)
                .put(handlers::update_catmat_item)
                .delete(handlers::delete_catmat_item),
        );

    // CATSER Groups routes
    let catser_groups_router = Router::new()
        .route(
            "/",
            get(handlers::list_catser_groups).post(handlers::create_catser_group),
        )
        .route("/tree", get(handlers::get_catser_tree))
        .route(
            "/{id}",
            get(handlers::get_catser_group)
                .put(handlers::update_catser_group)
                .delete(handlers::delete_catser_group),
        );

    // CATSER Classes routes
    let catser_classes_router = Router::new()
        .route(
            "/",
            get(handlers::list_catser_classes).post(handlers::create_catser_class),
        )
        .route(
            "/{id}",
            get(handlers::get_catser_class)
                .put(handlers::update_catser_class)
                .delete(handlers::delete_catser_class),
        );

    // CATSER Items (Serviço) routes
    let catser_items_router = Router::new()
        .route(
            "/",
            get(handlers::list_catser_items).post(handlers::create_catser_item),
        )
        .route(
            "/{id}",
            get(handlers::get_catser_item)
                .put(handlers::update_catser_item)
                .delete(handlers::delete_catser_item),
        );

    Router::new()
        .nest("/units-of-measure", units_router)
        .nest("/conversions", conversions_router)
        .nest("/catmat/groups", catmat_groups_router)
        .nest("/catmat/classes", catmat_classes_router)
        .nest("/catmat/items", catmat_items_router)
        .nest("/catser/groups", catser_groups_router)
        .nest("/catser/classes", catser_classes_router)
        .nest("/catser/items", catser_items_router)
}
