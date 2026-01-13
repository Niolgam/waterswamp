pub mod contracts;
pub mod handlers;

pub use contracts::{BudgetClassificationResponse, BudgetClassificationWithParentResponse};

use crate::infra::state::AppState;
use axum::{routing::get, Router};

/// Creates the budget_classifications router with all CRUD routes
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handlers::list_budget_classifications).post(handlers::create_budget_classification),
        )
        .route("/tree", get(handlers::get_tree))
        .route(
            "/{id}",
            get(handlers::get_budget_classification)
                .put(handlers::update_budget_classification)
                .delete(handlers::delete_budget_classification),
        )
}
