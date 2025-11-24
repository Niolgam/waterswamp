//! User self-service routes
//!
//! This module provides endpoints for users to manage their own profiles.

use axum::{
    routing::{get, put},
    Router,
};

use crate::infra::state::AppState;

pub mod handlers;

/// Creates the users router with all self-service endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/profile", get(handlers::get_profile))
        .route("/profile", put(handlers::update_profile))
        .route("/password", put(handlers::change_password))
}
