use axum::{extract::State, http::StatusCode, Json};
use casbin::{CoreApi, MgmtApi};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub database: DatabaseHealth,
    pub casbin: CasbinHealth,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseHealth {
    pub auth_db: bool,
    pub logs_db: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CasbinHealth {
    pub operational: bool,
    pub policy_count: Option<usize>,
}

// Variável estática para armazenar o tempo de início do servidor
static mut SERVER_START_TIME: Option<u64> = None;

/// Inicializa o tempo de início do servidor
/// Deve ser chamado no início da função main() ou run()
pub fn init_server_start_time() {
    unsafe {
        SERVER_START_TIME = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Erro ao obter timestamp")
                .as_secs(),
        );
    }
}

/// Calcula o uptime do servidor em segundos
fn get_uptime_seconds() -> u64 {
    unsafe {
        match SERVER_START_TIME {
            Some(start) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Erro ao obter timestamp")
                    .as_secs();
                now.saturating_sub(start)
            }
            None => 0,
        }
    }
}

/// GET /health
/// Retorna 200 OK se o servidor está saudável
/// Retorna 503 Service Unavailable se houver problemas críticos
pub async fn handler_health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // 1. Testa conexão com o banco de autenticação
    let auth_db_healthy = check_database(&state.db_pool_auth).await;

    // 2. Testa conexão com o banco de logs
    let logs_db_healthy = check_database(&state.db_pool_logs).await;

    // 3. Testa se o Casbin está operacional
    let (casbin_operational, policy_count) = check_casbin(&state).await;

    // 4. Determina status geral
    let all_healthy = auth_db_healthy && logs_db_healthy && casbin_operational;

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
        casbin: CasbinHealth {
            operational: casbin_operational,
            policy_count,
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
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

/// Helper: Verifica se o Casbin está operacional
/// Retorna (operational: bool, policy_count: Option<usize>)
async fn check_casbin(state: &AppState) -> (bool, Option<usize>) {
    // Tenta fazer uma verificação simples de enforce
    // Usa um subject/object/action que sabemos que deve existir (ou não importa se não existir)
    let test_result = {
        let enforcer_guard = state.enforcer.read().await;

        // Tenta fazer um enforce de teste
        match enforcer_guard.enforce(vec![
            "health_check_test".to_string(),
            "/health".to_string(),
            "GET".to_string(),
        ]) {
            Ok(_) => {
                // Conseguiu fazer o enforce - Casbin está operacional
                // Tenta pegar a contagem de políticas
                let policy_count = enforcer_guard.get_policy().len();
                (true, Some(policy_count))
            }
            Err(e) => {
                tracing::error!("Erro ao verificar saúde do Casbin: {}", e);
                (false, None)
            }
        }
    };

    test_result
}

/// GET /health/ready
/// Endpoint mais leve - verifica se o servidor está pronto para receber tráfego
/// Não verifica o banco de dados (mais rápido)
/// Usado pelo Kubernetes para saber quando começar a enviar tráfego
pub async fn handler_ready() -> StatusCode {
    StatusCode::OK
}

/// GET /health/live
/// Endpoint de liveness - verifica se o servidor está vivo
/// Kubernetes pode usar isso para decidir se reinicia o pod
pub async fn handler_liveness() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_get_uptime() {
        init_server_start_time();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let uptime = get_uptime_seconds();
        assert!(uptime >= 1, "Uptime deve ser pelo menos 1 segundo");
    }
}
