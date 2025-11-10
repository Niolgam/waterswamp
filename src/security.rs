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

pub fn validate_password_strength(password: &str) -> Result<(), String> {
    // Mínimo 8 caracteres
    if password.len() < 8 {
        return Err("Senha deve ter no mínimo 8 caracteres".to_string());
    }

    // Máximo 128 caracteres (prevenir DoS)
    if password.len() > 128 {
        return Err("Senha muito longa (máximo 128 caracteres)".to_string());
    }

    // Deve conter letra maiúscula
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Senha deve conter pelo menos uma letra maiúscula".to_string());
    }

    // Deve conter letra minúscula
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Senha deve conter pelo menos uma letra minúscula".to_string());
    }

    // Deve conter número
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Senha deve conter pelo menos um número".to_string());
    }

    // Deve conter caractere especial
    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        return Err("Senha deve conter pelo menos um caractere especial".to_string());
    }

    // Verificar senhas comuns
    const COMMON_PASSWORDS: &[&str] = &[
        "password",
        "12345678",
        "qwerty",
        "abc123",
        "password123",
        "111111",
        "123123",
        "admin",
        "letmein",
        "welcome",
        "monkey",
        "1234567890",
        "password1",
        "qwertyuiop",
        "123456789",
    ];

    let password_lower = password.to_lowercase();
    if COMMON_PASSWORDS.iter().any(|&p| password_lower.contains(p)) {
        return Err("Senha muito comum ou previsível. Escolha uma senha mais segura.".to_string());
    }

    // Verificar sequências repetidas (ex: "aaaaaa")
    let mut prev_char = '\0';
    let mut repeat_count = 0;
    for c in password.chars() {
        if c == prev_char {
            repeat_count += 1;
            if repeat_count >= 4 {
                return Err("Senha contém muitos caracteres repetidos".to_string());
            }
        } else {
            repeat_count = 1;
            prev_char = c;
        }
    }

    Ok(())
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

    #[test]
    fn test_password_too_short() {
        assert!(validate_password_strength("Abc1!").is_err());
    }

    #[test]
    fn test_password_no_uppercase() {
        assert!(validate_password_strength("abc123!@").is_err());
    }

    #[test]
    fn test_password_no_lowercase() {
        assert!(validate_password_strength("ABC123!@").is_err());
    }

    #[test]
    fn test_password_no_number() {
        assert!(validate_password_strength("Abcdefg!").is_err());
    }

    #[test]
    fn test_password_no_special() {
        assert!(validate_password_strength("Abcdefg123").is_err());
    }

    #[test]
    fn test_password_common() {
        assert!(validate_password_strength("Password123!").is_err());
    }

    #[test]
    fn test_password_repeated_chars() {
        assert!(validate_password_strength("Aaaaa123!").is_err());
    }

    #[test]
    fn test_password_valid() {
        assert!(validate_password_strength("MyP@ssw0rd2024").is_ok());
        assert!(validate_password_strength("Tr0ng$ecuR3!").is_ok());
        assert!(validate_password_strength("C0mpl3x&P@ss").is_ok());
    }
}
