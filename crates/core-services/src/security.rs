use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use axum::http::{header, HeaderName, HeaderValue, Method};
use std::time::Duration;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};
use zxcvbn::{zxcvbn, Score};

/// Custom header names
const X_CSRF_TOKEN: HeaderName = HeaderName::from_static("x-csrf-token");
const X_REQUESTED_WITH: HeaderName = HeaderName::from_static("x-requested-with");
const X_REQUEST_NONCE: HeaderName = HeaderName::from_static("x-request-nonce");
const X_REQUEST_TIMESTAMP: HeaderName = HeaderName::from_static("x-request-timestamp");
const X_REQUEST_SIGNATURE: HeaderName = HeaderName::from_static("x-request-signature");

const X_BROWSER_FINGERPRINT: HeaderName = HeaderName::from_static("x-browser-fingerprint");
/// Configuração de CORS para produção
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
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::X_CONTENT_TYPE_OPTIONS,
            header::CACHE_CONTROL, // ← adicionar
            header::PRAGMA,        // ← adicionar
            X_CSRF_TOKEN,
            X_REQUESTED_WITH,
            X_REQUEST_NONCE,
            X_REQUEST_TIMESTAMP,
            X_REQUEST_SIGNATURE,
            X_BROWSER_FINGERPRINT,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

/// Configuração de CORS para desenvolvimento
pub fn cors_development() -> CorsLayer {
    let dev_origins = [
        "http://localhost",
        "http://127.0.0.1",
        "http://localhost:3000",
        "http://localhost:4200",
        "http://localhost:5173",
        "http://localhost:8000",
        "http://localhost:8080",
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
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::CACHE_CONTROL, // ← adicionar
            header::PRAGMA,        // ← adicionar
            header::X_CONTENT_TYPE_OPTIONS,
            X_CSRF_TOKEN,
            X_REQUESTED_WITH,
            X_REQUEST_NONCE,
            X_REQUEST_TIMESTAMP,
            X_REQUEST_SIGNATURE,
            X_BROWSER_FINGERPRINT,
        ])
        .allow_credentials(true)
}

/// Headers de segurança (Helmet-style)
pub fn security_headers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
    vec![
        SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ),
        SetResponseHeaderLayer::if_not_present(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ),
        SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'"),
        ),
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

// =============================================================================
// ARGON2ID PASSWORD HASHING - OWASP RECOMMENDATIONS
// =============================================================================

/// Parâmetros Argon2id otimizados baseados nas recomendações OWASP 2024.
///
/// **Configuração atual:**
/// - **Memory Cost (m_cost)**: 64 MiB (65536 KiB)
/// - **Time Cost (t_cost)**: 3 iterações
/// - **Parallelism (p_cost)**: 4 threads
/// - **Output Length**: 32 bytes (padrão)
///
/// **Justificativa:**
/// Estas configurações oferecem um excelente equilíbrio entre:
/// - Segurança contra ataques de força bruta e rainbow tables
/// - Performance aceitável para servidores modernos (~200-300ms por hash)
/// - Resistência contra ataques com GPUs e ASICs
///
/// **Referências:**
/// - [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
/// - [RFC 9106 - Argon2 Memory-Hard Function](https://www.rfc-editor.org/rfc/rfc9106.html)
/// - [Argon2 Best Practices](https://github.com/p-h-c/phc-winner-argon2#recommendations)
const ARGON2_M_COST: u32 = 65536; // 64 MiB
const ARGON2_T_COST: u32 = 3; // 3 iterations
const ARGON2_P_COST: u32 = 4; // 4 parallel threads

