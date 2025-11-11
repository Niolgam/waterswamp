use crate::{
    metrics,
    middleware::{mw_authenticate, mw_authorize},
    rate_limit::{admin_rate_limiter, api_rate_limiter, login_rate_limiter},
    security::{cors_development, cors_production, security_headers},
    state::AppState,
};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod admin_handler;
pub mod auth_handler;
pub mod health_handler;
mod protected;
mod public;

pub fn build_router(app_state: AppState) -> Router {
    // --- Rotas Públicas (sem autenticação) ---
    let public_routes = Router::new()
        .route("/public", get(public::handler_public))
        .route("/health", get(health_handler::handler_health))
        .route("/health/live", get(health_handler::handler_liveness))
        .route("/health/ready", get(health_handler::handler_ready))
        .route("/metrics", get(metrics::handler_metrics))
        .route("/login", post(auth_handler::handler_login))
        .route("/register", post(auth_handler::handler_register))
        .route("/refresh-token", post(auth_handler::handler_refresh_token))
        .route("/logout", post(auth_handler::handler_logout))
        .layer(login_rate_limiter());

    // --- Rotas Protegidas (requerem autenticação E autorização) ---
    let protected_routes = Router::new()
        .route("/users/profile", get(protected::handler_user_profile))
        .route("/admin/dashboard", get(protected::handler_admin_dashboard))
        // Rotas de admin com rate limiting dedicado
        .route("/api/admin/policies", post(admin_handler::add_policy))
        .route("/api/admin/policies", delete(admin_handler::remove_policy))
        .route("/api/admin/users", get(admin_handler::list_users))
        .route("/api/admin/users", post(admin_handler::create_user))
        .route("/api/admin/users/{id}", get(admin_handler::get_user))
        .route("/api/admin/users/{id}", put(admin_handler::update_user))
        .route("/api/admin/users/{id}", delete(admin_handler::delete_user))
        .layer(admin_rate_limiter())
        // Middlewares de autenticação e autorização (ordem importa!)
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authenticate,
        ))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            mw_authorize,
        ))
        // Rate limiting geral para rotas protegidas
        .layer(api_rate_limiter());

    // --- Configuração de CORS baseada no ambiente ---
    let cors_layer = match std::env::var("ENVIRONMENT") {
        Ok(env) if env == "production" => {
            let allowed_origins = std::env::var("CORS_ALLOW_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            cors_production(allowed_origins)
        }
        _ => {
            tracing::warn!("⚠️  Usando CORS permissivo (desenvolvimento)");
            cors_development()
        }
    };

    // --- Montagem Final do Router ---
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state)
        // Camada de serviços globais (aplicados a TODAS as rotas)
        .layer(
            ServiceBuilder::new()
                // ⭐ NOVO: Middleware de métricas
                .layer(middleware::from_fn(metrics::metrics_middleware))
                // Tracing para observabilidade
                .layer(TraceLayer::new_for_http())
                // CORS
                .layer(cors_layer)
                // Headers de segurança (Helmet-style)
                .layer(
                    ServiceBuilder::new()
                        .layer(security_headers()[0].clone())
                        .layer(security_headers()[1].clone())
                        .layer(security_headers()[2].clone())
                        .layer(security_headers()[3].clone())
                        .layer(security_headers()[4].clone()),
                ),
        )
}
