use axum::http::{header, HeaderValue, Method};
use std::time::Duration;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};

/// Configuração de CORS para produção
/// Permite apenas origens específicas
pub fn cors_production(allowed_origins: Vec<String>) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .into_iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

/// Configuração de CORS para desenvolvimento
/// Permite origens comuns de localhost explicitamente para suportar credenciais
pub fn cors_development() -> CorsLayer {
    // Lista expandida de origens comuns para desenvolvimento e testes
    let dev_origins = [
        "http://localhost",      // Usado frequentemente por ferramentas de teste
        "http://127.0.0.1",      // Variação de IP local
        "http://localhost:3000", // React, Node
        "http://localhost:4200", // Angular
        "http://localhost:5173", // Vite (Vue, React, Svelte)
        "http://localhost:8000", // Django, PHP, etc
        "http://localhost:8080", // Java, outras APIs
        "http://127.0.0.1:3000",
        "http://127.0.0.1:4200",
        "http://127.0.0.1:5173",
        "http://127.0.0.1:8000",
        "http://127.0.0.1:8080",
    ]
    .into_iter()
    .filter_map(|o| o.parse::<HeaderValue>().ok())
    .collect::<Vec<HeaderValue>>();

    CorsLayer::new()
        .allow_origin(dev_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true)
}

/// Headers de segurança (Helmet-style)
/// Retorna uma lista de layers para adicionar headers de segurança
pub fn security_headers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
    vec![
        // Previne MIME sniffing
        SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        // Protege contra clickjacking
        SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ),
        // Ativa proteção XSS do browser
        SetResponseHeaderLayer::if_not_present(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ),
        // Content Security Policy básico
        SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'"),
        ),
        // Controla quais features do browser estão disponíveis
        SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_production_parsing() {
        let origins = vec![
            "https://example.com".to_string(),
            "https://app.example.com".to_string(),
        ];
        let _cors = cors_production(origins);
        // Se compilar sem panic, está OK
    }

    #[test]
    fn test_security_headers_count() {
        let headers = security_headers();
        // Verifica que temos pelo menos 5 headers de segurança
        assert!(headers.len() >= 5);
    }
}
