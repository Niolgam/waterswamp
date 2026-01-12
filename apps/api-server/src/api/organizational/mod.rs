pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{routing::{get, post}, Router};

/// Creates the organizational router with all CRUD routes
pub fn router() -> Router<AppState> {
    // System Settings routes
    let settings_router = Router::new()
        .route(
            "/",
            get(handlers::list_system_settings).post(handlers::create_system_setting),
        )
        .route(
            "/{key}",
            get(handlers::get_system_setting)
                .put(handlers::update_system_setting)
                .delete(handlers::delete_system_setting),
        );

    // Organizations routes
    let organizations_router = Router::new()
        .route(
            "/",
            get(handlers::list_organizations).post(handlers::create_organization),
        )
        .route(
            "/{id}",
            get(handlers::get_organization)
                .put(handlers::update_organization)
                .delete(handlers::delete_organization),
        );

    // Unit Categories routes
    let unit_categories_router = Router::new()
        .route(
            "/",
            get(handlers::list_unit_categories).post(handlers::create_unit_category),
        )
        .route(
            "/{id}",
            get(handlers::get_unit_category)
                .put(handlers::update_unit_category)
                .delete(handlers::delete_unit_category),
        );

    // Unit Types routes
    let unit_types_router = Router::new()
        .route("/", get(handlers::list_unit_types).post(handlers::create_unit_type))
        .route(
            "/{id}",
            get(handlers::get_unit_type)
                .put(handlers::update_unit_type)
                .delete(handlers::delete_unit_type),
        );

    // Organizational Units routes
    let units_router = Router::new()
        .route(
            "/",
            get(handlers::list_organizational_units).post(handlers::create_organizational_unit),
        )
        .route("/tree", get(handlers::get_organizational_units_tree))
        .route(
            "/{id}",
            get(handlers::get_organizational_unit)
                .put(handlers::update_organizational_unit)
                .delete(handlers::delete_organizational_unit),
        )
        .route("/{id}/children", get(handlers::get_organizational_unit_children))
        .route("/{id}/path", get(handlers::get_organizational_unit_path))
        .route("/{id}/deactivate", get(handlers::deactivate_organizational_unit))
        .route("/{id}/activate", get(handlers::activate_organizational_unit));

    // SIORG Sync routes
    let sync_router = Router::new()
        .route("/organization", post(handlers::sync_organization))
        .route("/unit", post(handlers::sync_unit))
        .route("/org-units", post(handlers::sync_organization_units))
        .route("/health", get(handlers::check_siorg_health));

    Router::new()
        .nest("/settings", settings_router)
        .nest("/organizations", organizations_router)
        .nest("/unit-categories", unit_categories_router)
        .nest("/unit-types", unit_types_router)
        .nest("/units", units_router)
        .nest("/sync", sync_router)
}