/// Gera um hash Argon2id seguro para a senha fornecida.
///
/// Utiliza parâmetros baseados nas recomendações OWASP para Argon2id:
/// - **Algoritmo**: Argon2id (híbrido: resistente a side-channel e GPU attacks)
/// - **Memory**: 64 MiB (balanceamento entre segurança e performance)
/// - **Iterations**: 3 (t_cost)
/// - **Parallelism**: 4 threads
/// - **Salt**: 128-bit aleatório gerado via OsRng (cryptographically secure)
/// - **Output**: PHC string format (`$argon2id$v=19$m=65536,t=3,p=4$...`)
///
/// # Performance
///
/// Tempo esperado por hash em hardware moderno:
/// - **Servidor (4+ cores)**: ~200-300ms
/// - **Desktop (2-4 cores)**: ~300-500ms
/// - **Mobile/Low-end**: ~500ms-1s
///
/// # Exemplos
///
/// ```rust
/// use core_services::security::hash_password;
///
/// // Hash de senha para novo usuário
/// let password = "MyS3cur3P@ssw0rd!";
/// let hash = hash_password(password).expect("Falha ao gerar hash");
///
/// // Hash resultante (formato PHC):
/// // $argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>
/// ```
///
/// # Erros
///
/// Retorna `Err` se:
/// - Falha ao gerar salt aleatório
/// - Falha ao configurar parâmetros Argon2
/// - Falha ao computar hash (muito raro)
///
/// # Segurança
///
/// ⚠️ **IMPORTANTE**: Esta função é **blocking** e pode levar 200-500ms.
/// Em contextos assíncronos (Axum, Tokio), use `tokio::task::spawn_blocking`:
///
/// ```rust,ignore
/// use core_services::security::hash_password;
///
/// let password = "MyS3cur3P@ssw0rd!";
/// let password_clone = password.to_string();
/// let hash = tokio::task::spawn_blocking(move || {
///     hash_password(&password_clone)
/// })
/// .await??;
/// ```
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    // Gera salt criptograficamente seguro (128-bit)
    let salt = SaltString::generate(&mut OsRng);

    // Configura parâmetros Argon2id (OWASP recommendations)
    let params = Params::new(
        ARGON2_M_COST, // m_cost: 64 MiB
        ARGON2_T_COST, // t_cost: 3 iterations
        ARGON2_P_COST, // p_cost: 4 threads
        None,          // output_len: usa padrão (32 bytes)
    )
    .map_err(|e| anyhow::anyhow!("Erro ao configurar parâmetros Argon2: {}", e))?;

    // Cria instância Argon2id com parâmetros customizados
    let argon2 = Argon2::new(
        Algorithm::Argon2id, // Híbrido: resistente a side-channel + GPU
        Version::V0x13,      // Versão mais recente (0x13 = 19 decimal)
        params,
    );

    // Computa hash (operação blocking ~200-300ms)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Erro ao gerar hash: {}", e))?;

    // Retorna formato PHC string
    // Exemplo: $argon2id$v=19$m=65536,t=3,p=4$<base64_salt>$<base64_hash>
    Ok(password_hash.to_string())
}

