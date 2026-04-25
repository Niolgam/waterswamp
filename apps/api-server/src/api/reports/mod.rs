pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/fuel-consumption", get(handlers::get_fuel_consumption))
        .route("/fleet-summary", get(handlers::get_fleet_summary))
        .route("/vehicles/{id}/dashboard", get(handlers::get_vehicle_dashboard))
}
