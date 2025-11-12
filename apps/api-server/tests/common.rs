use axum::http::StatusCode;
use axum_test::{TestResponse, TestServer};
use core_services::security::{hash_password, validate_password_strength};
use domain::models::{LoginPayload, LoginResponse, RegisterPayload};
use once_cell::sync::Lazy;
use persistence::repositories::UserRepository;
use sqlx::{Connection, Executor, PgPool};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use waterswamp::{config::AppConfig, create_app_state, startup::create_app};

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
    Lazy::force(&TRACING);

    // Carrega configuração de um .env de teste (se existir) ou usa defaults
    let config = AppConfig {
        database_auth_url: std::env::var("DATABASE_AUTH_URL_TEST")
            .unwrap_or_else(|_| "postgres://test:test@localhost:5435/waterswamp_auth_test".into()),
        database_logs_url: std::env::var("DATABASE_LOGS_URL_TEST")
            .unwrap_or_else(|_| "postgres://test:test@localhost:5435/waterswamp_logs_test".into()),
        jwt_private_key_pem: fs::read_to_string("tests/keys/private_test.pem").unwrap(),
        jwt_public_key_pem: fs::read_to_string("tests/keys/public_test.pem").unwrap(),
        ..AppConfig::default()
    };

    // Configura banco de dados (limpa e migra)
    configure_database(&config.database_auth_url, "auth").await;
    configure_database(&config.database_logs_url, "logs").await;

    let app_state = create_app_state(&config)
        .await
        .expect("Falha ao criar AppState de teste");

    let app = create_app(app_state.clone()).await;
    let server = TestServer::new(app).expect("Falha ao criar TestServer");

    // Criar usuários e tokens de teste
    let (admin_token, user_token) = setup_test_users(&app_state.db_pool_auth, &server).await;

    TestApp {
        api: server,
        db_auth: app_state.db_pool_auth,
        db_logs: app_state.db_pool_logs,
        admin_token,
        user_token,
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
        .json()
        .await
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
        .json::<LoginResponse>()
        .await
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
        .json::<LoginResponse>()
        .await
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
    validate_password_strength(password)?;
    let password_hash = tokio::task::spawn_blocking(move || hash_password(password)).await??;
    let user_repo = UserRepository::new(db_pool);
    let user = user_repo.create(username, &password_hash).await?;
    Ok(user.id)
}
