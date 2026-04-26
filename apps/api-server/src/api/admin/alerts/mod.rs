mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stock-alerts", get(handlers::list_alerts))
        .route("/stock-alerts", post(handlers::create_alert))
        .route("/stock-alerts/sla-check", post(handlers::process_sla_breaches))
        .route("/stock-alerts/:id", get(handlers::get_alert))
        .route("/stock-alerts/:id/acknowledge", post(handlers::acknowledge_alert))
        .route("/stock-alerts/:id/resolve", post(handlers::resolve_alert))
}
