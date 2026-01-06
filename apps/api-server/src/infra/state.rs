use crate::infra::config::Config;
use application::services::audit_services::AuditService;
use application::services::auth_service::AuthService;
use application::services::location_service::LocationService;
use application::services::mfa_service::MfaService;
use application::services::user_service::UserService;
use casbin::Enforcer;
use core_services::jwt::JwtService;
use domain::ports::{
    BuildingRepositoryPort, BuildingTypeRepositoryPort, FloorRepositoryPort, SiteRepositoryPort,
    SiteTypeRepositoryPort, SpaceRepositoryPort, SpaceTypeRepositoryPort,
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
    pub location_service: Arc<LocationService>,
    pub config: Arc<Config>,
    // Repositories for direct access in public handlers
    pub site_repository: Arc<dyn SiteRepositoryPort>,
    pub building_repository: Arc<dyn BuildingRepositoryPort>,
    pub floor_repository: Arc<dyn FloorRepositoryPort>,
    pub space_repository: Arc<dyn SpaceRepositoryPort>,
    pub building_type_repository: Arc<dyn BuildingTypeRepositoryPort>,
    pub space_type_repository: Arc<dyn SpaceTypeRepositoryPort>,
}
