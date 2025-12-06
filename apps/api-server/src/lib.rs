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
    auth_service::AuthService, campus_service::CampusService, mfa_service::MfaService,
    organizational_unit_service::OrganizationalUnitService,
    unit_category_service::UnitCategoryService, user_service::UserService,
};
use domain::ports::{
    AuthRepositoryPort, CampusRepositoryPort, EmailServicePort, MfaRepositoryPort,
    OrganizationalUnitRepositoryPort, UnitCategoryRepositoryPort, UserRepositoryPort,
};
use persistence::repositories::{
    auth_repository::AuthRepository, campus_repository::CampusRepository,
    mfa_repository::MfaRepository, organizational_unit_repository::OrganizationalUnitRepository,
    unit_category_repository::UnitCategoryRepository, user_repository::UserRepository,
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

    // Criar configuração de expiração de tokens a partir do Config
    let token_config = application::services::auth_service::TokenExpiryConfig {
        access_token_expiry_seconds: config.access_token_expiry_seconds,
        refresh_token_expiry_seconds: config.refresh_token_expiry_seconds,
        mfa_challenge_expiry_seconds: config.mfa_challenge_expiry_seconds,
        password_reset_expiry_seconds: config.password_reset_expiry_seconds,
        email_verification_expiry_seconds: config.email_verification_expiry_seconds,
    };

    let auth_service = Arc::new(AuthService::new(
        user_repo_port.clone(),
        auth_repo_port.clone(),
        email_service_port.clone(),
        jwt_service.clone(),
        token_config,
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

    // Create Campus repository and service
    let campus_repo_port: Arc<dyn CampusRepositoryPort> =
        Arc::new(CampusRepository::new(pool_auth.clone()));
    let campus_service = Arc::new(CampusService::new(campus_repo_port.clone()));

    // Create UnitCategory repository and service
    let unit_category_repo_port: Arc<dyn UnitCategoryRepositoryPort> =
        Arc::new(UnitCategoryRepository::new(pool_auth.clone()));
    let unit_category_service =
        Arc::new(UnitCategoryService::new(unit_category_repo_port.clone()));

    // Create OrganizationalUnit repository and service
    let org_unit_repo_port: Arc<dyn OrganizationalUnitRepositoryPort> =
        Arc::new(OrganizationalUnitRepository::new(pool_auth.clone()));
    let organizational_unit_service = Arc::new(OrganizationalUnitService::new(
        org_unit_repo_port.clone(),
        unit_category_repo_port.clone(),
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
        campus_service,
        unit_category_service,
        organizational_unit_service,
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
