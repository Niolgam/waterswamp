use casbin::Enforcer;
use jsonwebtoken::{DecodingKey, EncodingKey};
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
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}
