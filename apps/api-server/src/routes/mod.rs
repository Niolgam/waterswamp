use axum::{middleware, Router};
use core_services::security::security_headers;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    infra::{cors, telemetry},
    middleware::audit,
    middleware::rate_limit::api_rate_limiter,
    middleware::{mw_authenticate, mw_authorize},
    state::AppState,
};

pub mod admin;
pub mod auth;
pub mod protected;
pub mod public;

pub fn build(app_state: AppState) -> Router {
    let public_routes = public::router().merge(auth::router(app_state.clone()));

    // Routes that require authentication but NOT Casbin authorization
    // (any authenticated user can access these)
    let authenticated_routes = auth::protected_auth_router()
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ))
        .layer(api_rate_limiter());

    // Routes that require authentication AND Casbin authorization
    // (role-based access control)
    let protected_routes = protected::router()
        .merge(admin::router())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authorize,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ))
        .layer(api_rate_limiter());

    let router = Router::new()
        .merge(public_routes)
        .merge(authenticated_routes)
        .merge(protected_routes)
        .with_state(app_state.clone());

    apply_global_middleware(router, app_state)
}

fn apply_global_middleware(router: Router, app_state: AppState) -> Router {
    let headers = security_headers();

    router.layer(
        ServiceBuilder::new()
            .layer(middleware::from_fn_with_state(app_state, audit::mw_audit))
            .layer(middleware::from_fn(telemetry::metrics_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(cors::configure())
            .layer(headers[0].clone())
            .layer(headers[1].clone())
            .layer(headers[2].clone())
            .layer(headers[3].clone())
            .layer(headers[4].clone()),
    )
}
