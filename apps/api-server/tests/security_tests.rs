mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use once_cell::sync::OnceCell;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

// =============================================================================
// CACHE GLOBAL (Thread-Safe)
// =============================================================================

/// Pool de conexão compartilhado entre todos os testes
static TEST_POOL: OnceCell<Arc<Mutex<PgPool>>> = OnceCell::new();

/// Obtém ou inicializa o pool de conexão
async fn get_pool() -> Arc<Mutex<PgPool>> {
    TEST_POOL
        .get_or_init(|| {
            common::init_test_env();

            let database_url = std::env::var("WS_AUTH_DATABASE_URL")
                .expect("WS_AUTH_DATABASE_URL must be set for tests");

            // Usa o runtime atual (não cria novo)
            let pool = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    PgPool::connect(&database_url)
                        .await
                        .expect("Failed to connect to test database")
                })
            });

            Arc::new(Mutex::new(pool))
        })
        .clone()
}

// =============================================================================
// HELPERS DE SETUP
// =============================================================================

/// Cria a aplicação Axum para testes (sem iniciar servidor TCP)
async fn create_test_app() -> axum::Router {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use waterswamp::*;

    common::init_test_env();

    // Carrega configuração
    let config = config::config::Config::from_env().expect("Failed to load config");

    // Conecta aos bancos de dados
    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Failed to connect to auth database");

    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Failed to connect to logs database");

    // Inicializa Casbin
    let enforcer = casbin_setup::setup_casbin(pool_auth.clone())
        .await
        .expect("Failed to setup Casbin");

    // Chaves JWT
    use jsonwebtoken::{DecodingKey, EncodingKey};
    let encoding_key = EncodingKey::from_ed_pem(config.jwt_private_key.as_bytes())
        .expect("Failed to load JWT private key");

    let decoding_key = DecodingKey::from_ed_pem(config.jwt_public_key.as_bytes())
        .expect("Failed to load JWT public key");

    // Email service
    let email_config =
        email_service::config::EmailConfig::from_env().expect("Failed to load email config");
    let email_service =
        email_service::EmailService::new(email_config).expect("Failed to create email service");

    // Audit service
    let audit_service = handlers::audit_services::AuditService::new(pool_logs.clone());

    // Policy cache
    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    // Cria o AppState
    let app_state = state::AppState {
        enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        encoding_key,
        decoding_key,
        email_service: Arc::new(email_service),
        audit_service,
    };

    // Constrói as rotas
    routes::build(app_state)
}

/// Cria servidor de teste (leve, pode ser chamado múltiplas vezes)
async fn create_test_server() -> TestServer {
    let app = create_test_app().await;
    TestServer::new(app.into_make_service()).expect("Failed to create test server")
}

// =============================================================================
// TESTES DE LOGIN E AUTENTICAÇÃO
// =============================================================================

#[tokio::test]
async fn test_login_success() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    // Limpa usuários de teste anteriores
    common::cleanup_test_users(&*pool).await.unwrap();

    // Cria usuário de teste diretamente no banco
    let (username, _email, password) = common::create_test_user(&*pool).await.unwrap();

    // Libera lock antes de fazer requisição
    drop(pool);

    // Faz login
    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::OK,
        "Login deveria ter sucesso. Body: {}",
        login_response.text()
    );

    let body: serde_json::Value = login_response.json();
    assert!(
        body.get("access_token").is_some(),
        "Response deveria conter access_token"
    );
    assert!(
        body.get("refresh_token").is_some(),
        "Response deveria conter refresh_token"
    );
    assert_eq!(body["token_type"], "Bearer");
}

#[tokio::test]
async fn test_login_fail_wrong_password() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    // Limpa usuários de teste anteriores
    common::cleanup_test_users(&*pool).await.unwrap();

    // Cria usuário de teste
    let (username, _email, _password) = common::create_test_user(&*pool).await.unwrap();

    drop(pool);

    // Tenta login com senha errada
    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": "WrongPassword123!"
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Login com senha errada deveria falhar"
    );

    let body: serde_json::Value = login_response.json();
    assert!(
        body.get("error").is_some(),
        "Response deveria conter mensagem de erro"
    );
}

