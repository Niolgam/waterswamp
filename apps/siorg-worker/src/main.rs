use anyhow::{Context, Result};
use application::external::{SiorgClient, SiorgSyncService};
use application::workers::siorg_sync_worker::{SiorgSyncWorkerCore, WorkerConfig};
use domain::ports::{
    OrganizationRepositoryPort, OrganizationalUnitCategoryRepositoryPort,
    OrganizationalUnitRepositoryPort, OrganizationalUnitTypeRepositoryPort,
    SiorgHistoryRepositoryPort, SiorgSyncQueueRepositoryPort,
};
use persistence::repositories::{
    organizational_repository::{
        OrganizationRepository, OrganizationalUnitCategoryRepository,
        OrganizationalUnitRepository, OrganizationalUnitTypeRepository,
    },
    siorg_sync_repository::{SiorgHistoryRepository, SiorgSyncQueueRepository},
};
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::from_path("apps/siorg-worker/.env")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    // Initialize logging
    init_logging()?;

    info!("🚀 Iniciando SIORG Sync Worker (standalone)...");

    // Load configuration from environment
    let config = load_config_from_env()?;
    info!("📝 Configuração carregada: {:?}", config);

    // Connect to database
    info!("🔌 Conectando ao banco de dados...");
    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL deve estar definida no ambiente")?;
    let pool = PgPool::connect(&database_url)
        .await
        .context("Falha ao conectar ao banco de dados")?;
    info!("✅ Conectado ao banco de dados");

    // Setup repositories
    info!("📦 Inicializando repositórios...");
    let sync_queue_repo: Arc<dyn SiorgSyncQueueRepositoryPort> =
        Arc::new(SiorgSyncQueueRepository::new(pool.clone()));
    let history_repo: Arc<dyn SiorgHistoryRepositoryPort> =
        Arc::new(SiorgHistoryRepository::new(pool.clone()));

    let organization_repo: Arc<dyn OrganizationRepositoryPort> =
        Arc::new(OrganizationRepository::new(pool.clone()));
    let unit_repo: Arc<dyn OrganizationalUnitRepositoryPort> =
        Arc::new(OrganizationalUnitRepository::new(pool.clone()));
    let category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort> =
        Arc::new(OrganizationalUnitCategoryRepository::new(pool.clone()));
    let type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort> =
        Arc::new(OrganizationalUnitTypeRepository::new(pool.clone()));

    // Setup SIORG client and sync service
    info!("🌐 Inicializando cliente SIORG...");
    let siorg_base_url = env::var("SIORG_API_URL")
        .unwrap_or_else(|_| "https://api.siorg.gov.br".to_string());
    let siorg_token = env::var("SIORG_API_TOKEN").ok();

    let siorg_client = Arc::new(
        SiorgClient::new(siorg_base_url, siorg_token)
            .expect("Failed to create SIORG client"),
    );

    let sync_service = Arc::new(SiorgSyncService::new(
        siorg_client,
        organization_repo,
        unit_repo,
        category_repo,
        type_repo,
    ));

    // Create worker
    info!("⚙️  Criando worker...");
    let worker = SiorgSyncWorkerCore::new(
        config,
        sync_queue_repo,
        history_repo,
        sync_service,
    );

    info!("✨ Worker inicializado com sucesso!");
    info!("🔄 Iniciando processamento...");

    // Run worker forever
    if let Err(e) = worker.run_forever().await {
        error!("❌ Worker falhou: {}", e);
        return Err(anyhow::anyhow!("Worker failed: {}", e));
    }

    Ok(())
}

/// Initialize logging with JSON format for production
fn init_logging() -> Result<()> {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,siorg_worker=debug,application::workers=debug".to_string());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    // Use JSON format in production, pretty format in development
    let use_json = env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "text".to_string())
        .eq_ignore_ascii_case("json");

    if use_json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }

    Ok(())
}

/// Load worker configuration from environment variables
fn load_config_from_env() -> Result<WorkerConfig> {
    let config = WorkerConfig {
        batch_size: env::var("WORKER_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10),
        poll_interval_secs: env::var("WORKER_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5),
        max_retries: env::var("WORKER_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3),
        retry_base_delay_ms: env::var("WORKER_RETRY_BASE_DELAY_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000),
        retry_max_delay_ms: env::var("WORKER_RETRY_MAX_DELAY_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60000),
        enable_cleanup: env::var("WORKER_ENABLE_CLEANUP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        cleanup_interval_secs: env::var("WORKER_CLEANUP_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600),
    };

    Ok(config)
}
