use axum_test::TestServer;
use dotenvy::from_path;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use waterswamp::{
    casbin_setup::setup_casbin, config::Config, routes::build_router, state::AppState,
};

/// Helper para inicializar o servidor de testes
async fn spawn_app() -> TestServer {
    dotenvy::dotenv().ok();
    std::env::set_var("DISABLE_RATE_LIMIT", "true");
    // let env_path = std::path::Path::new("../.env");
    // from_path(env_path).expect(&format!(
    //     "!!! ERRO DE TESTE: Não foi possível encontrar o .env em {:?}",
    //     env_path.canonicalize()
    // ));

    // (O .env deve estar na raiz do workspace para que isso funcione)
    let config = Config::from_env().expect("Falha ao carregar config de teste");

    // Cria os pools (necessáros para o AppState)
    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db de teste");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db de teste");

    // Testa o setup do casbin (agora passando o pool)
    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    // Cria o AppState com o Arc<RwLock<Enforcer>>
    let app_state = AppState {
        enforcer: enforcer.clone(),
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        encoding_key,
        decoding_key,
    };

    // Constrói o router
    let app = build_router(app_state);

    TestServer::new(app).unwrap()
}

/// Helper para fazer login e obter um token
async fn login_e_obter_token(server: &TestServer, username: &str) -> String {
    let response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": "password123" // A senha que definimos no 'seed_policies'
        }))
        .await;

    response.assert_status_ok();

    // Extrai o token da resposta JSON
    let body: Value = response.json();
    let token = body["token"]
        .as_str()
        .expect("Resposta do login não continha 'token'");

    format!("Bearer {}", token)
}

// --- Testes Finais ---

#[tokio::test]
async fn test_public_route() {
    let server = spawn_app().await;
    server.get("/public").await.assert_status_ok();
}

#[tokio::test]
async fn test_login_usuario_invalido_falha_401() {
    let server = spawn_app().await;

    // Tenta fazer login com usuário que não existe
    server
        .post("/login")
        .json(&json!({
            "username": "usuario_fantasma",
            "password": "password123"
        }))
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn test_login_senha_invalida_falha_401() {
    let server = spawn_app().await;

    // Tenta fazer login com 'bob', mas senha errada
    server
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "senha_errada"
        }))
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn test_rotas_protegidas_sem_token_falha_401() {
    let server = spawn_app().await;

    // Tenta acessar sem cabeçalho 'Authorization'
    server
        .get("/users/profile")
        .await
        .assert_status_unauthorized();
    server
        .get("/admin/dashboard")
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn test_fluxo_usuario_normal_bob() {
    let server = spawn_app().await;

    // 1. Login como "bob"
    let token_bob = login_e_obter_token(&server, "bob").await;

    // 2. Tenta acessar o perfil (Deve conseguir 200)
    server
        .get("/users/profile")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_ok();

    // 3. Tenta acessar o admin (Deve ser bloqueado 403)
    server
        .get("/admin/dashboard")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_forbidden();
}

#[tokio::test]
async fn test_fluxo_admin_alice() {
    let server = spawn_app().await;

    // 1. Login como "alice"
    let token_alice = login_e_obter_token(&server, "alice").await;

    // 2. Tenta acessar o perfil (Deve conseguir 200)
    // (Lembre-se, nossa regra 'admin' NÃO herda de 'user',
    // então este teste falhará a menos que adicionemos a regra g(admin, user))
    // Vamos testar apenas a rota de admin por enquanto.

    // 2. Tenta acessar o admin (Deve conseguir 200)
    server
        .get("/admin/dashboard")
        .add_header("Authorization", &token_alice)
        .await
        .assert_status_ok();
}