#[tokio::test]
async fn test_login_fail_nonexistent_user() {
    common::init_test_env();

    let server = create_test_server().await;

    let login_response = server
        .post("/login")
        .json(&json!({
            "username": "usuario_que_nao_existe_12345",
            "password": "qualquersenha"
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Login com usuário inexistente deveria falhar"
    );
}

// =============================================================================
// TESTES DE REFRESH TOKEN ROTATION
// =============================================================================

#[tokio::test]
async fn test_refresh_token_rotation_success() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    common::cleanup_test_users(&*pool).await.unwrap();

    // Cria usuário e faz login
    let (username, _email, password) = common::create_test_user(&*pool).await.unwrap();

    drop(pool);

    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Usa o refresh token para obter novos tokens
    let refresh_response = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        refresh_response.status_code(),
        StatusCode::OK,
        "Refresh token deveria funcionar. Body: {}",
        refresh_response.text()
    );

    let refresh_body: serde_json::Value = refresh_response.json();
    assert!(refresh_body.get("access_token").is_some());
    assert!(refresh_body.get("refresh_token").is_some());

    // Novo refresh token deve ser diferente do antigo
    let new_refresh_token = refresh_body["refresh_token"].as_str().unwrap();
    assert_ne!(
        refresh_token, new_refresh_token,
        "Novo refresh token deve ser diferente (rotation)"
    );
}

#[tokio::test]
async fn test_refresh_token_revoked_after_use() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    common::cleanup_test_users(&*pool).await.unwrap();

    // Cria usuário e faz login
    let (username, _email, password) = common::create_test_user(&*pool).await.unwrap();

    drop(pool);

    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap().to_string();

    // Usa o refresh token uma vez (deve funcionar)
    let first_refresh = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(first_refresh.status_code(), StatusCode::OK);

    // Tenta usar o mesmo refresh token novamente (deve falhar)
    let second_refresh = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        second_refresh.status_code(),
        StatusCode::UNAUTHORIZED,
        "Token já usado não deveria funcionar novamente"
    );
}

#[tokio::test]
async fn test_refresh_token_theft_detection_revokes_family() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    common::cleanup_test_users(&*pool).await.unwrap();

    // Cria usuário e faz login
    let (username, _email, password) = common::create_test_user(&*pool).await.unwrap();

    drop(pool);

    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let token1 = login_body["refresh_token"].as_str().unwrap().to_string();

    // Rotaciona para obter token2
    let refresh1 = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token1
        }))
        .await;

    let refresh1_body: serde_json::Value = refresh1.json();
    let token2 = refresh1_body["refresh_token"].as_str().unwrap().to_string();

    // Rotaciona novamente para obter token3
    let refresh2 = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token2
        }))
        .await;

    let refresh2_body: serde_json::Value = refresh2.json();
    let token3 = refresh2_body["refresh_token"].as_str().unwrap().to_string();

    // Simula roubo: tenta usar token1 (já foi usado)
    // Isso deve revogar toda a família de tokens
    let theft_attempt = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token1
        }))
        .await;

    assert_eq!(
        theft_attempt.status_code(),
        StatusCode::UNAUTHORIZED,
        "Token já usado deveria falhar"
    );

    // Verifica que token3 (mais recente) também foi revogado
    let token3_attempt = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token3
        }))
        .await;

    assert_eq!(
        token3_attempt.status_code(),
        StatusCode::UNAUTHORIZED,
        "Token3 deveria ter sido revogado após detecção de roubo"
    );
}

// =============================================================================
// TESTES DE REGISTRO
// =============================================================================

#[tokio::test]
async fn test_register_success() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    common::cleanup_test_users(&*pool).await.unwrap();

    drop(pool);

    let (username, _email, password) = common::register_test_user(&server).await.unwrap();

    // Verifica que pode fazer login com as credenciais
    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    assert_eq!(login_response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_duplicate_username() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    common::cleanup_test_users(&*pool).await.unwrap();

    drop(pool);

    // Registra primeiro usuário
    let (username, _email, _password) = common::register_test_user(&server).await.unwrap();

    // Tenta registrar com mesmo username
    let response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": "outro_email@test.com",
            "password": "OutraS3nh@123"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CONFLICT,
        "Registro com username duplicado deveria falhar"
    );
}

#[tokio::test]
async fn test_register_weak_password() {
    common::init_test_env();

    let server = create_test_server().await;

    let counter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let response = server
        .post("/register")
        .json(&json!({
            "username": format!("test_user_{}", counter),
            "email": format!("test_{}@test.com", counter),
            "password": "weak"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::BAD_REQUEST,
        "Registro com senha fraca deveria falhar"
    );
}

// =============================================================================
// TESTES DE LOGOUT
// =============================================================================

#[tokio::test]
async fn test_logout_success() {
    common::init_test_env();

    let server = create_test_server().await;
    let pool_mutex = get_pool().await;
    let pool = pool_mutex.lock().await;

    common::cleanup_test_users(&*pool).await.unwrap();

    // Cria usuário e faz login
    let (username, _email, password) = common::create_test_user(&*pool).await.unwrap();

    drop(pool);

    let login_response = server
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Faz logout
    let logout_response = server
        .post("/logout")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(logout_response.status_code(), StatusCode::OK);

    // Tenta usar o refresh token após logout (deve falhar)
    let refresh_after_logout = server
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        refresh_after_logout.status_code(),
        StatusCode::UNAUTHORIZED,
        "Token revogado não deveria funcionar"
    );
}
