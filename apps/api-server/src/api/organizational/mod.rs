pub mod contracts;
pub mod handlers;
pub mod sync_handlers;

use crate::infra::state::AppState;
use axum::{routing::{delete, get, post}, Router};

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

    // SIORG Sync routes (immediate operations)
    let sync_router = Router::new()
        .route("/organization", post(handlers::sync_organization))
        .route("/unit", post(handlers::sync_unit))
        .route("/org-units", post(handlers::sync_organization_units))
        .route("/health", get(handlers::check_siorg_health));

    // Sync Queue Management routes
    let queue_router = Router::new()
        .route(
            "/",
            get(sync_handlers::list_queue_items),
        )
        .route("/stats", get(sync_handlers::get_queue_stats))
        .route(
            "/{id}",
            get(sync_handlers::get_queue_item).delete(sync_handlers::delete_queue_item),
        );

    // Conflict Resolution routes
    let conflicts_router = Router::new()
        .route("/", get(sync_handlers::list_conflicts))
        .route("/{id}", get(sync_handlers::get_conflict_detail))
        .route("/{id}/resolve", post(sync_handlers::resolve_conflict));

    // Sync History routes
    let history_router = Router::new()
        .route("/", get(sync_handlers::list_history))
        .route("/{id}", get(sync_handlers::get_history_item))
        .route("/{id}/review", post(sync_handlers::review_history_item))
        .route(
            "/entity/{entity_type}/{siorg_code}",
            get(sync_handlers::get_entity_history),
        );

    Router::new()
        .nest("/settings", settings_router)
        .nest("/organizations", organizations_router)
        .nest("/unit-categories", unit_categories_router)
        .nest("/unit-types", unit_types_router)
        .nest("/units", units_router)
        .nest("/sync", sync_router)
        .nest("/sync/queue", queue_router)
        .nest("/sync/conflicts", conflicts_router)
        .nest("/sync/history", history_router)
}
