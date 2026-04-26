mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/abc-analysis/run", post(handlers::run_analysis))
        .route("/abc-analysis/results", get(handlers::get_results))
        .route("/abc-analysis/latest-run", get(handlers::get_latest_run))
}
