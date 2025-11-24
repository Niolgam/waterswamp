use core_services::security::{cors_development, cors_production};
use tower_http::cors::CorsLayer;

pub fn configure() -> CorsLayer {
    match std::env::var("ENVIRONMENT") {
        Ok(env) if env == "production" => {
            let origins = parse_allowed_origins();
            cors_production(origins)
        }
        _ => {
            tracing::warn!("⚠️  Usando CORS permissivo (desenvolvimento)");
            cors_development()
        }
    }
}

fn parse_allowed_origins() -> Vec<String> {
    std::env::var("CORS_ALLOW_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}
