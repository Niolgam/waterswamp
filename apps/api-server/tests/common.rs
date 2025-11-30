use axum_test::TestServer;
use core_services::jwt::JwtService;
use domain::models::{Claims, TokenType};
use email_service::{EmailSender, MockEmailService}; // IMPORTAR do crate
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use waterswamp::handlers::audit_services::AuditService;
use waterswamp::infra::casbin_setup::setup_casbin;
use waterswamp::infra::config::Config;
use waterswamp::routes::build;
use waterswamp::state::AppState;

// REMOVER: Não defina MockEmailService aqui!
// REMOVER: Não defina MockEmail aqui!

// ============================================================================
// TestApp
// ============================================================================

#[allow(dead_code)]
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

    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db");

    sqlx::migrate!("../../crates/persistence/migrations_auth")
        .run(&pool_auth)
        .await
        .expect("Falha ao rodar migrations no auth_db");

    sqlx::migrate!("../../crates/persistence/migrations_logs")
        .run(&pool_logs)
        .await
        .expect("Falha ao rodar migrations no logs_db");

    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");

    let jwt_service = JwtService::new(private_pem, public_pem).expect("Falha ao criar JwtService");
    let email_service = Arc::new(MockEmailService::new()); // Agora vem do crate
    let audit_service = AuditService::new(pool_logs.clone());

    let app_state = AppState {
        enforcer,
        policy_cache: Arc::new(RwLock::new(HashMap::new())),
        db_pool_auth: pool_auth.clone(),
        db_pool_logs: pool_logs.clone(),
        jwt_service,
        email_service: email_service.clone(),
        audit_service,
    };

    let app = build(app_state);
    let api = TestServer::new(app).unwrap();

    let alice_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&pool_auth)
        .await
        .expect("Alice não encontrada no seed");

    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&pool_auth)
        .await
        .expect("Bob não encontrado no seed");

    TestApp {
        api,
        db_auth: pool_auth,
        db_logs: pool_logs,
        admin_token: generate_test_token(alice_id),
        user_token: generate_test_token(bob_id),
        email_service,
    }
}

/// Helper para criar AppState de teste (reutiliza lógica do spawn_app)
pub async fn create_test_app_state() -> AppState {
    dotenvy::dotenv().ok();
    std::env::set_var("DISABLE_RATE_LIMIT", "true");

    let config = Config::from_env().expect("Falha ao carregar config de teste");

    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db");

    sqlx::migrate!("../../crates/persistence/migrations_auth")
        .run(&pool_auth)
        .await
        .expect("Falha ao rodar migrations no auth_db");

    sqlx::migrate!("../../crates/persistence/migrations_logs")
        .run(&pool_logs)
        .await
        .expect("Falha ao rodar migrations no logs_db");

    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");

    let jwt_service = JwtService::new(private_pem, public_pem).expect("Falha ao criar JwtService");
    let email_service = Arc::new(MockEmailService::new()); // Agora vem do crate
    let audit_service = AuditService::new(pool_logs.clone());

    AppState {
        enforcer,
        policy_cache: Arc::new(RwLock::new(HashMap::new())),
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        jwt_service,
        email_service,
        audit_service,
    }
}

/// Helper para criar TestServer com as novas rotas de api::auth
pub async fn create_api_auth_test_server(state: AppState) -> TestServer {
    use axum::Router;
    use waterswamp::api;

    let app = Router::new()
        .nest("/auth", api::auth::router())
        .with_state(state);

    TestServer::new(app).unwrap()
}

// ... resto das funções (generate_test_token, init_test_env, etc) ...

pub fn generate_test_token(user_id: Uuid) -> String {
    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let encoding_key = EncodingKey::from_ed_pem(private_pem).unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
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
    sqlx::query(
        r#"
        DELETE FROM refresh_tokens 
        WHERE user_id IN (
            SELECT id FROM users WHERE username LIKE 'test_user_%'
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM email_verification_tokens 
        WHERE user_id IN (
            SELECT id FROM users WHERE username LIKE 'test_user_%'
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM mfa_setup_tokens 
        WHERE user_id IN (
            SELECT id FROM users WHERE username LIKE 'test_user_%'
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM users 
        WHERE username LIKE 'test_user_%'
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

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

    let password_clone = password.clone();
    let password_hash =
        tokio::task::spawn_blocking(move || hash_password(&password_clone)).await??;

    sqlx::query(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .execute(pool)
    .await?;

    Ok((username, email, password))
}

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
            "username": username,
            "email": email,
            "password": password
        }))
        .await;

    if response.status_code() != 201 {
        return Err(format!(
            "Falha ao registrar usuário: status={}, body={}",
            response.status_code(),
            response.text()
        )
        .into());
    }

    Ok((username, email, password))
}

pub async fn create_unique_test_user(
    pool: &PgPool,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    use core_services::security::hash_password;
    use uuid::Uuid;

    let unique_id = Uuid::new_v4();
    let username = format!("test_user_{}", unique_id);
    let email = format!("test_{}@test.com", unique_id);
    let password = "SecureP@ssw0rd!123".to_string();

    let password_clone = password.clone();
    let password_hash =
        tokio::task::spawn_blocking(move || hash_password(&password_clone)).await??;

    sqlx::query(
        r#"
        INSERT INTO users (username, email, password_hash, email_verified)
        VALUES ($1, $2, $3, TRUE)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .execute(pool)
    .await?;

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
