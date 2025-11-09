use axum_test::TestServer;
use http::{header::AUTHORIZATION, HeaderValue};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use waterswamp::{
    casbin_setup::setup_casbin, config::Config, routes::build_router, state::AppState,
};

// =============================================================================
// HELPERS
// =============================================================================

async fn create_test_server() -> TestServer {
    let config = Config::from_env().expect("Falha ao carregar configuração");

    let pool_auth = sqlx::PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao banco de autenticação");

    let pool_logs = sqlx::PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao banco de logs");

    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao configurar Casbin");

    let secret = config.jwt_secret;
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

    let policy_cache = Arc::new(RwLock::new(std::collections::HashMap::new()));

    let app_state = AppState {
        enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        encoding_key,
        decoding_key,
    };

    let app = build_router(app_state);
    TestServer::new(app).expect("Falha ao criar servidor de teste")
}

async fn test_login(server: &TestServer, username: &str, password: &str) -> String {
    let response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    if response.status_code() != 200 {
        panic!("Login falhou para '{}'", username);
    }

    let body: serde_json::Value = response.json();
    body["access_token"]
        .as_str()
        .expect("access_token ausente")
        .to_string()
}

// Helper para criar header de autorização
fn auth_header(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("Bearer {}", token))
        .expect("Falha ao criar header de autorização")
}

// =============================================================================
// TESTES
// =============================================================================

#[tokio::test]
async fn test_admin_adicionar_politica_duplicada_retorna_200() {
    let server = create_test_server().await;
    let token = test_login(&server, "alice", "password123").await;

    // Primeira adição
    let response1 = server
        .post("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&token))
        .json(&json!({
            "subject": "bob",
            "object": "/api/test",
            "action": "POST"
        }))
        .await;

    assert_eq!(response1.status_code(), 200);

    // Segunda adição (duplicada)
    let response2 = server
        .post("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&token))
        .json(&json!({
            "subject": "bob",
            "object": "/api/test",
            "action": "POST"
        }))
        .await;

    assert_eq!(response2.status_code(), 200);
}

#[tokio::test]
async fn test_admin_adicionar_politica_usuario_inexistente_retorna_404() {
    let server = create_test_server().await;
    let token = test_login(&server, "alice", "password123").await;

    let response = server
        .post("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&token))
        .json(&json!({
            "subject": "usuario_que_nao_existe_123456",
            "object": "/api/test",
            "action": "GET"
        }))
        .await;

    assert_eq!(response.status_code(), 404);

    let body: serde_json::Value = response.json();
    assert!(
        body["error"].as_str().unwrap().contains("não encontrado")
            || body["error"].as_str().unwrap().contains("inexistente")
    );
}

#[tokio::test]
async fn test_admin_payload_invalido_retorna_400() {
    let server = create_test_server().await;
    let token = test_login(&server, "alice", "password123").await;

    let response = server
        .post("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&token))
        .json(&json!({
            "subject": "",
            "object": "/api/test",
            "action": "GET"
        }))
        .await;

    assert_eq!(response.status_code(), 400);
}

#[tokio::test]
async fn test_admin_remover_politica_inexistente_retorna_404() {
    let server = create_test_server().await;
    let token = test_login(&server, "alice", "password123").await;

    let response = server
        .delete("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&token))
        .json(&json!({
            "subject": "usuario_fantasma",
            "object": "/api/rota_inexistente",
            "action": "DELETE"
        }))
        .await;

    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_fluxo_dinamico_de_permissoes() {
    let server = create_test_server().await;

    println!("\n🧪 Fluxo dinâmico de permissões");

    let admin_token = test_login(&server, "alice", "password123").await;
    let user_token = test_login(&server, "bob", "password123").await;

    // Bob tenta acessar (deve falhar)
    let response = server
        .get("/admin/dashboard")
        .add_header(AUTHORIZATION, auth_header(&user_token))
        .await;
    assert_eq!(response.status_code(), 403);

    // Admin adiciona permissão
    let add_response = server
        .post("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&admin_token))
        .json(&json!({
            "subject": "bob",
            "object": "/admin/dashboard",
            "action": "GET"
        }))
        .await;
    assert_eq!(add_response.status_code(), 200);

    // Bob tenta novamente (deve funcionar)
    let response2 = server
        .get("/admin/dashboard")
        .add_header(AUTHORIZATION, auth_header(&user_token))
        .await;
    assert_eq!(response2.status_code(), 200);

    // Admin remove permissão
    let remove_response = server
        .delete("/api/admin/policies")
        .add_header(AUTHORIZATION, auth_header(&admin_token))
        .json(&json!({
            "subject": "bob",
            "object": "/admin/dashboard",
            "action": "GET"
        }))
        .await;
    assert_eq!(remove_response.status_code(), 200);

    // Bob tenta novamente (deve falhar)
    let response3 = server
        .get("/admin/dashboard")
        .add_header(AUTHORIZATION, auth_header(&user_token))
        .await;
    assert_eq!(response3.status_code(), 403);

    println!("✅ Teste passou!");
}
