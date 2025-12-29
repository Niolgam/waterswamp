//! Warehouse Module
//!
//! Handles warehouse operations including:
//! - Material Groups: Classification and organization of materials
//! - Materials: Individual materials/services with their properties

pub mod contracts;
pub mod handlers;

pub use contracts::{MaterialGroupResponse, MaterialResponse, MaterialWithGroupResponse};

use crate::infra::state::AppState;
use axum::{routing::get, Router};

/// Creates the warehouse router with Material Groups and Materials routes
pub fn router() -> Router<AppState> {
    Router::new()
        // Material Groups routes
        .route(
            "/material-groups",
            get(handlers::list_material_groups).post(handlers::create_material_group),
        )
        .route(
            "/material-groups/{id}",
            get(handlers::get_material_group)
                .put(handlers::update_material_group)
                .delete(handlers::delete_material_group),
        )
        // Materials routes
        .route(
            "/materials",
            get(handlers::list_materials).post(handlers::create_material),
        )
        .route(
            "/materials/{id}",
            get(handlers::get_material)
                .put(handlers::update_material)
                .delete(handlers::delete_material),
        )
}
