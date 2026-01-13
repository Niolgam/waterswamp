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
    // 1. ROTAS PÚBLICAS
    // Inclui health checks, auth básico e a API pública de localizações para o mapa
    let public_routes = public::router()
        .merge(auth::router())
        .merge(email_verification::router())
        .merge(mfa::router());
    // .nest("/api/locations/public", locations::public_router());

    // 2. ROTAS DE UTILIZADOR (Apenas Autenticação JWT)
    let user_routes = users::router()
        .merge(auth::protected_router())
        .merge(email_verification::protected_router())
        .merge(mfa::protected_router())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ));

    // 3. ROTAS ADMINISTRATIVAS E PROTEGIDAS (JWT + Casbin RBAC)
    // Aqui incluímos geo_regions explicitamente como parte do admin ou rotas protegidas
    let admin_protected_routes = Router::new()
        .nest("/api/admin", admin::router()) // O admin::router já deve conter geo_regions internamente
        .merge(protected::router())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authorize,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ));

    // 4. MONTAGEM DO ROUTER PRINCIPAL
    // Unimos as rotas e aplicamos o Estado e o Rate Limiting uma única vez
    let main_router = Router::new()
        .merge(public_routes)
        .merge(user_routes)
        .merge(admin_protected_routes)
        .with_state(app_state.clone())
        .layer(api_rate_limiter());

    // 5. APLICAÇÃO DE MIDDLEWARE GLOBAL (CORS, Audit, Tracing)
    apply_global_middleware(main_router, app_state)

    // 6. ISOLAMENTO DO SWAGGER (A Solução Segura)
    // Geramos o Swagger separadamente. Ao fazer merge no final do objeto 'app',
    // evitamos que a complexidade do ApiDoc::openapi() interfira na construção
    // da árvore de tipos do router da aplicação.
    //  let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());

    // Merge final: aplicação processada + documentação
    // app.merge(swagger_ui)
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
