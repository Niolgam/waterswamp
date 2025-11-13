use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, Encoder, HistogramVec,
    IntCounterVec, IntGauge, TextEncoder,
};
use std::time::Instant;

// =============================================================================
// MÉTRICAS GLOBAIS (Lazy Static)
// =============================================================================

lazy_static::lazy_static! {
    /// Contador de requisições HTTP por método e status
    /// Labels: method (GET, POST, etc), status (200, 404, etc), path
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total de requisições HTTP recebidas",
        &["method", "status", "path"]
    ).unwrap();

    /// Histograma de latência das requisições HTTP (em segundos)
    /// Labels: method, path
    /// Buckets: 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0 segundos
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "Duração das requisições HTTP",
        &["method", "path"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]
    ).unwrap();

    /// Contador de hits no cache de políticas
    pub static ref POLICY_CACHE_HITS: IntCounterVec = register_int_counter_vec!(
        "policy_cache_hits_total",
        "Total de cache hits (política encontrada no cache)",
        &["result"]  // "hit" ou "miss"
    ).unwrap();

    /// Gauge do número de políticas no Casbin
    pub static ref CASBIN_POLICIES_COUNT: IntGauge = register_int_gauge!(
        "casbin_policies_count",
        "Número atual de políticas carregadas no Casbin"
    ).unwrap();

    /// Contador de tentativas de login
    pub static ref LOGIN_ATTEMPTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "login_attempts_total",
        "Total de tentativas de login",
        &["result"]  // "success" ou "failure"
    ).unwrap();

    /// Contador de tokens renovados (refresh)
    pub static ref TOKEN_REFRESH_TOTAL: IntCounterVec = register_int_counter_vec!(
        "token_refresh_total",
        "Total de renovações de token",
        &["result"]  // "success" ou "failure"
    ).unwrap();

    /// Gauge de conexões ativas com o banco de dados
    pub static ref DB_CONNECTIONS_ACTIVE: IntGauge = register_int_gauge!(
        "db_connections_active",
        "Número de conexões ativas com o banco de dados"
    ).unwrap();

    /// Contador de erros HTTP por tipo
    pub static ref HTTP_ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_errors_total",
        "Total de erros HTTP",
        &["error_type"]  // "validation", "unauthorized", "forbidden", "internal"
    ).unwrap();
}

// =============================================================================
// MIDDLEWARE DE MÉTRICAS
// =============================================================================

/// Middleware que coleta métricas de requisições HTTP
///
/// Coleta:
/// - Contador de requisições por método/status/path
/// - Latência de cada requisição
pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let start = Instant::now();

    // Executa a requisição
    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Registra métricas
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &status, &path])
        .inc();

    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&method, &path])
        .observe(duration);

    response
}

// =============================================================================
// ENDPOINT /metrics
// =============================================================================

/// GET /metrics
/// Retorna métricas no formato Prometheus
pub async fn handler_metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Erro ao codificar métricas: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erro ao gerar métricas".to_string(),
        )
            .into_response();
    }

    let output = String::from_utf8(buffer).unwrap_or_else(|e| {
        tracing::error!("Erro ao converter métricas para UTF-8: {}", e);
        "Erro ao gerar métricas".to_string()
    });

    (StatusCode::OK, output).into_response()
}

// =============================================================================
// HELPERS PARA REGISTRAR MÉTRICAS NOS HANDLERS
// =============================================================================

/// Registra tentativa de login
pub fn record_login_attempt(success: bool) {
    let result = if success { "success" } else { "failure" };
    LOGIN_ATTEMPTS_TOTAL.with_label_values(&[result]).inc();
}

/// Registra renovação de token
pub fn record_token_refresh(success: bool) {
    let result = if success { "success" } else { "failure" };
    TOKEN_REFRESH_TOTAL.with_label_values(&[result]).inc();
}

/// Registra hit/miss no cache de políticas
pub fn record_cache_hit(is_hit: bool) {
    let result = if is_hit { "hit" } else { "miss" };
    POLICY_CACHE_HITS.with_label_values(&[result]).inc();
}

/// Atualiza contagem de políticas do Casbin
pub fn update_policy_count(count: i64) {
    CASBIN_POLICIES_COUNT.set(count);
}

/// Registra erro HTTP por tipo
pub fn record_http_error(error_type: &str) {
    HTTP_ERRORS_TOTAL.with_label_values(&[error_type]).inc();
}

// =============================================================================
// TESTES
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_login_attempt() {
        record_login_attempt(true);
        record_login_attempt(false);

        let metrics = prometheus::gather();
        let login_metric = metrics
            .iter()
            .find(|m| m.get_name() == "login_attempts_total")
            .expect("Métrica login_attempts_total não encontrada");

        assert_eq!(login_metric.get_metric().len(), 2); // success e failure
    }

    #[test]
    fn test_record_cache_hit() {
        record_cache_hit(true);
        record_cache_hit(false);

        let metrics = prometheus::gather();
        let cache_metric = metrics
            .iter()
            .find(|m| m.get_name() == "policy_cache_hits_total")
            .expect("Métrica policy_cache_hits_total não encontrada");

        assert_eq!(cache_metric.get_metric().len(), 2); // hit e miss
    }

    #[test]
    fn test_update_policy_count() {
        update_policy_count(42);

        let metrics = prometheus::gather();
        let policy_metric = metrics
            .iter()
            .find(|m| m.get_name() == "casbin_policies_count")
            .expect("Métrica casbin_policies_count não encontrada");

        let gauge_value = policy_metric.get_metric()[0].get_gauge().get_value();
        assert_eq!(gauge_value, 42.0);
    }
}
