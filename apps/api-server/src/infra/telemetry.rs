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

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

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

    /// Histograma de tempo de hash de senha (em segundos)
    /// Importante para detectar ataques de força bruta e monitorar performance
    pub static ref PASSWORD_HASH_DURATION: HistogramVec = register_histogram_vec!(
        "password_hash_duration_seconds",
        "Tempo de hash de senha usando Argon2",
        &["operation"],  // "hash" ou "verify"
        vec![0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 1.0]
    ).unwrap();

    /// Contador de detecções de roubo de tokens (token theft detection)
    /// Incrementado quando tokens reutilizados ou famílias invalidadas são detectadas
    pub static ref TOKEN_THEFT_DETECTED: IntCounterVec = register_int_counter_vec!(
        "token_theft_detected_total",
        "Total de detecções de possível roubo de tokens",
        &["detection_type"]  // "reuse", "family_invalidation"
    ).unwrap();

    /// Histograma de tempo de verificação de permissões do Casbin (em segundos)
    pub static ref CASBIN_ENFORCEMENT_DURATION: HistogramVec = register_histogram_vec!(
        "casbin_enforcement_duration_seconds",
        "Tempo de verificação de permissões no Casbin",
        &["result"],  // "allowed" ou "denied"
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]
    ).unwrap();

    /// Contador de operações MFA
    pub static ref MFA_OPERATIONS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "mfa_operations_total",
        "Total de operações MFA",
        &["operation", "result"]  // operation: "enable", "disable", "verify"; result: "success", "failure"
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

/// Registra tempo de operação de hash de senha
pub fn record_password_operation_duration(operation: &str, duration_secs: f64) {
    PASSWORD_HASH_DURATION
        .with_label_values(&[operation])
        .observe(duration_secs);
}

/// Registra detecção de possível roubo de token
pub fn record_token_theft_detection(detection_type: &str) {
    TOKEN_THEFT_DETECTED
        .with_label_values(&[detection_type])
        .inc();
}

/// Registra tempo de enforcement do Casbin
pub fn record_casbin_enforcement_duration(allowed: bool, duration_secs: f64) {
    let result = if allowed { "allowed" } else { "denied" };
    CASBIN_ENFORCEMENT_DURATION
        .with_label_values(&[result])
        .observe(duration_secs);
}

/// Registra operação MFA
pub fn record_mfa_operation(operation: &str, success: bool) {
    let result = if success { "success" } else { "failure" };
    MFA_OPERATIONS_TOTAL
        .with_label_values(&[operation, result])
        .inc();
}

// =============================================================================
// TESTES
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Formato de texto legível para humanos (com cores no terminal)
    /// Ideal para desenvolvimento local
    Text,
    /// Formato JSON estruturado
    /// Ideal para produção e ferramentas de análise (Datadog, ELK, Grafana Loki)
    Json,
}

impl LogFormat {
    /// Detecta o formato baseado em variável de ambiente
    /// Prioridade: RUST_LOG_FORMAT > ENVIRONMENT
    pub fn from_env() -> Self {
        // 1. Tenta RUST_LOG_FORMAT primeiro
        if let Ok(format) = std::env::var("RUST_LOG_FORMAT") {
            return match format.to_lowercase().as_str() {
                "json" => LogFormat::Json,
                "text" => LogFormat::Text,
                _ => {
                    eprintln!(
                        "⚠️  RUST_LOG_FORMAT inválido: '{}'. Usando 'text' como padrão.",
                        format
                    );
                    LogFormat::Text
                }
            };
        }

        // 2. Fallback para ENVIRONMENT
        match std::env::var("ENVIRONMENT") {
            Ok(env) if env == "production" => LogFormat::Json,
            Ok(env) if env == "staging" => LogFormat::Json,
            _ => LogFormat::Text, // development ou não definido
        }
    }
}

/// Configuração de logging
pub struct LoggingConfig {
    /// Formato de saída (text ou json)
    pub format: LogFormat,
    /// Filtro de nível de log (ex: "info", "debug", "warn")
    pub level: String,
    /// Incluir timestamps nos logs
    pub with_timestamp: bool,
    /// Incluir nome do target (módulo) nos logs
    pub with_target: bool,
    /// Incluir número da linha nos logs (útil para debug)
    pub with_line_number: bool,
    /// Incluir informações de span (para tracing distribuído)
    pub with_span_events: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::from_env(),
            level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            with_timestamp: true,
            with_target: true,
            with_line_number: false, // Desabilitado por padrão (overhead)
            with_span_events: false, // Desabilitado por padrão
        }
    }
}

impl LoggingConfig {
    /// Cria configuração otimizada para desenvolvimento
    pub fn development() -> Self {
        Self {
            format: LogFormat::Text,
            level: "debug".to_string(),
            with_timestamp: true,
            with_target: true,
            with_line_number: true, // Útil em dev
            with_span_events: true,
        }
    }

    /// Cria configuração otimizada para produção
    pub fn production() -> Self {
        Self {
            format: LogFormat::Json,
            level: "info".to_string(),
            with_timestamp: true,
            with_target: true,
            with_line_number: false, // Overhead desnecessário em prod
            with_span_events: false,
        }
    }

