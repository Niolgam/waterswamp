pub mod contracts;
pub mod handlers;

use crate::infra::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/audit/logs", get(handlers::list_logs))
}
