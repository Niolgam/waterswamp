use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Formato de log configurável
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
