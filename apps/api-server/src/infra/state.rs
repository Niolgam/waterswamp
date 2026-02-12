use crate::infra::config::Config;
use application::services::audit_services::AuditService;
use application::services::auth_service::AuthService;
use application::services::budget_classifications_service::BudgetClassificationsService;
use application::services::catalog_service::CatalogService;
use application::services::geo_regions_service::GeoRegionsService;
use application::services::mfa_service::MfaService;
use application::services::organizational_service::{
    OrganizationService, OrganizationalUnitCategoryService, OrganizationalUnitService,
    OrganizationalUnitTypeService, SystemSettingsService,
};
use application::services::requisition_service::RequisitionService;
use application::services::user_service::UserService;
use application::services::supplier_service::SupplierService;
use application::services::vehicle_service::VehicleService;
use application::services::driver_service::DriverService;
use application::services::fueling_service::FuelingService;
use application::services::vehicle_fine_service::VehicleFineService;
use application::external::SiorgSyncService;
use casbin::Enforcer;
use core_services::jwt::JwtService;
use domain::ports::{
    BuildingRepositoryPort, BuildingTypeRepositoryPort, FloorRepositoryPort, SiteRepositoryPort,
    SiorgHistoryRepositoryPort, SiorgSyncQueueRepositoryPort, SpaceRepositoryPort,
    SpaceTypeRepositoryPort,
};
use email_service::EmailSender;
use moka::future::Cache;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedEnforcer = Arc<RwLock<Enforcer>>;
pub type PolicyCache = Cache<String, bool>;

#[derive(Clone)]
pub struct AppState {
    pub enforcer: SharedEnforcer,
    pub policy_cache: PolicyCache,
    pub db_pool_auth: PgPool,
    pub db_pool_logs: PgPool,
    pub jwt_service: Arc<JwtService>,
    pub email_service: Arc<dyn EmailSender + Send + Sync>,
    pub audit_service: AuditService,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub mfa_service: Arc<MfaService>,
    pub location_service: Arc<GeoRegionsService>,
    pub budget_classifications_service: Arc<BudgetClassificationsService>,
    pub catalog_service: Arc<CatalogService>,
    pub system_settings_service: Arc<SystemSettingsService>,
    pub organization_service: Arc<OrganizationService>,
    pub organizational_unit_category_service: Arc<OrganizationalUnitCategoryService>,
    pub organizational_unit_type_service: Arc<OrganizationalUnitTypeService>,
    pub organizational_unit_service: Arc<OrganizationalUnitService>,
    pub siorg_sync_service: Arc<SiorgSyncService>,
    pub siorg_sync_queue_repository: Arc<dyn SiorgSyncQueueRepositoryPort>,
    pub siorg_history_repository: Arc<dyn SiorgHistoryRepositoryPort>,
    pub requisition_service: Arc<RequisitionService>,
    pub supplier_service: Arc<SupplierService>,
    pub vehicle_service: Arc<VehicleService>,
    pub driver_service: Arc<DriverService>,
    pub fueling_service: Arc<FuelingService>,
    pub vehicle_fine_service: Arc<VehicleFineService>,
    pub config: Arc<Config>,
    // Repositories for direct access in public handlers
    pub site_repository: Arc<dyn SiteRepositoryPort>,
    pub building_repository: Arc<dyn BuildingRepositoryPort>,
    pub floor_repository: Arc<dyn FloorRepositoryPort>,
    pub space_repository: Arc<dyn SpaceRepositoryPort>,
    pub building_type_repository: Arc<dyn BuildingTypeRepositoryPort>,
    pub space_type_repository: Arc<dyn SpaceTypeRepositoryPort>,
}
