use axum::{middleware, Router};
use core_services::security::security_headers;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    config, metrics,
    middleware::{mw_authenticate, mw_authorize},
    rate_limit::api_rate_limiter,
    state::AppState,
};

pub mod admin;
pub mod auth;
pub mod protected;
pub mod public;

pub fn build(app_state: AppState) -> Router {
    let public_routes = public::router().merge(auth::router(app_state.clone()));

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
        .merge(protected_routes)
        .with_state(app_state);

    apply_global_middleware(router)
}

fn apply_global_middleware(router: Router) -> Router {
    let headers = security_headers();

    router.layer(
        ServiceBuilder::new()
            .layer(middleware::from_fn(metrics::metrics_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(config::cors::configure())
            .layer(headers[0].clone())
            .layer(headers[1].clone())
            .layer(headers[2].clone())
            .layer(headers[3].clone())
            .layer(headers[4].clone()),
    )
}
