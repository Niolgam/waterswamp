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
        .route("/{id}/operational-status", axum::routing::put(handlers::change_operational_status))
        .route("/{id}/history", get(handlers::get_vehicle_status_history))
        .route("/{id}/odometer", axum::routing::post(handlers::register_odometer_reading).get(handlers::list_odometer_readings))
        .route("/{id}/odometer/projection", get(handlers::get_odometer_projection))
        // RF-AST-06: Transferência departamental
        .route("/{id}/transfers", axum::routing::post(handlers::register_department_transfer).get(handlers::list_department_transfers))
        // RF-AST-11: Depreciação
        .route("/{id}/depreciation", get(handlers::get_vehicle_depreciation))
        // RF-AST-12: Sinistros
        .route("/{id}/incidents", axum::routing::post(handlers::open_incident).get(handlers::list_incidents))
        // RF-AST-09/10: Processo de baixa
        .route("/{id}/disposal", axum::routing::post(handlers::open_disposal).get(handlers::get_disposal_by_vehicle))
        // RF-MNT: Manutenção (atalho por veículo)
        .route("/{id}/maintenance", axum::routing::post(handlers::open_maintenance_order).get(handlers::list_maintenance_orders))
        .route("/{id}/maintenance/cost", get(handlers::get_maintenance_cost_summary))
        .route(
            "/{id}/documents",
            get(handlers::list_vehicle_documents).post(handlers::upload_vehicle_document),
        )
        .route(
            "/{vehicle_id}/documents/{doc_id}",
            axum::routing::delete(handlers::delete_vehicle_document),
        );

    // Odometer quarantine resolution + incident update (standalone routes)
    let odometer_router = Router::new()
        .route("/{reading_id}/resolve", axum::routing::put(handlers::resolve_odometer_quarantine));

    let incidents_router = Router::new()
        .route("/{incident_id}", axum::routing::put(handlers::update_incident));

    // RF-AST-09/10: Disposal process management
    let disposal_router = Router::new()
        .route("/", get(handlers::list_disposals))
        .route("/{disposal_id}", axum::routing::put(handlers::advance_disposal))
        .route("/{disposal_id}/steps", axum::routing::post(handlers::add_disposal_step).get(handlers::list_disposal_steps));

    // RF-AST-11: Depreciation config management (admin)
    let depreciation_router = Router::new()
        .route("/", get(handlers::list_depreciation_configs).post(handlers::upsert_depreciation_config));

    // RF-ADM-07: Fleet fuel catalog
    let fuel_catalog_router = Router::new()
        .route("/", get(handlers::list_fuels).post(handlers::create_fuel))
        .route("/{id}", axum::routing::put(handlers::update_fuel));

    // RF-ADM-08: Fleet maintenance services catalog
    let maintenance_services_router = Router::new()
        .route("/", get(handlers::list_maintenance_services).post(handlers::create_maintenance_service))
        .route("/{id}", axum::routing::put(handlers::update_maintenance_service));

    // RF-ADM-01: Fleet system params
    let system_params_router = Router::new()
        .route("/", get(handlers::list_system_params).post(handlers::upsert_system_param));

    // RF-ADM-02: Checklist templates
    let checklist_router = Router::new()
        .route("/", get(handlers::list_checklist_templates).post(handlers::create_checklist_template))
        .route("/{template_id}/items", axum::routing::post(handlers::add_checklist_item).get(handlers::list_checklist_items));

    Router::new()
        .nest("/categories", categories_router)
        .nest("/makes", makes_router)
        .nest("/models", models_router)
        .nest("/colors", colors_router)
        .nest("/fuel-types", fuel_types_router)
        .nest("/transmission-types", transmission_types_router)
        .nest("/vehicles", vehicles_router)
        .nest("/odometer", odometer_router)
        .nest("/incidents", incidents_router)
        .nest("/disposals", disposal_router)
        .nest("/depreciation-configs", depreciation_router)
        .nest("/fuel-catalog", fuel_catalog_router)
        .nest("/maintenance-services", maintenance_services_router)
        .nest("/system-params", system_params_router)
        .nest("/checklists", checklist_router)
}
