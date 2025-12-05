use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "auth_database_url")]
    pub auth_db: String,

    #[serde(rename = "logs_database_url")]
    pub logs_db: String,

    pub jwt_private_key: String,
    pub jwt_public_key: String,

    // Token Expiry Configuration
    /// Tempo de expiração do Access Token em segundos (default: 3600 = 1h)
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry_seconds: i64,

    /// Tempo de expiração do Refresh Token em segundos (default: 604800 = 7 dias)
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry_seconds: i64,

    /// Tempo de expiração do Password Reset Token em segundos (default: 900 = 15 min)
    #[serde(default = "default_password_reset_expiry")]
    pub password_reset_expiry_seconds: i64,

    /// Tempo de expiração do MFA Challenge Token em segundos (default: 300 = 5 min)
    #[serde(default = "default_mfa_challenge_expiry")]
    pub mfa_challenge_expiry_seconds: i64,

    /// Tempo de expiração do Email Verification Token em segundos (default: 86400 = 24h)
    #[serde(default = "default_email_verification_expiry")]
    pub email_verification_expiry_seconds: i64,
}

// Default functions for token expiry
fn default_access_token_expiry() -> i64 {
    3600 // 1 hora
}

fn default_refresh_token_expiry() -> i64 {
    604800 // 7 dias
}

fn default_password_reset_expiry() -> i64 {
    900 // 15 minutos
}

fn default_mfa_challenge_expiry() -> i64 {
    300 // 5 minutos
}

fn default_email_verification_expiry() -> i64 {
    86400 // 24 horas
}

impl Config {
    /// Carrega a configuração do ambiente
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenv().ok();
        // Inicializa o builder de configuração
        let cfg = config::Config::builder()
            // 1. Adiciona variáveis de ambiente (ex: WS_AUTH_DATABASE_URL)
            // O prefixo 'WS' e o separador '_' são definidos aqui
            .add_source(
                config::Environment::with_prefix("WS")
                    .prefix_separator("_")
                    .separator("__"), // Permite aninhamento (ex: WS_DB__URL)
            )
            // 2. (Opcional) Carrega o .env se 'config' não o fez
            // .add_source(config::File::with_name(".env").required(false))
            .build()?;

        // Deserializa na nossa struct Config
        cfg.try_deserialize()
    }
}
