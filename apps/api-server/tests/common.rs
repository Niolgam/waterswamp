use axum_test::TestServer;
use core_services::jwt::JwtService;
use domain::models::{Claims, TokenType};
use domain::ports::EmailServicePort;
use email_service::{EmailSender, MockEmailService};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use waterswamp::build_application_state;
use waterswamp::infra::casbin_setup::setup_casbin;
use waterswamp::infra::config::Config;
use waterswamp::routes::build;
use waterswamp::state::AppState;

pub struct TestApp {
    pub api: TestServer,
    pub db_auth: PgPool,
    pub db_logs: PgPool,
    pub admin_token: String,
    pub user_token: String,
    pub email_service: Arc<MockEmailService>,
}

pub async fn spawn_app() -> TestApp {
    dotenvy::dotenv().ok();
    std::env::set_var("DISABLE_RATE_LIMIT", "true");

    let config = Config::from_env().expect("Falha ao carregar config de teste");

    let pool_auth = PgPool::connect(&config.main_db)
        .await
        .expect("Falha main_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha logs_db");

    sqlx::migrate!("../../crates/persistence/migrations_auth")
        .run(&pool_auth)
        .await
        .expect("Migration auth");
    sqlx::migrate!("../../crates/persistence/migrations_main")
        .run(&pool_auth)
        .await
        .expect("Migration main");
    sqlx::migrate!("../../crates/persistence/migrations_logs")
        .run(&pool_logs)
        .await
        .expect("Migration logs");

    // Setup Casbin com retry para concorrência
    let enforcer = match setup_casbin(pool_auth.clone()).await {
        Ok(e) => e,
        Err(_) => setup_casbin(pool_auth.clone())
            .await
            .expect("Casbin falhou no retry"),
    };

    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");

    let jwt_service = Arc::new(JwtService::new(private_pem, public_pem).expect("JwtService"));

    // MOCK: Aqui injetamos o Mock em vez do serviço real
    let email_service = Arc::new(MockEmailService::new());

    // Prepara os traits para injeção
    let email_service_legacy: Arc<dyn EmailSender + Send + Sync> = email_service.clone();
    let email_service_port: Arc<dyn EmailServicePort> = email_service.clone();

    // --- WIRING COM FACTORY ---
    let app_state = build_application_state(
        Arc::new(config),
        pool_auth.clone(),
        pool_logs.clone(),
        enforcer,
        jwt_service,
        email_service_legacy,
        email_service_port,
    );

    let app = build(app_state);
    let api = TestServer::new(app).unwrap();

    // Seed opcional (ignora erros se já existir)
    let alice_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_optional(&pool_auth)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_optional(&pool_auth)
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    TestApp {
        api,
        db_auth: pool_auth,
        db_logs: pool_logs,
        admin_token: generate_test_token(alice_id),
        user_token: generate_test_token(bob_id),
        email_service,
    }
}

/// Helper para criar AppState de teste isolado (sem subir servidor HTTP completo)
pub async fn create_test_app_state() -> AppState {
    dotenvy::dotenv().ok();
    std::env::set_var("DISABLE_RATE_LIMIT", "true");

    let config = Config::from_env().expect("Falha ao carregar config");
    let pool_auth = PgPool::connect(&config.main_db).await.expect("Auth DB");
    let pool_logs = PgPool::connect(&config.logs_db).await.expect("Logs DB");

    let enforcer = setup_casbin(pool_auth.clone()).await.expect("Casbin");

    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");
    let jwt_service = Arc::new(JwtService::new(private_pem, public_pem).expect("JWT"));

    let email_service = Arc::new(MockEmailService::new());

    build_application_state(
        Arc::new(config),
        pool_auth,
        pool_logs,
        enforcer,
        jwt_service,
        email_service.clone(), // Legacy
        email_service.clone(), // Port
    )
}

// ... (Resto do arquivo: create_api_auth_test_server, generate_test_token, etc.) ...
pub async fn create_api_auth_test_server(state: AppState) -> TestServer {
    use axum::Router;
    use waterswamp::api;
    let app = Router::new()
        .nest("/auth", api::auth::router())
        .with_state(state);
    TestServer::new(app).unwrap()
}

pub fn generate_test_token(user_id: Uuid) -> String {
    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let encoding_key = EncodingKey::from_ed_pem(private_pem).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
        username: "testuser".to_string(),
        exp: now + 3600,
        iat: now,
        token_type: TokenType::Access,
    };
    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &encoding_key).unwrap()
}

static INIT: Once = Once::new();
pub fn init_test_env() {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
        std::env::set_var("DISABLE_RATE_LIMIT", "true");
        std::env::set_var("RUST_LOG", "info,waterswamp=debug");
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    });
}

pub async fn cleanup_test_users(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'test_user_%')").execute(pool).await?;
    sqlx::query("DELETE FROM email_verification_tokens WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'test_user_%')").execute(pool).await?;
    sqlx::query("DELETE FROM mfa_setup_tokens WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'test_user_%')").execute(pool).await?;
    sqlx::query("DELETE FROM users WHERE username LIKE 'test_user_%'")
        .execute(pool)
        .await?;
    Ok(())
}

// Helpers para criar usuário no banco (bypass API)
pub async fn create_test_user(
    pool: &PgPool,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    use core_services::security::hash_password;
    let counter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_user_{}", counter);
    let email = format!("test_{}@test.com", counter);
    let password = "SecureP@ssw0rd!123".to_string();
    let hash = tokio::task::spawn_blocking({
        let p = password.clone();
        move || hash_password(&p)
    })
    .await??;

    sqlx::query("INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)")
        .bind(&username)
        .bind(&email)
        .bind(&hash)
        .execute(pool)
        .await?;

    Ok((username, email, password))
}

// Helper para registrar via API
pub async fn register_test_user(
    server: &axum_test::TestServer,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let counter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let username = format!("test_user_{}", counter);
    let email = format!("test_{}@test.com", counter);
    let password = "SecureP@ssw0rd!123".to_string();

    let response = server
        .post("/register")
        .json(&serde_json::json!({
            "username": username, "email": email, "password": password
        }))
        .await;

    if response.status_code() != 201 {
        return Err(format!("Falha ao registrar: {}", response.text()).into());
    }
    Ok((username, email, password))
}

pub fn generate_reset_token(user_id: Uuid) -> String {
    generate_custom_token(user_id, TokenType::PasswordReset, 900)
}

pub fn generate_custom_token(
    user_id: Uuid,
    token_type: TokenType,
    expires_in_seconds: i64,
) -> String {
    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let encoding_key = EncodingKey::from_ed_pem(private_pem).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
        username: "testuser".to_string(),
        exp: now + expires_in_seconds,
        iat: now,
        token_type,
    };
    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &encoding_key).unwrap()
}

pub fn generate_expired_token(user_id: Uuid, token_type: TokenType) -> String {
    generate_custom_token(user_id, token_type, -60)
}
