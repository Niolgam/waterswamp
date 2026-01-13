use axum::{middleware, Router};
use core_services::security::security_headers;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{admin, auth, email_verification, mfa, users},
    infra::{cors, telemetry},
    middleware::audit,
    middleware::rate_limit::api_rate_limiter,
    middleware::{mw_authenticate, mw_authorize},
    openapi::ApiDoc,
    state::AppState,
};

pub mod protected;
pub mod public;

pub fn build(app_state: AppState) -> Router {
    // 1. Rotas Públicas
    let public_routes = public::router()
        .merge(auth::router())
        .merge(email_verification::router())
        .merge(mfa::router());

    // 2. Rotas Autenticadas (requerem apenas JWT)
    let authenticated_routes = auth::protected_router()
        .merge(email_verification::protected_router())
        .merge(mfa::protected_router())
        .merge(users::router())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ))
        .layer(api_rate_limiter());

    // 3. Rotas Protegidas (requerem JWT + Autorização Casbin)
    let protected_routes = protected::router()
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authorize,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ))
        .layer(api_rate_limiter());

    // 4. Rotas Administrativas (com prefixo /api/admin)
    let admin_routes = Router::new()
        .nest("/api/admin", admin::router()) // Aqui o admin::router já deve ter o geo_regions corrigido
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authorize,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ))
        .layer(api_rate_limiter());

    // 5. Swagger UI
    let swagger_routes =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());

    // Montagem do router base
    let router = Router::new()
        .merge(public_routes)
        .merge(authenticated_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(app_state.clone())
        .merge(swagger_routes);

    // Aplicação das camadas globais (CORS, Headers, Audit, etc.)
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
