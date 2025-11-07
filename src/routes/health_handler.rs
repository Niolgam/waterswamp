use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::state::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub database: DatabaseHealth,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseHealth {
    pub auth_db: bool,
    pub logs_db: bool,
}

/// GET /health
/// Retorna 200 OK se o servidor está saudável
/// Retorna 503 Service Unavailable se houver problemas
pub async fn handler_health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // Testa conexão com o banco de autenticação
    let auth_db_healthy = check_database(&state.db_pool_auth).await;

    // Testa conexão com o banco de logs
    let logs_db_healthy = check_database(&state.db_pool_logs).await;

    let all_healthy = auth_db_healthy && logs_db_healthy;

    let response = HealthResponse {
        status: if all_healthy {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        database: DatabaseHealth {
            auth_db: auth_db_healthy,
            logs_db: logs_db_healthy,
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    if all_healthy {
        Ok(Json(response))
    } else {
        tracing::error!("Health check falhou: {:?}", response);
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Helper: Verifica se o banco está acessível
async fn check_database(pool: &PgPool) -> bool {
    // Tenta fazer uma query simples
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Erro ao verificar saúde do banco: {}", e);
            false
        }
    }
}

/// GET /health/ready
/// Endpoint mais leve - verifica se o servidor está pronto para receber tráfego
/// Não verifica o banco de dados (mais rápido)
pub async fn handler_ready() -> StatusCode {
    StatusCode::OK
}

/// GET /health/live
/// Endpoint de liveness - verifica se o servidor está vivo
/// Kubernetes pode usar isso para decidir se reinicia o pod
pub async fn handler_liveness() -> StatusCode {
    StatusCode::OK
}
