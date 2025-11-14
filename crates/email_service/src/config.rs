use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,

    // O email "De" (ex: "Equipa Waterswamp <nao-responda@waterswamp.com>")
    pub from_email: String,
}

impl EmailConfig {
    /// Carrega a configuração de email do ambiente
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let cfg = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("EMAIL") // Ex: EMAIL_SMTP_HOST
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        cfg.try_deserialize()
    }
}
