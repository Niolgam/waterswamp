use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use waterswamp::{
    casbin_setup::setup_casbin, config::Config, routes::build_router, state::AppState,
};

async fn create_test_server() -> TestServer {
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

#[tokio::test]
async fn test_register_success() {
    let server = create_test_server().await;

    // Usar um username único para evitar conflitos
    let unique_username = format!("user_{}", uuid::Uuid::new_v4());

    let response = server
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "password": "S3nh@Forte123"
        }))
        .await;

    // Debug: ver o corpo da resposta se falhar
    if response.status_code() != 200 {
        let body: serde_json::Value = response.json();
        println!("Erro no registro: {:?}", body);
        panic!("Registro falhou com status {}", response.status_code());
    }

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["access_token"].is_string(), "access_token ausente");
    assert!(body["refresh_token"].is_string(), "refresh_token ausente");
}

#[tokio::test]
async fn test_register_username_taken() {
    let server = create_test_server().await;

    let username = format!("duplicated_{}", uuid::Uuid::new_v4());

    // Primeiro registro (deve funcionar)
    let response1 = server
        .post("/register")
        .json(&json!({
            "username": username,
            "password": "S3nh@Forte123"
        }))
        .await;

    if response1.status_code() != 200 {
        let body: serde_json::Value = response1.json();
        println!("Erro no primeiro registro: {:?}", body);
        panic!("Primeiro registro falhou");
    }

    response1.assert_status_ok();

    // Segundo registro (deve falhar com 400 ou 409)
    let response2 = server
        .post("/register")
        .json(&json!({
            "username": username,
            "password": "Outr@Senh@456"
        }))
        .await;

    // Aceitar tanto 400 quanto 409
    assert!(
        response2.status_code() == 400 || response2.status_code() == 409,
        "Esperado 400 ou 409, recebido {}",
        response2.status_code()
    );
}

#[tokio::test]
async fn test_register_weak_password() {
    let server = create_test_server().await;

    let response = server
        .post("/register")
        .json(&json!({
            "username": format!("user_{}", uuid::Uuid::new_v4()),
            "password": "senha123" // Sem maiúscula e caractere especial
        }))
        .await;

    assert_eq!(
        response.status_code(),
        400,
        "Senha fraca deveria retornar 400"
    );
}

#[tokio::test]
async fn test_register_short_password() {
    let server = create_test_server().await;

    let response = server
        .post("/register")
        .json(&json!({
            "username": format!("user_{}", uuid::Uuid::new_v4()),
            "password": "Abc1!" // Muito curta
        }))
        .await;

    assert_eq!(response.status_code(), 400);
}

#[tokio::test]
async fn test_register_short_username() {
    let server = create_test_server().await;

    let response = server
        .post("/register")
        .json(&json!({
            "username": "ab", // Menos de 3 caracteres
            "password": "S3nh@Forte123"
        }))
        .await;

    assert_eq!(response.status_code(), 400);
}
