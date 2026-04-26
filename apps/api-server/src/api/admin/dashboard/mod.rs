mod handlers;

use crate::infra::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard/stock-summary", get(handlers::get_stock_summary))
        .route("/dashboard/daily-movements", get(handlers::get_daily_movements))
        .route("/dashboard/supplier-performance", get(handlers::get_supplier_performance))
        .route("/dashboard/refresh", post(handlers::refresh_all))
}
