use axum::body::Body;
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use std::net::IpAddr;
use std::time::Duration;
use tower_governor::{
    errors::GovernorError,
    governor::GovernorConfigBuilder,
    key_extractor::{KeyExtractor, SmartIpKeyExtractor},
    GovernorLayer,
};

/// Extrator de IP robusto que SEMPRE retorna um IP válido
/// Em testes, retorna localhost; em produção, tenta extrair o IP real
#[derive(Clone, Copy)]
pub struct RobustIpExtractor;

impl KeyExtractor for RobustIpExtractor {
    type Key = IpAddr;

    fn extract<B>(&self, req: &axum::http::Request<B>) -> Result<Self::Key, GovernorError> {
        // Se rate limiting estiver desabilitado, sempre retorna localhost
        // Isso DEVE vir primeiro para garantir que testes funcionem
        if is_rate_limiting_disabled() {
            return Ok(IpAddr::from([127, 0, 0, 1]));
        }

        // Em produção, tenta usar o extrator padrão inteligente
        match SmartIpKeyExtractor.extract(req) {
            Ok(ip) => Ok(ip),
            Err(_) => {
                // Fallback para localhost se não conseguir extrair
                // Isso é mais seguro que falhar completamente
                Ok(IpAddr::from([127, 0, 0, 1]))
            }
        }
    }
}

/// Type alias para simplificar o retorno das funções.
pub type RateLimitLayer = GovernorLayer<RobustIpExtractor, NoOpMiddleware<QuantaInstant>, Body>;

/// Verifica se rate limiting está desabilitado (para testes)
pub fn is_rate_limiting_disabled() -> bool {
    std::env::var("DISABLE_RATE_LIMIT")
        .unwrap_or_default()
        .to_lowercase()
        == "true"
}

/// Rate limiter estrito para rotas de login (proteção contra brute-force)
pub fn login_rate_limiter() -> RateLimitLayer {
    let (period, burst) = if is_rate_limiting_disabled() {
        (Duration::from_millis(1), 10000)
    } else {
        (Duration::from_secs(10), 5)
    };

    let config = GovernorConfigBuilder::default()
        .key_extractor(RobustIpExtractor)
        .period(period)
        .burst_size(burst)
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

/// Rate limiter para rotas administrativas sensíveis
pub fn admin_rate_limiter() -> RateLimitLayer {
    let (period, burst) = if is_rate_limiting_disabled() {
        (Duration::from_millis(1), 10000)
    } else {
        (Duration::from_secs(2), 10)
    };

    let config = GovernorConfigBuilder::default()
        .key_extractor(RobustIpExtractor)
        .period(period)
        .burst_size(burst)
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

/// Rate limiter geral para a API autenticada
pub fn api_rate_limiter() -> RateLimitLayer {
    let (period, burst) = if is_rate_limiting_disabled() {
        (Duration::from_millis(1), 10000)
    } else {
        (Duration::from_millis(200), 50)
    };

    let config = GovernorConfigBuilder::default()
        .key_extractor(RobustIpExtractor)
        .period(period)
        .burst_size(burst)
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}
