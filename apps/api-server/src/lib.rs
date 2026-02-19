use anyhow::{Context, Result};
use application::services::audit_services::AuditService;
use moka::future::Cache;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info};

// Imports de Portas e Serviços
use application::services::{
    auth_service::AuthService, budget_classifications_service::BudgetClassificationsService,
    catalog_service::CatalogService, geo_regions_service::GeoRegionsService,
    mfa_service::MfaService,
    organizational_service::{
        OrganizationService, OrganizationalUnitCategoryService, OrganizationalUnitService,
        OrganizationalUnitTypeService, SystemSettingsService,
    },
    requisition_service::RequisitionService,
    supplier_service::SupplierService,
    driver_service::DriverService,
    fueling_service::FuelingService,
    vehicle_fine_service::VehicleFineService,
    user_service::UserService,
    vehicle_service::VehicleService,
};
use domain::ports::{
    AuthRepositoryPort, BudgetClassificationRepositoryPort, BuildingRepositoryPort,
    BuildingTypeRepositoryPort, CatalogGroupRepositoryPort, CatalogItemRepositoryPort,
    CityRepositoryPort, CountryRepositoryPort, EmailServicePort, FloorRepositoryPort,
    MfaRepositoryPort, OrganizationRepositoryPort, OrganizationalUnitCategoryRepositoryPort,
    OrganizationalUnitRepositoryPort, OrganizationalUnitTypeRepositoryPort,
    RequisitionItemRepositoryPort, RequisitionRepositoryPort,
    SiorgHistoryRepositoryPort, SiorgSyncQueueRepositoryPort, SiteRepositoryPort,
    SpaceRepositoryPort, SpaceTypeRepositoryPort, StateRepositoryPort,
    SystemSettingsRepositoryPort, UnitConversionRepositoryPort, UnitOfMeasureRepositoryPort,
    UserRepositoryPort,
    VehicleCategoryRepositoryPort, VehicleMakeRepositoryPort, VehicleModelRepositoryPort,
    VehicleColorRepositoryPort, VehicleFuelTypeRepositoryPort, VehicleTransmissionTypeRepositoryPort,
    SupplierRepositoryPort,
    DriverRepositoryPort,
    FuelingRepositoryPort,
    VehicleFineTypeRepositoryPort, VehicleFineRepositoryPort, VehicleFineStatusHistoryRepositoryPort,
    VehicleRepositoryPort, VehicleDocumentRepositoryPort, VehicleStatusHistoryRepositoryPort,
};
use persistence::repositories::{
    auth_repository::AuthRepository,
    budget_classifications_repository::BudgetClassificationRepository,
    catalog_repository::{
        CatalogGroupRepository, CatalogItemRepository, UnitConversionRepository,
        UnitOfMeasureRepository,
    },
    facilities_repository::{
        BuildingRepository, BuildingTypeRepository, FloorRepository, SiteRepository,
        SpaceRepository, SpaceTypeRepository,
    },
    geo_regions_repository::{CityRepository, CountryRepository, StateRepository},
    mfa_repository::MfaRepository,
    organizational_repository::{
        OrganizationRepository, OrganizationalUnitCategoryRepository,
        OrganizationalUnitRepository, OrganizationalUnitTypeRepository, SystemSettingsRepository,
    },
    requisition_repository::{RequisitionItemRepository, RequisitionRepository},
    user_repository::UserRepository,
    supplier_repository::SupplierRepository,
    driver_repository::DriverRepository,
    fueling_repository::FuelingRepository,
    vehicle_fine_repository::{VehicleFineTypeRepository, VehicleFineRepository, VehicleFineStatusHistoryRepository},
    vehicle_repository::{
        VehicleCategoryRepository, VehicleMakeRepository, VehicleModelRepository,
        VehicleColorRepository, VehicleFuelTypeRepository, VehicleTransmissionTypeRepository,
        VehicleRepository, VehicleDocumentRepository, VehicleStatusHistoryRepository,
    },
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

    let country_repo_port: Arc<dyn CountryRepositoryPort> =
        Arc::new(CountryRepository::new(pool_auth.clone()));
    let state_repo_port: Arc<dyn StateRepositoryPort> =
        Arc::new(StateRepository::new(pool_auth.clone()));
    let city_repo_port: Arc<dyn CityRepositoryPort> =
        Arc::new(CityRepository::new(pool_auth.clone()));

    let building_type_repo_port: Arc<dyn BuildingTypeRepositoryPort> =
        Arc::new(BuildingTypeRepository::new(pool_auth.clone()));
    let space_type_repo_port: Arc<dyn SpaceTypeRepositoryPort> =
        Arc::new(SpaceTypeRepository::new(pool_auth.clone()));
    let site_repo_port: Arc<dyn SiteRepositoryPort> =
        Arc::new(SiteRepository::new(pool_auth.clone()));
    let building_repo_port: Arc<dyn BuildingRepositoryPort> =
        Arc::new(BuildingRepository::new(pool_auth.clone()));
    let floor_repo_port: Arc<dyn FloorRepositoryPort> =
        Arc::new(FloorRepository::new(pool_auth.clone()));
    let space_repo_port: Arc<dyn SpaceRepositoryPort> =
        Arc::new(SpaceRepository::new(pool_auth.clone()));

    let location_service = Arc::new(GeoRegionsService::new(
        country_repo_port,
        state_repo_port,
        city_repo_port,
    ));

    let budget_classifications_repo_port: Arc<dyn BudgetClassificationRepositoryPort> =
        Arc::new(BudgetClassificationRepository::new(pool_auth.clone()));

    let budget_classifications_service = Arc::new(BudgetClassificationsService::new(
        budget_classifications_repo_port,
    ));

    let unit_repo_port: Arc<dyn UnitOfMeasureRepositoryPort> =
        Arc::new(UnitOfMeasureRepository::new(pool_auth.clone()));
    let group_repo_port: Arc<dyn CatalogGroupRepositoryPort> =
        Arc::new(CatalogGroupRepository::new(pool_auth.clone()));
    let item_repo_port: Arc<dyn CatalogItemRepositoryPort> =
        Arc::new(CatalogItemRepository::new(pool_auth.clone()));
    let conversion_repo_port: Arc<dyn UnitConversionRepositoryPort> =
        Arc::new(UnitConversionRepository::new(pool_auth.clone()));

    let catalog_service = Arc::new(CatalogService::new(
        unit_repo_port,
        group_repo_port,
        item_repo_port,
        conversion_repo_port,
    ));

    // Organizational repositories
    let pool_auth_arc = Arc::new(pool_auth.clone());
    let system_settings_repo_port: Arc<dyn SystemSettingsRepositoryPort> =
        Arc::new(SystemSettingsRepository::new(pool_auth_arc.clone()));
    let organization_repo_port: Arc<dyn OrganizationRepositoryPort> =
        Arc::new(OrganizationRepository::new(pool_auth_arc.clone()));
    let unit_category_repo_port: Arc<dyn OrganizationalUnitCategoryRepositoryPort> =
        Arc::new(OrganizationalUnitCategoryRepository::new(pool_auth_arc.clone()));
    let unit_type_repo_port: Arc<dyn OrganizationalUnitTypeRepositoryPort> =
        Arc::new(OrganizationalUnitTypeRepository::new(pool_auth_arc.clone()));
    let organizational_unit_repo_port: Arc<dyn OrganizationalUnitRepositoryPort> =
        Arc::new(OrganizationalUnitRepository::new(pool_auth_arc.clone()));

    // SIORG Sync repositories
    let siorg_sync_queue_repo_port: Arc<dyn SiorgSyncQueueRepositoryPort> =
        Arc::new(persistence::repositories::siorg_sync_repository::SiorgSyncQueueRepository::new(pool_auth.clone()));
    let siorg_history_repo_port: Arc<dyn SiorgHistoryRepositoryPort> =
        Arc::new(persistence::repositories::siorg_sync_repository::SiorgHistoryRepository::new(pool_auth.clone()));

    // Organizational services
    let system_settings_service = Arc::new(SystemSettingsService::new(
        system_settings_repo_port.clone(),
    ));
    let organization_service = Arc::new(OrganizationService::new(
        organization_repo_port.clone(),
    ));
    let organizational_unit_category_service = Arc::new(OrganizationalUnitCategoryService::new(
        unit_category_repo_port.clone(),
    ));
    let organizational_unit_type_service = Arc::new(OrganizationalUnitTypeService::new(
        unit_type_repo_port.clone(),
    ));
    let organizational_unit_service = Arc::new(OrganizationalUnitService::new(
        organizational_unit_repo_port.clone(),
        organization_repo_port.clone(),
        unit_category_repo_port.clone(),
        unit_type_repo_port.clone(),
        system_settings_repo_port.clone(),
    ));

    // SIORG Sync Service
    let siorg_base_url = std::env::var("SIORG_API_URL")
        .unwrap_or_else(|_| "https://estruturaorganizacional.dados.gov.br/doc".to_string());
    let siorg_token = std::env::var("SIORG_API_TOKEN").ok();

    let siorg_client = Arc::new(
        application::external::SiorgClient::new(siorg_base_url, siorg_token)
            .expect("Failed to create SIORG client")
    );

    let siorg_sync_service = Arc::new(application::external::SiorgSyncService::new(
        siorg_client,
        organization_repo_port,
        organizational_unit_repo_port,
        unit_category_repo_port,
        unit_type_repo_port,
    ));

    // Requisition repositories and service
    let requisition_repo_port: Arc<dyn RequisitionRepositoryPort> =
        Arc::new(RequisitionRepository::new(pool_auth.clone()));
    let requisition_item_repo_port: Arc<dyn RequisitionItemRepositoryPort> =
        Arc::new(RequisitionItemRepository::new(pool_auth.clone()));

    let requisition_service = Arc::new(RequisitionService::new(
        requisition_repo_port,
        requisition_item_repo_port,
    ));

    // Supplier repository and service
    let supplier_repo: Arc<dyn SupplierRepositoryPort> =
        Arc::new(SupplierRepository::new(pool_auth.clone()));
    let supplier_service = Arc::new(SupplierService::new(supplier_repo));

    // Driver repository and service
    let driver_repo: Arc<dyn DriverRepositoryPort> =
        Arc::new(DriverRepository::new(pool_auth.clone()));
    let driver_service = Arc::new(DriverService::new(driver_repo));

    // Fueling repository and service
    let fueling_repo: Arc<dyn FuelingRepositoryPort> =
        Arc::new(FuelingRepository::new(pool_auth.clone()));
    let fueling_service = Arc::new(FuelingService::new(fueling_repo));

    // Vehicle fleet repositories and service
    let vehicle_category_repo: Arc<dyn VehicleCategoryRepositoryPort> =
        Arc::new(VehicleCategoryRepository::new(pool_auth.clone()));
    let vehicle_make_repo: Arc<dyn VehicleMakeRepositoryPort> =
        Arc::new(VehicleMakeRepository::new(pool_auth.clone()));
    let vehicle_model_repo: Arc<dyn VehicleModelRepositoryPort> =
        Arc::new(VehicleModelRepository::new(pool_auth.clone()));
    let vehicle_color_repo: Arc<dyn VehicleColorRepositoryPort> =
        Arc::new(VehicleColorRepository::new(pool_auth.clone()));
    let vehicle_fuel_type_repo: Arc<dyn VehicleFuelTypeRepositoryPort> =
        Arc::new(VehicleFuelTypeRepository::new(pool_auth.clone()));
    let vehicle_transmission_type_repo: Arc<dyn VehicleTransmissionTypeRepositoryPort> =
        Arc::new(VehicleTransmissionTypeRepository::new(pool_auth.clone()));
    let vehicle_repo: Arc<dyn VehicleRepositoryPort> =
        Arc::new(VehicleRepository::new(pool_auth.clone()));
    let vehicle_document_repo: Arc<dyn VehicleDocumentRepositoryPort> =
        Arc::new(VehicleDocumentRepository::new(pool_auth.clone()));
    let vehicle_status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort> =
        Arc::new(VehicleStatusHistoryRepository::new(pool_auth.clone()));

    let vehicle_service = Arc::new(VehicleService::new(
        vehicle_repo,
        vehicle_category_repo,
        vehicle_make_repo,
        vehicle_model_repo,
        vehicle_color_repo,
        vehicle_fuel_type_repo,
        vehicle_transmission_type_repo,
        vehicle_document_repo,
        vehicle_status_history_repo,
    ));

    // Vehicle fine repositories and service
    let vehicle_fine_type_repo: Arc<dyn VehicleFineTypeRepositoryPort> =
        Arc::new(VehicleFineTypeRepository::new(pool_auth.clone()));
    let vehicle_fine_repo: Arc<dyn VehicleFineRepositoryPort> =
        Arc::new(VehicleFineRepository::new(pool_auth.clone()));
    let vehicle_fine_status_history_repo: Arc<dyn VehicleFineStatusHistoryRepositoryPort> =
        Arc::new(VehicleFineStatusHistoryRepository::new(pool_auth.clone()));
    let vehicle_fine_service = Arc::new(VehicleFineService::new(
        vehicle_fine_type_repo,
        vehicle_fine_repo,
        vehicle_fine_status_history_repo,
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
        budget_classifications_service,
        catalog_service,
        system_settings_service,
        organization_service,
        organizational_unit_category_service,
        organizational_unit_type_service,
        organizational_unit_service,
        siorg_sync_service,
        siorg_sync_queue_repository: siorg_sync_queue_repo_port,
        siorg_history_repository: siorg_history_repo_port,
        requisition_service,
        supplier_service,
        vehicle_service,
        driver_service,
        fueling_service,
        vehicle_fine_service,
        config,

        site_repository: site_repo_port,
        building_repository: building_repo_port,
        floor_repository: floor_repo_port,
        space_repository: space_repo_port,
        building_type_repository: building_type_repo_port,
        space_type_repository: space_type_repo_port,
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
    let pool_auth = PgPool::connect(&config.main_db).await?;
    let pool_logs = PgPool::connect(&config.logs_db).await?;

    info!("🔐 Inicializando Casbin...");
    let enforcer = match infra::casbin_setup::setup_casbin(pool_auth.clone()).await {
        Ok(e) => e,
        Err(_) => infra::casbin_setup::setup_casbin(pool_auth.clone()).await?,
    };

    info!("🔑 Inicializando serviço JWT...");
    let (private_pem, public_pem) = if !config.jwt_private_key.is_empty()
        && !config.jwt_public_key.is_empty()
    {
        (
            config.jwt_private_key.as_bytes().to_vec(),
            config.jwt_public_key.as_bytes().to_vec(),
        )
    } else {
        info!("Carregando chaves JWT dos arquivos private.pem e public.pem");
        let private_pem =
            std::fs::read("private.pem").context("Falha ao ler private.pem. Certifique-se de que o arquivo existe no diretório raiz do projeto.")?;
        let public_pem = std::fs::read("public.pem")
            .context("Falha ao ler public.pem. Certifique-se de que o arquivo existe no diretório raiz do projeto.")?;
        (private_pem, public_pem)
    };

    let jwt_service = Arc::new(
        JwtService::new(&private_pem, &public_pem).context("Falha ao inicializar chaves JWT")?,
    );

    info!("📧 Inicializando serviço de email...");
    let email_config = EmailConfig::from_env().context("Config email")?;
    let email_service =
        Arc::new(EmailService::new(email_config).context("Falha ao criar serviço de email")?);

    let app_state = build_application_state(
        config_arc.clone(),
        pool_auth.clone(),
        pool_logs,
        enforcer,
        jwt_service,
        email_service.clone(),
        email_service.clone(),
    );

    // Optional embedded SIORG sync worker
    let enable_worker = std::env::var("ENABLE_EMBEDDED_WORKER")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if enable_worker {
        info!("🔄 Inicializando worker embutido de sincronização SIORG...");

        // Clone necessary dependencies for worker
        let sync_queue_repo = app_state.siorg_sync_queue_repository.clone();
        let history_repo = app_state.siorg_history_repository.clone();
        let sync_service = app_state.siorg_sync_service.clone();

        // Create worker config
        let worker_config = application::workers::siorg_sync_worker::WorkerConfig {
            batch_size: std::env::var("WORKER_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            poll_interval_secs: std::env::var("WORKER_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            max_retries: 3,
            retry_base_delay_ms: 1000,
            retry_max_delay_ms: 60000,
            enable_cleanup: true,
            cleanup_interval_secs: 3600,
        };

        let worker = application::workers::SiorgSyncWorkerCore::new(
            worker_config,
            sync_queue_repo,
            history_repo,
            sync_service,
        );

        // Spawn worker task
        tokio::spawn(async move {
            info!("Worker de sincronização iniciado");
            if let Err(e) = worker.run_forever().await {
                error!("Worker de sincronização falhou: {}", e);
            }
        });

        info!("✅ Worker embutido iniciado com sucesso");
    } else {
        info!("⏭️  Worker embutido desabilitado (ENABLE_EMBEDDED_WORKER=false)");
    }

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
