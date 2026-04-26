mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/legacy-import/jobs", get(handlers::list_jobs))
        .route("/legacy-import/jobs/:id", get(handlers::get_job))
        .route("/legacy-import/suppliers", post(handlers::import_suppliers))
        .route("/legacy-import/catalog-items", post(handlers::import_catalog_items))
        .route("/legacy-import/initial-stock", post(handlers::import_initial_stock))
}
