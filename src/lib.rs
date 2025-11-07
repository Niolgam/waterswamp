use anyhow::Result;
use config::Config;
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
mod middleware;
pub mod models;
pub mod rate_limit;
pub mod routes;
pub mod security;
pub mod shutdown;
pub mod state;

pub async fn run(addr: SocketAddr) -> Result<()> {
    let config = Config::from_env()?;

    let pool_auth = PgPool::connect(&config.auth_db).await?;
    let pool_logs = PgPool::connect(&config.logs_db).await?;

    let enforcer = casbin_setup::setup_casbin(pool_auth.clone()).await?;
    let secret = config.jwt_secret;
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    let policy_cache = Arc::new(RwLock::new(HashMap::new()));

    let app_state = state::AppState {
        enforcer: enforcer,
        policy_cache,
        db_pool_auth: pool_auth,
        db_pool_logs: pool_logs,
        encoding_key: encoding_key,
        decoding_key: decoding_key,
    };

    let app = routes::build_router(app_state);

    let listener = TcpListener::bind(addr).await?;
    info!("🚀 Servidor ouvindo em {}", addr);
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    Ok(())
}
