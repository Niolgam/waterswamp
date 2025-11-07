use axum_test::TestServer;
use http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use waterswamp::{
    casbin_setup::setup_casbin, config::Config, models::Claims, routes::build_router,
    state::AppState,
};

// --- Helpers ---

async fn spawn_app() -> TestServer {
    dotenvy::dotenv().ok();

    std::env::set_var("DISABLE_RATE_LIMIT", "true");
    let config = Config::from_env().expect("Falha ao carregar config");

    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db de teste");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db de teste");

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

async fn login_token(server: &TestServer, user: &str) -> String {
    let response = server
        .post("/login")
        .json(&json!({ "username": user, "password": "password123" }))
        .await;

    // --- ADICIONE ESTE BLOCO DE DEBUG ---
    if response.status_code() != http::StatusCode::OK {
        println!("!!! ERRO NO LOGIN (DEBUG) !!!");
        println!("Usuário: {}", user);
        println!("Status Code: {}", response.status_code());
        println!("Headers: {:#?}", response.headers());
        println!("Corpo da resposta: '{}'", response.text());
        panic!("Falha ao fazer login no teste para o usuário '{}'", user);
    }
    // ------------------------------------

    let body: Value = response.json();
    format!("Bearer {}", body["token"].as_str().unwrap())
}

fn decode_token(bearer_token: &str) -> Claims {
    let token = bearer_token.strip_prefix("Bearer ").unwrap();
    let secret = std::env::var("WS_JWT_SECRET").expect("JWT_SECRET must be set");
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &jsonwebtoken::Validation::default())
        .unwrap()
        .claims
}

// --- TESTES ---

#[tokio::test]
async fn test_fluxo_dinamico_de_permissoes() {
    let server = spawn_app().await;
    let token_alice = login_token(&server, "alice").await;
    let token_bob = login_token(&server, "bob").await;

    let bob_claims = decode_token(&token_bob);
    let bob_uuid = bob_claims.sub.to_string();

    // 1. Bob tenta acessar admin -> DEVE FALHAR (403)
    server
        .get("/admin/dashboard")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_forbidden();

    // 2. Alice adiciona permissão para Bob (POST /api/admin/policies)
    server
        .post("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": bob_uuid,
            "object": "/admin/dashboard",
            "action": "GET"
        }))
        .await
        .assert_status(StatusCode::CREATED);

    // 3. Bob tenta acessar admin de novo -> DEVE FUNCIONAR (200)
    server
        .get("/admin/dashboard")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_ok();

    // 4. Alice remove a permissão de Bob (DELETE /api/admin/policies)
    server
        .delete("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": bob_uuid,
            "object": "/admin/dashboard",
            "action": "GET"
        }))
        .await
        .assert_status_no_content();

    // 5. Bob tenta acessar admin mais uma vez -> DEVE FALHAR (403)
    server
        .get("/admin/dashboard")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_forbidden();
}

#[tokio::test]
async fn test_admin_remover_politica_inexistente_retorna_404() {
    let server = spawn_app().await;
    let token_alice = login_token(&server, "alice").await;

    // Tenta remover uma regra com usuário que não existe
    // Deve retornar 404 Not Found (não 401 Unauthorized)
    server
        .delete("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": "usuario_fantasma",
            "object": "/recurso/inexistente",
            "action": "GET"
        }))
        .await
        .assert_status_not_found();
}

#[tokio::test]
async fn test_admin_adicionar_politica_duplicada_retorna_200() {
    let server = spawn_app().await;
    let token_alice = login_token(&server, "alice").await;

    let bob_claims = decode_token(&login_token(&server, "bob").await);
    let bob_uuid = bob_claims.sub.to_string();

    // 1. Adiciona uma regra nova (201 Created)
    server
        .post("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": bob_uuid,
            "object": "/data",
            "action": "READ"
        }))
        .await
        .assert_status(StatusCode::CREATED);

    // 2. Tenta adicionar a MESMA regra de novo (Deve retornar 200 OK)
    server
        .post("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": bob_uuid,
            "object": "/data",
            "action": "READ"
        }))
        .await
        .assert_status_ok();

    // Limpar: Remover a política criada
    server
        .delete("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": bob_uuid,
            "object": "/data",
            "action": "READ"
        }))
        .await
        .assert_status_no_content();
}

#[tokio::test]
async fn test_admin_payload_invalido_retorna_400() {
    let server = spawn_app().await;
    let token_alice = login_token(&server, "alice").await;

    // Envia payload com campos vazios (deve falhar na validação)
    server
        .post("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": "",  // Inválido (min length 1)
            "object": "",   // Inválido
            "action": ""    // Inválido
        }))
        .await
        .assert_status_bad_request();
}

#[tokio::test]
async fn test_admin_adicionar_politica_usuario_inexistente_retorna_404() {
    let server = spawn_app().await;
    let token_alice = login_token(&server, "alice").await;

    // Tenta adicionar política para usuário que não existe
    server
        .post("/api/admin/policies")
        .add_header("Authorization", &token_alice)
        .json(&json!({
            "subject": "usuario_que_nao_existe",
            "object": "/algum/recurso",
            "action": "GET"
        }))
        .await
        .assert_status_not_found();
}
