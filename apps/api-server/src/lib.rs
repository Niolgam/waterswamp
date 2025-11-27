use anyhow::{Context, Result};
use core_services::jwt::JwtService;
use email_service::{EmailConfig, EmailSender, EmailService};
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

use crate::handlers::audit_services::AuditService;

pub mod infra;
pub use infra::casbin_setup;
pub use infra::config;
pub use infra::errors as error;
pub mod handlers;
pub use infra::telemetry;
mod middleware;
pub mod openapi;
pub mod routes;
pub mod shutdown;
pub use infra::state;
pub mod api;
pub mod extractors;
pub mod utils;
pub mod web_models;

pub async fn run(addr: SocketAddr) -> Result<()> {
    // ⭐ 1. Inicializar logging ANTES de tudo
    let log_config = telemetry::LoggingConfig::default();
    telemetry::init_logging(log_config)?;

    // ⭐ 2. Inicializar timestamp de uptime
    handlers::health_handler::init_server_start_time();

    info!("🚀 Iniciando Waterswamp API...");

    let config = config::Config::from_env()?;

    info!("🔌 Conectando aos bancos de dados...");
    let pool_auth = PgPool::connect(&config.auth_db).await?;
    let pool_logs = PgPool::connect(&config.logs_db).await?;
    info!("✅ Conexões com bancos estabelecidas");

    info!("🔐 Inicializando Casbin...");
    let enforcer = casbin_setup::setup_casbin(pool_auth.clone()).await?;
    info!("✅ Casbin inicializado");

    info!("🔑 Inicializando serviço JWT (EdDSA)...");
    let jwt_service = JwtService::new(
        config.jwt_private_key.as_bytes(),
        config.jwt_public_key.as_bytes(),
    )
    .context("Falha ao inicializar JwtService")?;

    info!("📧 Inicializando serviço de email...");
    let email_config =
        EmailConfig::from_env().context("Falha ao carregar configuração de email")?;
    let email_service =
        EmailService::new(email_config).context("Falha ao criar transportador de email")?;
    info!("✅ Serviço de email pronto");

    info!("📝 Inicializando serviço de audit...");
    let audit_service = AuditService::new(pool_logs.clone());
    info!("✅ Serviço de audit pronto");

    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    let app_state = state::AppState {
        enforcer: enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        jwt_service,
        email_service: Arc::new(email_service),
        audit_service,
    };

    info!("📡 Construindo rotas...");
    let app = routes::build(app_state);

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 Servidor ouvindo em {}", addr);
    info!("✨ Waterswamp API pronta para receber requisições!");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    info!("👋 Servidor encerrado graciosamente");
    Ok(())
}
