pub mod handlers;
pub mod routes;

use crate::infra::state::AppState;
use axum::Router;

/// Re-exports requisition routes for admin integration
pub fn router() -> Router<AppState> {
    routes::requisition_routes()
}
