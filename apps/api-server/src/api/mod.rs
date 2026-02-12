//! Cada feature é autocontida e exporta:
//! - `router()` - Router Axum com todas as rotas da feature
//! - `contracts` - DTOs de request/response
//!

pub mod admin;
pub mod auth;
pub mod budget_classifications;
pub mod catalog;
pub mod email_verification;
pub mod fleet;
pub mod geo_regions;
pub mod suppliers;
pub mod drivers;
pub mod fuelings;
pub mod vehicle_fines;
pub mod locations;
pub mod mfa;
pub mod organizational;
pub mod users;

use axum::Router;

use crate::infra::state::AppState;
