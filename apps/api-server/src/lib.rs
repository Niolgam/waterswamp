use crate::handlers::audit_services::AuditService;
use crate::infra::config::Config;
use crate::infra::telemetry::LoggingConfig;
use crate::state::AppState;
use anyhow::{Context, Result};
use application::services::auth_service::AuthService;
use application::services::mfa_service::MfaService;
use application::services::user_service::UserService;
use core_services::jwt::JwtService;
use domain::ports::MfaRepositoryPort;
use domain::ports::{AuthRepositoryPort, EmailServicePort, UserRepositoryPort};
use email_service::{EmailConfig, EmailSender, EmailService};
use persistence::repositories::mfa_repository::MfaRepository;
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

pub mod handlers;
pub mod infra;
mod middleware;
pub mod openapi;
pub mod routes;
pub mod shutdown;
pub use infra::state;
pub mod api;
pub mod extractors;
pub mod utils;

pub async fn run(addr: SocketAddr) -> Result<()> {
    let log_config = LoggingConfig::default();
    infra::telemetry::init_logging(log_config)?;

    // 2. Inicializar Uptime
    handlers::health_handler::init_server_start_time();

    info!("🚀 Iniciando Waterswamp API...");

    // 3. Carregar Configurações
    // O truque: Isso funciona porque o main.rs já rodou dotenvy::dotenv()!
    let config = Config::from_env().context("Falha ao carregar configurações do ambiente")?;
    let config_arc = Arc::new(config.clone());

    info!("🔌 Conectando aos bancos de dados...");
    let pool_auth = sqlx::PgPool::connect(&config.auth_db)
        .await
        .context("Falha ao conectar no banco de Auth")?;
    let pool_logs = sqlx::PgPool::connect(&config.logs_db)
        .await
        .context("Falha ao conectar no banco de Logs")?;

    info!("🔐 Inicializando Casbin...");
    // Mantive sua lógica de retry/fallback
    let enforcer = match infra::casbin_setup::setup_casbin(pool_auth.clone()).await {
        Ok(e) => e,
        Err(_) => infra::casbin_setup::setup_casbin(pool_auth.clone()).await?,
    };

    info!("🔑 Inicializando serviço JWT...");
    let jwt_service = Arc::new(
        JwtService::new(
            config.jwt_private_key.as_bytes(),
            config.jwt_public_key.as_bytes(),
        )
        .context("Falha ao inicializar chaves JWT")?,
    );

    info!("📧 Inicializando serviço de email...");
    // Assumindo que EmailConfig também tem um from_env ou vem do config principal
    let email_config = EmailConfig::from_env()?;
    let email_service =
        Arc::new(EmailService::new(email_config).context("Falha ao criar serviço de email")?);

    info!("📝 Inicializando serviço de audit...");
    let audit_service = AuditService::new(pool_logs.clone());

    // --- WIRING (Injeção de Dependência) ---
    // Dica: Use Arc::clone(&var) é mais idiomático que var.clone() para Arcs,
    // deixa claro que é só uma referência e não deep copy.

    let user_repo = Arc::new(UserRepository::new(pool_auth.clone()));
    let auth_repo = Arc::new(AuthRepository::new(pool_auth.clone()));
    let mfa_repo_port: Arc<dyn MfaRepositoryPort> = Arc::new(MfaRepository::new(pool_auth.clone()));

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        auth_repo.clone(),
        email_service.clone(), // Passa a referência do Arc
        jwt_service.clone(),   // Passa a referência do Arc
    ));

    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        auth_repo.clone(),
        email_service.clone(),
    ));

    let mfa_service = Arc::new(MfaService::new(
        mfa_repo_port.clone(),
        user_repo.clone(),
        auth_repo.clone(),
        email_service.clone(),
        jwt_service.clone(), // Arc<JwtService>
    ));
    // Cache simples em memória
    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    // Construção do Estado Global (State)
    let app_state = AppState {
        enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        jwt_service,
        email_service,
        audit_service,
        auth_service,
        user_service,
        mfa_service,
        config: config_arc,
    };

    info!("📡 Construindo rotas e middleware...");
    let app = routes::build(app_state);

    // O "addr" veio passado pelo main.rs
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("Falha ao fazer bind na porta {}", addr.port()))?;

    info!("✨ Waterswamp API ouvindo em {}", addr);

    // Inicia o servidor Axum
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 Servidor encerrado graciosamente");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("falha ao instalar handler do Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("falha ao instalar handler do sinal de terminação")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
