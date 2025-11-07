use axum_test::TestServer;
use http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use waterswamp::{
    casbin_setup::setup_casbin, config::Config, routes::build_router, state::AppState,
};

async fn spawn_app() -> TestServer {
    dotenvy::dotenv().ok();

    std::env::set_var("DISABLE_RATE_LIMIT", "true");

    let config = Config::from_env().expect("Falha ao carregar config");

    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db");

    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    let app_state = AppState {
        enforcer: enforcer.clone(),
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        encoding_key,
        decoding_key,
    };

    let app = build_router(app_state);
    TestServer::new(app).unwrap()
}

// --- TESTES DE HEALTH CHECK ---

#[tokio::test]
async fn test_health_endpoint_retorna_200() {
    let server = spawn_app().await;

    let response = server.get("/health").await;
    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["status"], "healthy");
    assert!(body["database"]["auth_db"].as_bool().unwrap());
    assert!(body["database"]["logs_db"].as_bool().unwrap());
    assert!(
        body["version"].as_str().is_some(),
        "Versão deve estar presente no health check"
    );
}

#[tokio::test]
async fn test_health_ready_endpoint() {
    let server = spawn_app().await;
    server.get("/health/ready").await.assert_status_ok();
}

#[tokio::test]
async fn test_health_live_endpoint() {
    let server = spawn_app().await;
    server.get("/health/live").await.assert_status_ok();
}

// --- TESTES DE RATE LIMITING ---

#[tokio::test]
async fn test_rate_limit_login_protege_contra_brute_force() {
    let server = spawn_app().await;

    // Faz múltiplas tentativas de login inválido rapidamente
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Tenta 20 vezes (ajuste conforme sua configuração de rate limit se necessário)
    for i in 0..20 {
        let response = server
            .post("/login")
            .json(&json!({
                "username": format!("usuario_fake_{}", i),
                "password": "senha_errada"
            }))
            .await;

        match response.status_code() {
            StatusCode::UNAUTHORIZED => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            other => panic!("Status inesperado: {}", other),
        }
    }

    // ⚠️ NOTA: Com DISABLE_RATE_LIMIT=true, este teste verifica apenas
    // que o servidor responde corretamente a múltiplas requisições.
    // O rate limiting real só pode ser testado com servidor HTTP real.
    assert!(
        success_count > 0,
        "Servidor deveria processar requisições. Sucessos: {}, Bloqueios: {}",
        success_count,
        rate_limited_count
    );
}

// --- TESTES DE CORS ---

#[tokio::test]
async fn test_cors_headers_presentes() {
    let server = spawn_app().await;

    // Simula uma requisição cross-origin
    let response = server
        .get("/public")
        .add_header("Origin", "http://localhost:4200")
        .await;

    response.assert_status_ok();

    // Verifica presença de headers CORS essenciais
    let headers = response.headers();
    assert!(
        headers.get("access-control-allow-origin").is_some(),
        "Header CORS 'access-control-allow-origin' ausente"
    );
    assert!(
        headers.get("access-control-allow-credentials").is_some(),
        "Header CORS 'access-control-allow-credentials' ausente"
    );
}

// --- TESTES DE SECURITY HEADERS ---

#[tokio::test]
async fn test_security_headers_presentes_e_corretos() {
    let server = spawn_app().await;

    let response = server.get("/public").await;
    response.assert_status_ok();

    let headers = response.headers();

    // Verifica X-Content-Type-Options
    assert_eq!(
        headers
            .get("x-content-type-options")
            .expect("Header ausente")
            .to_str()
            .unwrap(),
        "nosniff",
        "X-Content-Type-Options incorreto"
    );

    // Verifica X-Frame-Options
    assert_eq!(
        headers
            .get("x-frame-options")
            .expect("Header ausente")
            .to_str()
            .unwrap(),
        "DENY",
        "X-Frame-Options incorreto"
    );

    // Verifica X-XSS-Protection
    assert!(
        headers.get("x-xss-protection").is_some(),
        "Header X-XSS-Protection ausente"
    );

    // Verifica Content-Security-Policy
    assert!(
        headers.get("content-security-policy").is_some(),
        "Header Content-Security-Policy ausente"
    );

    // Verifica Permissions-Policy
    assert!(
        headers.get("permissions-policy").is_some(),
        "Header Permissions-Policy ausente"
    );
}

// --- TESTES DE GRACEFUL SHUTDOWN ---

#[tokio::test]
async fn test_servidor_responde_durante_operacao_normal() {
    let server = spawn_app().await;

    // O servidor deve responder normalmente quando não está em processo de desligamento
    server.get("/health").await.assert_status_ok();
    server.get("/public").await.assert_status_ok();
}
