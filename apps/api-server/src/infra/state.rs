use crate::handlers::audit_services::AuditService;
use application::services::auth_service::AuthService;
use casbin::Enforcer;
use core_services::jwt::JwtService;
use email_service::EmailSender;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedEnforcer = Arc<RwLock<Enforcer>>;
pub type PolicyCache = Arc<RwLock<HashMap<String, bool>>>;

#[derive(Clone)]
pub struct AppState {
    pub enforcer: SharedEnforcer,
    pub policy_cache: PolicyCache,
    pub db_pool_auth: PgPool,
    pub db_pool_logs: PgPool,
    pub jwt_service: JwtService,
    pub email_service: Arc<dyn EmailSender + Send + Sync>,
    pub audit_service: AuditService,
    pub auth_service: Arc<AuthService>,
}