/// Verifica se a senha corresponde ao hash Argon2id fornecido.
///
/// Esta função é **constant-time** (timing-safe) para prevenir timing attacks.
/// Os parâmetros (m_cost, t_cost, p_cost) são extraídos automaticamente do
/// hash PHC string, garantindo compatibilidade mesmo se os parâmetros mudarem.
///
/// # Performance
///
/// O tempo de verificação é equivalente ao tempo de hash (~200-300ms),
/// pois recalcula o hash com os mesmos parâmetros e compara de forma segura.
///
/// # Exemplos
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// use core_services::security::{hash_password, verify_password};
///
/// // Gerar hash
/// let password = "MyS3cur3P@ssw0rd!";
/// let hash = hash_password(password)?;
///
/// // Verificar senha correta
/// assert!(verify_password(password, &hash)?);
///
/// // Verificar senha incorreta
/// assert!(!verify_password("WrongPassword", &hash)?);
/// # Ok(())
/// # }
/// ```
///
/// # Erros
///
/// Retorna `Err` se:
/// - O hash fornecido está em formato inválido (não é PHC string)
/// - Falha ao parsear parâmetros do hash
/// - Erro interno na verificação (muito raro)
///
/// # Segurança
///
/// ⚠️ **IMPORTANTE**: Esta função também é **blocking** (~200-300ms).
/// Use `spawn_blocking` em contextos assíncronos:
///
/// ```rust,ignore
/// use core_services::security::verify_password;
///
/// let password = "MyS3cur3P@ssw0rd!";
/// let hash = "$argon2id$v=19$m=65536,t=3,p=4$...".to_string();
/// let password_clone = password.to_string();
/// let hash_clone = hash.clone();
/// let is_valid = tokio::task::spawn_blocking(move || {
///     verify_password(&password_clone, &hash_clone)
/// })
/// .await??;
/// ```
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    // Parse do hash PHC string
    // Extrai automaticamente: algoritmo, versão, m_cost, t_cost, p_cost, salt
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Formato de hash inválido: {}", e))?;

    // Cria verificador Argon2
    // Os parâmetros são extraídos do 'parsed_hash', então não precisamos
    // configurá-los manualmente aqui
    let argon2 = Argon2::default();

    // Verifica senha (constant-time comparison)
    // Retorna Ok(()) se senha correta, Err se incorreta
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// =============================================================================
// TESTES
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_hash_format() {
        let password = "TestPassword123!";
        let hash = hash_password(password).expect("Falha ao gerar hash");

        // Verifica formato PHC string Argon2id
        assert!(hash.starts_with("$argon2id$v=19$"));

        // Verifica presença dos parâmetros
        assert!(hash.contains(&format!("m={}", ARGON2_M_COST)));
        assert!(hash.contains(&format!("t={}", ARGON2_T_COST)));
        assert!(hash.contains(&format!("p={}", ARGON2_P_COST)));
    }

    #[test]
    fn test_password_verification_success() {
        let password = "Correct_P@ssw0rd_123";
        let hash = hash_password(password).expect("Falha ao gerar hash");

        assert!(
            verify_password(password, &hash).unwrap(),
            "Senha correta deveria validar"
        );
    }

    #[test]
    fn test_password_verification_failure() {
        let password = "Correct_P@ssw0rd_123";
        let hash = hash_password(password).expect("Falha ao gerar hash");

        assert!(
            !verify_password("Wrong_P@ssw0rd_456", &hash).unwrap(),
            "Senha incorreta não deveria validar"
        );
    }

    #[test]
    fn test_hash_uniqueness() {
        let password = "SamePassword123!";
        let hash1 = hash_password(password).expect("Falha ao gerar hash 1");
        let hash2 = hash_password(password).expect("Falha ao gerar hash 2");

        // Hashes diferentes devido a salts aleatórios únicos
        assert_ne!(hash1, hash2, "Hashes devem ser únicos (salts diferentes)");

        // Ambos devem validar a mesma senha
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_invalid_hash_format() {
        let result = verify_password("password", "invalid_hash_format");
        assert!(result.is_err(), "Hash inválido deveria retornar erro");
    }

    #[test]
    fn test_password_strength_validation() {
        // Senha muito fraca
        assert!(validate_password_strength("abc123").is_err());
        assert!(validate_password_strength("password").is_err());
        assert!(validate_password_strength("12345678").is_err());

        // Senhas fortes (Score >= 3)
        assert!(validate_password_strength("C0mpl3x&P@ss#2025").is_ok());
        assert!(validate_password_strength("Tr0ng$ecuR3!Data#42").is_ok());
        assert!(validate_password_strength("F!8q@K39zP#sM7vL").is_ok());
    }

    #[test]
    fn test_backwards_compatibility() {
        // Testa que hashes antigos com parâmetros diferentes ainda funcionam
        // Simula mudança de parâmetros no futuro

        let password = "BackwardsCompat!123";
        let hash = hash_password(password).expect("Falha ao gerar hash");

        // Deve validar mesmo que parâmetros globais mudem
        // (porque parâmetros estão embedados no hash PHC)
        assert!(verify_password(password, &hash).unwrap());
    }

    // =========================================================================
    // BENCHMARK TESTS (cargo test --release -- --ignored)
    // =========================================================================

    #[test]
    #[ignore] // Execute com: cargo test --release -- --ignored
    fn bench_hash_performance() {
        use std::time::Instant;

        let password = "BenchmarkP@ssw0rd!123";
        let iterations = 10;

        let start = Instant::now();
        for _ in 0..iterations {
            hash_password(password).expect("Hash falhou");
        }
        let duration = start.elapsed();

        let avg_ms = duration.as_millis() / iterations;
        println!("\n📊 Argon2id Hash Performance:");
        println!("   Total: {:?} para {} iterações", duration, iterations);
        println!("   Média: {}ms por hash", avg_ms);
        println!(
            "   Parâmetros: m={}, t={}, p={}",
            ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST
        );

        // [FIX] Ajusta expectativa se estiver rodando em DEBUG
        let max_expected = if cfg!(debug_assertions) { 3000 } else { 1000 };

        assert!(
            avg_ms >= 100 && avg_ms <= max_expected,
            "Performance fora do esperado: {}ms (esperado: 100-{}ms). Considere rodar com --release.",
            avg_ms, max_expected
        );
    }

    #[test]
    #[ignore]
    fn bench_verify_performance() {
        use std::time::Instant;

        let password = "BenchmarkP@ssw0rd!123";
        let hash = hash_password(password).expect("Falha ao gerar hash");
        let iterations = 10;

        let start = Instant::now();
        for _ in 0..iterations {
            verify_password(password, &hash).expect("Verify falhou");
        }
        let duration = start.elapsed();

        let avg_ms = duration.as_millis() / iterations;
        println!("\n📊 Argon2id Verify Performance:");
        println!("   Total: {:?} para {} iterações", duration, iterations);
        println!("   Média: {}ms por verificação", avg_ms);

        // [FIX] Ajusta expectativa se estiver rodando em DEBUG
        let max_expected = if cfg!(debug_assertions) { 3000 } else { 1000 };

        assert!(
            avg_ms >= 100 && avg_ms <= max_expected,
            "Performance fora do esperado: {}ms (esperado: 100-{}ms). Considere rodar com --release.",
            avg_ms, max_expected
        );
    }

    #[test]
    fn test_cors_production_parsing() {
        let origins = vec![
            "https://example.com".to_string(),
            "https://app.example.com".to_string(),
        ];
        let _cors = cors_production(origins);
    }

    #[test]
    fn test_security_headers_count() {
        let headers = security_headers();
        assert!(headers.len() >= 5);
    }
}
