use anyhow::{Context, Result};
use application::services::audit_services::AuditService;
use axum::Router;
use moka::future::Cache;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

// Imports de Portas e Serviços
use application::services::{
    auth_service::AuthService, location_service::LocationService, mfa_service::MfaService,
    user_service::UserService,
};
use domain::ports::{
    AuthRepositoryPort, BuildingTypeRepositoryPort, CityRepositoryPort,
    DepartmentCategoryRepositoryPort, EmailServicePort, MfaRepositoryPort, SiteRepositoryPort,
    SiteTypeRepositoryPort, SpaceTypeRepositoryPort, StateRepositoryPort, UserRepositoryPort,
};
use persistence::repositories::{
    auth_repository::AuthRepository,
    location_repository::{
        BuildingTypeRepository, CityRepository, DepartmentCategoryRepository, SiteRepository,
        SiteTypeRepository, SpaceTypeRepository, StateRepository,
    },
    mfa_repository::MfaRepository,
    user_repository::UserRepository,
};

// Core & Infra
use core_services::jwt::JwtService;
use email_service::{EmailConfig, EmailSender, EmailService}; // Removido MockEmailService

use crate::infra::config::Config;
use crate::infra::telemetry::LoggingConfig;
use crate::shutdown::shutdown_signal;

pub mod api;
pub mod extractors;
pub mod handlers;
pub mod infra;
mod middleware;
pub mod openapi;
pub mod routes;
pub mod shutdown;
pub mod utils;
pub use infra::state;

pub fn build_application_state(
    config: Arc<Config>,
    pool_auth: PgPool,
    pool_logs: PgPool,
    enforcer: state::SharedEnforcer,
    jwt_service: Arc<JwtService>,
    email_service_legacy: Arc<dyn EmailSender + Send + Sync>,
    email_service_port: Arc<dyn EmailServicePort>,
) -> state::AppState {
    let audit_service = AuditService::new(pool_logs.clone());

    let user_repo_port: Arc<dyn UserRepositoryPort> =
        Arc::new(UserRepository::new(pool_auth.clone()));

    let auth_repo_port: Arc<dyn AuthRepositoryPort> =
        Arc::new(AuthRepository::new(pool_auth.clone()));

    let mfa_repo_port: Arc<dyn MfaRepositoryPort> = Arc::new(MfaRepository::new(pool_auth.clone()));

    let auth_service = Arc::new(AuthService::new(
        user_repo_port.clone(),
        auth_repo_port.clone(),
        email_service_port.clone(),
        jwt_service.clone(),
    ));

    let user_service = Arc::new(UserService::new(
        user_repo_port.clone(),
        auth_repo_port.clone(),
        email_service_port.clone(),
        jwt_service.clone(),
    ));

    let mfa_service = Arc::new(MfaService::new(
        mfa_repo_port.clone(),
        user_repo_port.clone(),
        auth_repo_port.clone(),
        email_service_port.clone(),
        jwt_service.clone(),
    ));

    // Location repositories and service
    let state_repo_port: Arc<dyn StateRepositoryPort> =
        Arc::new(StateRepository::new(pool_auth.clone()));
    let city_repo_port: Arc<dyn CityRepositoryPort> =
        Arc::new(CityRepository::new(pool_auth.clone()));
    let site_type_repo_port: Arc<dyn SiteTypeRepositoryPort> =
        Arc::new(SiteTypeRepository::new(pool_auth.clone()));
    let building_type_repo_port: Arc<dyn BuildingTypeRepositoryPort> =
        Arc::new(BuildingTypeRepository::new(pool_auth.clone()));
    let space_type_repo_port: Arc<dyn SpaceTypeRepositoryPort> =
        Arc::new(SpaceTypeRepository::new(pool_auth.clone()));
    let department_category_repo_port: Arc<dyn DepartmentCategoryRepositoryPort> =
        Arc::new(DepartmentCategoryRepository::new(pool_auth.clone()));
    let site_repo_port: Arc<dyn SiteRepositoryPort> =
        Arc::new(SiteRepository::new(pool_auth.clone()));

    let location_service = Arc::new(LocationService::new(
        state_repo_port,
        city_repo_port,
        site_type_repo_port,
        building_type_repo_port,
        space_type_repo_port,
        department_category_repo_port,
        site_repo_port,
    ));

    // Cache com TTL e tamanho máximo para políticas do Casbin
    let policy_cache = Cache::builder()
        .max_capacity(10_000) // Máximo 10k entries
        .time_to_live(Duration::from_secs(300)) // TTL de 5 minutos
        .build();

    state::AppState {
        enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        jwt_service,
        email_service: email_service_legacy,
        audit_service,
        auth_service,
        user_service,
        mfa_service,
        location_service,
        config,
    }
}

pub async fn run(addr: SocketAddr) -> Result<()> {
    let log_config = LoggingConfig::default();
    infra::telemetry::init_logging(log_config)?;

    handlers::health_handler::init_server_start_time();

    info!("🚀 Iniciando Waterswamp API (lib run)...");

    let config = Config::from_env()?;
    let config_arc = Arc::new(config.clone());

    info!("🔌 Conectando aos bancos de dados...");
    let pool_auth = PgPool::connect(&config.auth_db).await?;
    let pool_logs = PgPool::connect(&config.logs_db).await?;

    info!("🔐 Inicializando Casbin...");
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
    let email_config = EmailConfig::from_env().context("Config email")?;
    let email_service =
        Arc::new(EmailService::new(email_config).context("Falha ao criar serviço de email")?);

    let app_state = build_application_state(
        config_arc,
        pool_auth,
        pool_logs,
        enforcer,
        jwt_service,
        email_service.clone(),
        email_service.clone(),
    );

    info!("📡 Construindo rotas...");
    let app = routes::build(app_state);

    let listener = TcpListener::bind(addr)
        .await
        .context(format!("Falha ao fazer bind na porta {}", addr.port()))?;

    info!("✨ Waterswamp API ouvindo em {}", addr);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
