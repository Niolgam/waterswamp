pub mod contracts;
pub mod handlers;

use crate::state::AppState;
use axum::{routing::get, Router};

/// Creates the fleet management router with all CRUD routes
pub fn router() -> Router<AppState> {
    // Vehicle Categories routes
    let categories_router = Router::new()
        .route("/", get(handlers::list_vehicle_categories).post(handlers::create_vehicle_category))
        .route(
            "/{id}",
            get(handlers::get_vehicle_category)
                .put(handlers::update_vehicle_category)
                .delete(handlers::delete_vehicle_category),
        );

    // Vehicle Makes routes
    let makes_router = Router::new()
        .route("/", get(handlers::list_vehicle_makes).post(handlers::create_vehicle_make))
        .route(
            "/{id}",
            get(handlers::get_vehicle_make)
                .put(handlers::update_vehicle_make)
                .delete(handlers::delete_vehicle_make),
        );

    // Vehicle Models routes
    let models_router = Router::new()
        .route("/", get(handlers::list_vehicle_models).post(handlers::create_vehicle_model))
        .route(
            "/{id}",
            get(handlers::get_vehicle_model)
                .put(handlers::update_vehicle_model)
                .delete(handlers::delete_vehicle_model),
        );

    // Vehicle Colors routes
    let colors_router = Router::new()
        .route("/", get(handlers::list_vehicle_colors).post(handlers::create_vehicle_color))
        .route(
            "/{id}",
            get(handlers::get_vehicle_color)
                .put(handlers::update_vehicle_color)
                .delete(handlers::delete_vehicle_color),
        );

    // Vehicle Fuel Types routes
    let fuel_types_router = Router::new()
        .route("/", get(handlers::list_vehicle_fuel_types).post(handlers::create_vehicle_fuel_type))
        .route(
            "/{id}",
            get(handlers::get_vehicle_fuel_type)
                .put(handlers::update_vehicle_fuel_type)
                .delete(handlers::delete_vehicle_fuel_type),
        );

    // Vehicle Transmission Types routes
    let transmission_types_router = Router::new()
        .route("/", get(handlers::list_vehicle_transmission_types).post(handlers::create_vehicle_transmission_type))
        .route(
            "/{id}",
            get(handlers::get_vehicle_transmission_type)
                .put(handlers::update_vehicle_transmission_type)
                .delete(handlers::delete_vehicle_transmission_type),
        );

    // Vehicles routes (main)
    let vehicles_router = Router::new()
        .route("/", get(handlers::list_vehicles).post(handlers::create_vehicle))
        .route("/search", get(handlers::search_vehicles))
        .route(
            "/{id}",
            get(handlers::get_vehicle)
                .put(handlers::update_vehicle)
                .delete(handlers::delete_vehicle),
        )
        .route("/{id}/status", axum::routing::put(handlers::change_vehicle_status))
        .route("/{id}/history", get(handlers::get_vehicle_status_history))
        .route(
            "/{id}/documents",
            get(handlers::list_vehicle_documents).post(handlers::upload_vehicle_document),
        )
        .route(
            "/{vehicle_id}/documents/{doc_id}",
            axum::routing::delete(handlers::delete_vehicle_document),
        );

    Router::new()
        .nest("/categories", categories_router)
        .nest("/makes", makes_router)
        .nest("/models", models_router)
        .nest("/colors", colors_router)
        .nest("/fuel-types", fuel_types_router)
        .nest("/transmission-types", transmission_types_router)
        .nest("/vehicles", vehicles_router)
}
