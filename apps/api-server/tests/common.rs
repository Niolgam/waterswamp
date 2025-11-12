use axum::http::StatusCode;
use axum_test::{TestResponse, TestServer};
use core_services::security::{hash_password, validate_password_strength};
use domain::models::{Claims, LoginPayload, LoginResponse, RegisterPayload, TokenType};
use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;
// --- CORREÇÃO: Caminho de importação completo ---
use persistence::repositories::user_repository::UserRepository;
use sqlx::{Executor, PgPool};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use waterswamp::routes::build_router;
// <-- 'Connection' removida (não usada)
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
// --- CORREÇÃO: Imports do crate 'waterswamp' ---
use waterswamp::casbin_setup::setup_casbin;
use waterswamp::{config::Config, state::AppState};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter = "info,waterswamp=debug,tower_http=debug".to_string();
    let test_filter = std::env::var("TEST_LOG").unwrap_or(default_filter);
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(test_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();
});

pub struct TestApp {
    pub api: TestServer,
    pub db_auth: PgPool,
    pub db_logs: PgPool,
    // Tokens pré-gerados para testes de autorização
    pub admin_token: String,
    pub user_token: String,
}

pub async fn spawn_app() -> TestApp {
    dotenvy::dotenv().ok();
    std::env::set_var("DISABLE_RATE_LIMIT", "true"); // Opcional, dependendo se quer testar rate limit ou não

    let config = Config::from_env().expect("Falha ao carregar config de teste");
    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db");
    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");
    let encoding_key = EncodingKey::from_ed_pem(private_pem).expect("Chave privada inválida");
    let decoding_key = DecodingKey::from_ed_pem(public_pem).expect("Chave pública inválida");
    let app_state = AppState {
        enforcer,
        policy_cache: Arc::new(RwLock::new(HashMap::new())),
        db_pool_auth: pool_auth.clone(),
        db_pool_logs: pool_logs.clone(),
        encoding_key,
        decoding_key,
    };

    let app = build_router(app_state);
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
    }
}

async fn configure_database(db_url: &str, db_name: &str) {
    let mut conn = PgPool::connect(db_url)
        .await
        .unwrap()
        .acquire()
        .await
        .unwrap();

    // Limpa tabelas (exemplo simples, ajuste para suas tabelas)
    conn.execute("DROP TABLE IF EXISTS users, refresh_tokens, casbin_rule CASCADE;")
        .await
        .unwrap();

    // Roda migrações
    let migration_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../persistence/migrations_{}", db_name));
    sqlx::migrate::Migrator::new(migration_path)
        .await
        .unwrap()
        .run(&mut *conn)
        .await
        .unwrap();
}

// --- FUNÇÕES DE AJUDA USADAS NOS TESTES ---

/// Registra um usuário via API e retorna a TestResponse
pub async fn register_user(app: &TestApp, username: &str, password: &str) -> TestResponse {
    let payload = RegisterPayload {
        username: username.to_string(),
        password: password.to_string(),
    };
    app.api.post("/auth/register").json(&payload).await
}

/// Faz login de um usuário via API e retorna a LoginResponse (já deserializada)
pub async fn login_user(app: &TestApp, username: &str, password: &str) -> LoginResponse {
    let payload = LoginPayload {
        username: username.to_string(),
        password: password.to_string(),
    };
    let response = app.api.post("/auth/login").json(&payload).await;
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Falha ao fazer login"
    );
    response
        .json() // <-- CORREÇÃO: .json() não é async
        .expect("Falha ao deserializar LoginResponse")
}

// --- SETUP DE USUÁRIOS PARA TESTES (INTERNO) ---

// Gera tokens para testes de autorização (admin e user)
async fn setup_test_users(db_pool: &PgPool, api: &TestServer) -> (String, String) {
    // 1. Criar usuário "admin"
    let admin_pass = "AdminP@ss123!";
    let admin_user = internal_create_user(db_pool, "admin_test", admin_pass)
        .await
        .expect("Falha ao criar admin interno");

    // 2. Criar usuário "user"
    let user_pass = "UserP@ss123!";
    internal_create_user(db_pool, "user_test", user_pass)
        .await
        .expect("Falha ao criar user interno");

    // 3. Fazer login de ambos para obter tokens
    let admin_login = LoginPayload {
        username: "admin_test".to_string(),
        password: admin_pass.to_string(),
    };
    let admin_token = api
        .post("/auth/login")
        .json(&admin_login)
        .await
        .json::<LoginResponse>() // <-- CORREÇÃO: .json() não é async
        .unwrap()
        .access_token;

    let user_login = LoginPayload {
        username: "user_test".to_string(),
        password: user_pass.to_string(),
    };
    let user_token = api
        .post("/auth/login")
        .json(&user_login)
        .await
        .json::<LoginResponse>() // <-- CORREÇÃO: .json() não é async
        .unwrap()
        .access_token;

    // 4. (Opcional, se precisar de Casbin) Adicionar role de admin
    // ...

    (admin_token, user_token)
}

// Cria usuário direto no DB (helper para setup)
async fn internal_create_user(
    db_pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<Uuid, anyhow::Error> {
    // --- CORREÇÃO: Tratar o erro String ---
    validate_password_strength(password).map_err(anyhow::anyhow)?;

    let password_hash = tokio::task::spawn_blocking(move || hash_password(password)).await??;
    let user_repo = UserRepository::new(db_pool);
    let user = user_repo.create(username, &password_hash).await?;
    Ok(user.id)
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
        exp: now + 3600,
        iat: now,
        token_type: TokenType::Access,
    };
}
