//! Departments Module
//!
//! Handles DepartmentCategory entities for organizational classification.

pub mod contracts;
pub mod handlers;

pub use contracts::DepartmentCategoryResponse;

use crate::infra::state::AppState;
use axum::{routing::get, Router};

/// Creates the departments router with DepartmentCategory routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handlers::list_department_categories).post(handlers::create_department_category),
        )
        .route(
            "/{id}",
            get(handlers::get_department_category)
                .put(handlers::update_department_category)
                .delete(handlers::delete_department_category),
        )
}
