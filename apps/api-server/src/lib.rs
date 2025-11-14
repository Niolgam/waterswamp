use anyhow::{Context, Result};
use config::Config;
use email_service::config::EmailConfig;
use email_service::EmailService;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

pub mod casbin_setup;
pub mod config;
mod constants;
pub mod error;
pub mod logging;
pub mod metrics;
mod middleware;
pub mod openapi;
pub mod rate_limit;
pub mod routes;
pub mod shutdown;
pub mod state;
pub mod web_models;

pub async fn run(addr: SocketAddr) -> Result<()> {
    // ⭐ 1. Inicializar logging ANTES de tudo
    let log_config = logging::LoggingConfig::default();
    logging::init_logging(log_config)?;

    // ⭐ 2. Inicializar timestamp de uptime
    routes::health_handler::init_server_start_time();

    info!("🚀 Iniciando Waterswamp API...");

    let config = Config::from_env()?;

    info!("🔌 Conectando aos bancos de dados...");
    let pool_auth = PgPool::connect(&config.auth_db).await?;
    let pool_logs = PgPool::connect(&config.logs_db).await?;
    info!("✅ Conexões com bancos estabelecidas");

    info!("🔐 Inicializando Casbin...");
    let enforcer = casbin_setup::setup_casbin(pool_auth.clone()).await?;
    info!("✅ Casbin inicializado");

    let encoding_key = EncodingKey::from_ed_pem(config.jwt_private_key.as_bytes())
        .context("Falha ao carregar chave privada EdDSA")?;

    let decoding_key = DecodingKey::from_ed_pem(config.jwt_public_key.as_bytes())
        .context("Falha ao carregar chave pública EdDSA")?;

    info!("📧 Inicializando serviço de email...");
    let email_config =
        EmailConfig::from_env().context("Falha ao carregar configuração de email")?;
    let email_service =
        EmailService::new(email_config).context("Falha ao criar transportador de email")?;
    info!("✅ Serviço de email pronto");

    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    let app_state = state::AppState {
        enforcer: enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        encoding_key,
        decoding_key,
        email_service: Arc::new(email_service),
    };

    info!("📡 Construindo rotas...");
    let app = routes::build_router(app_state);

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 Servidor ouvindo em {}", addr);
    info!("✨ Waterswamp API pronta para receber requisições!");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    info!("👋 Servidor encerrado graciosamente");
    Ok(())
}
