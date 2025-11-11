use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::http::{header, HeaderValue, Method};
use std::time::Duration;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};
use zxcvbn::{zxcvbn, Score};

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
    let estimate = zxcvbn(password, &[]);
    if estimate.score() < Score::Three {
        return Err("Senha muito fraca. Use uma senha mais complexa.".to_string());
    }
    Ok(())
}

/// Gera um hash Argon2id para a senha fornecida.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2id é o padrão da crate argon2 (v0.5+)
    let argon2 = Argon2::default();

    // Hash da senha (retorna um PasswordHash)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Erro interno de hash: {}", e))?;

    // Converte para o formato string PHC padrão ($argon2id$v=19$...)
    Ok(password_hash.to_string())
}

/// Verifica se a senha corresponde ao hash Argon2id fornecido.
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    // Parse do hash armazenado
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Formato de hash inválido: {}", e))?;

    // Verifica a senha
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
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
        assert!(validate_password_strength("F!8q@K39zP#s").is_ok());
        assert!(validate_password_strength("Tr0ng$ecuR3!Data#42").is_ok());
        assert!(validate_password_strength("C0mpl3x&P@ss#Sys2025").is_ok());
    }
}
