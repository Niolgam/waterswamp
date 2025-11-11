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

/// Extrator de IP robusto que fornece um fallback para ambientes de teste
#[derive(Clone, Copy)]
pub struct RobustIpExtractor;

impl KeyExtractor for RobustIpExtractor {
    type Key = IpAddr;

    fn extract<B>(&self, req: &axum::http::Request<B>) -> Result<Self::Key, GovernorError> {
        // 1. Tenta usar o extrator padrão inteligente (headers, ConnectInfo, etc.)
        match SmartIpKeyExtractor.extract(req) {
            Ok(ip) => Ok(ip),
            Err(e) => {
                // 2. Se falhar, verifica se estamos em ambiente de teste (DISABLE_RATE_LIMIT=true)
                if is_rate_limiting_disabled() {
                    // Retorna um IP fictício (localhost) para permitir que o teste prossiga
                    Ok(IpAddr::from([127, 0, 0, 1]))
                } else {
                    // Em produção, se não conseguirmos identificar o IP, é mais seguro falhar
                    Err(e)
                }
            }
        }
    }
}

/// Type alias para simplificar o retorno das funções.
/// Usamos nosso RobustIpExtractor em vez do SmartIpKeyExtractor diretamente.
pub type RateLimitLayer = GovernorLayer<RobustIpExtractor, NoOpMiddleware<QuantaInstant>, Body>;

/// Verifica se rate limiting está desabilitado (para testes)
fn is_rate_limiting_disabled() -> bool {
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
        .key_extractor(RobustIpExtractor) // <--- Alterado aqui
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
        .key_extractor(RobustIpExtractor) // <--- Alterado aqui
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
        .key_extractor(RobustIpExtractor) // <--- Alterado aqui
        .period(period)
        .burst_size(burst)
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}
