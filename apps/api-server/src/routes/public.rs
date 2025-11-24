use crate::handlers::{health_handler, public_handler};
use crate::{infra::telemetry, state::AppState};
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/public", get(public_handler::handler_public))
        .route("/health", get(health_handler::handler_health))
        .route("/health/live", get(health_handler::handler_liveness))
        .route("/health/ready", get(health_handler::handler_ready))
        .route("/metrics", get(telemetry::handler_metrics))
}