    /// Cria configuração personalizada
    pub fn custom(format: LogFormat, level: &str) -> Self {
        Self {
            format,
            level: level.to_string(),
            ..Default::default()
        }
    }
}

/// Inicializa o sistema de logging
/// Deve ser chamado uma vez no início da aplicação
pub fn init_logging(config: LoggingConfig) -> anyhow::Result<()> {
    // Cria o filtro de ambiente
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    match config.format {
        LogFormat::Text => init_text_logging(config, env_filter),
        LogFormat::Json => init_json_logging(config, env_filter),
    }
}

/// Inicializa logging em formato de texto (desenvolvimento)
fn init_text_logging(config: LoggingConfig, env_filter: EnvFilter) -> anyhow::Result<()> {
    let fmt_layer = fmt::layer()
        .with_target(config.with_target)
        .with_line_number(config.with_line_number)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true) // Cores no terminal
        .pretty(); // Formato bonito e legível

    let fmt_layer = if config.with_span_events {
        fmt_layer.with_span_events(FmtSpan::FULL)
    } else {
        fmt_layer.with_span_events(FmtSpan::NONE)
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    tracing::info!(
        "📝 Logging inicializado (formato: TEXT, nível: {})",
        config.level
    );

    Ok(())
}

/// Inicializa logging em formato JSON (produção)
fn init_json_logging(config: LoggingConfig, env_filter: EnvFilter) -> anyhow::Result<()> {
    let fmt_layer = fmt::layer()
        .with_target(config.with_target)
        .with_line_number(config.with_line_number)
        .with_thread_ids(false)
        .with_thread_names(false)
        .json(); // Formato JSON

    let fmt_layer = if config.with_span_events {
        fmt_layer.with_span_events(FmtSpan::FULL)
    } else {
        fmt_layer.with_span_events(FmtSpan::NONE)
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    // Log inicial em JSON
    tracing::info!(
        log_format = "json",
        log_level = %config.level,
        "Logging inicializado"
    );

    Ok(())
}

// Helper para criar logs estruturados com campos customizados
// Útil para logging de eventos importantes com contexto
//
// # Exemplo
// ```rust
// use waterswamp::logging::log_event;
//
// log_event!(
//     Level::INFO,
//     "user_login",
//     user_id = "123",
//     ip_address = "192.168.1.1",
//     "Usuário fez login com sucesso"
// );
// ```
#[macro_export]
macro_rules! log_event {
    ($level:expr, $event_type:expr, $($key:ident = $value:expr),* $(,)?, $message:expr) => {
        match $level {
            tracing::Level::ERROR => {
                tracing::error!(
                    event_type = $event_type,
                    $($key = ?$value),*,
                    $message
                );
            }
            tracing::Level::WARN => {
                tracing::warn!(
                    event_type = $event_type,
                    $($key = ?$value),*,
                    $message
                );
            }
            tracing::Level::INFO => {
                tracing::info!(
                    event_type = $event_type,
                    $($key = ?$value),*,
                    $message
                );
            }
            tracing::Level::DEBUG => {
                tracing::debug!(
                    event_type = $event_type,
                    $($key = ?$value),*,
                    $message
                );
            }
            tracing::Level::TRACE => {
                tracing::trace!(
                    event_type = $event_type,
                    $($key = ?$value),*,
                    $message
                );
            }
        }
    };
}

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

    #[test]
    fn test_log_format_from_env() {
        // Salva variáveis originais
        let original_format = std::env::var("RUST_LOG_FORMAT").ok();
        let original_env = std::env::var("ENVIRONMENT").ok();

        // Teste 1: RUST_LOG_FORMAT=json
        std::env::set_var("RUST_LOG_FORMAT", "json");
        assert_eq!(LogFormat::from_env(), LogFormat::Json);

        // Teste 2: RUST_LOG_FORMAT=text
        std::env::set_var("RUST_LOG_FORMAT", "text");
        assert_eq!(LogFormat::from_env(), LogFormat::Text);

        // Teste 3: ENVIRONMENT=production (sem RUST_LOG_FORMAT)
        std::env::remove_var("RUST_LOG_FORMAT");
        std::env::set_var("ENVIRONMENT", "production");
        assert_eq!(LogFormat::from_env(), LogFormat::Json);

        // Teste 4: ENVIRONMENT=development
        std::env::set_var("ENVIRONMENT", "development");
        assert_eq!(LogFormat::from_env(), LogFormat::Text);

        // Restaura variáveis
        match original_format {
            Some(val) => std::env::set_var("RUST_LOG_FORMAT", val),
            None => std::env::remove_var("RUST_LOG_FORMAT"),
        }
        match original_env {
            Some(val) => std::env::set_var("ENVIRONMENT", val),
            None => std::env::remove_var("ENVIRONMENT"),
        }
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(config.with_timestamp);
        assert!(config.with_target);
    }

    #[test]
    fn test_logging_config_development() {
        let config = LoggingConfig::development();
        assert_eq!(config.format, LogFormat::Text);
        assert_eq!(config.level, "debug");
        assert!(config.with_line_number);
    }

    #[test]
    fn test_logging_config_production() {
        let config = LoggingConfig::production();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "info");
        assert!(!config.with_line_number);
    }
}
