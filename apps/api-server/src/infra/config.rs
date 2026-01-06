use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "main_database_url")]
    pub main_db: String,

    #[serde(rename = "logs_database_url")]
    pub logs_db: String,

    #[serde(rename = "jwt_private_key")]
    pub jwt_private_key: String,

    #[serde(rename = "jwt_public_key")]
    pub jwt_public_key: String,
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
