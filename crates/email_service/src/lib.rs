pub mod config;

use crate::config::EmailConfig;
use anyhow::Context;
use htmlescape;
use lettre::{
    message::header::ContentType,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport,
    AsyncTransport,
    Message,
    Tokio1Executor, // O Executor correto
};
use once_cell::sync::Lazy;
use serde_json;
use std::collections::HashMap; // ⭐ CORREÇÃO: Importar o HashMap
use tera::{Context as TeraContext, Tera, Value};
// use tokio::runtime::Handle; // ⭐ CORREÇÃO: Removido (Warning/Erro)

// Inicializa o Tera (motor de templates)
pub static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = match Tera::new("crates/email_service/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Falha ao carregar templates de email: {}", e);
            std::process::exit(1);
        }
    };

    // ⭐ CORREÇÃO: Adicionar tipo explícito ao segundo argumento do closure
    tera.register_filter(
        "safe_html",
        |value: &Value, _args: &HashMap<String, Value>| {
            let s = value.as_str().unwrap_or_default().replace('\n', "<br>");
            // Usar 'encode_minimal' (como na correção anterior)
            Ok(serde_json::to_value(htmlescape::encode_minimal(&s)).unwrap())
        },
    );
    tera
});

// O EmailService será clonável e guardado no AppState
#[derive(Clone)]
pub struct EmailService {
    // Usar Tokio1Executor
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl EmailService {
    /// Inicializa o EmailService
    pub fn new(config: EmailConfig) -> anyhow::Result<Self> {
        let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());
        let tls_params = TlsParameters::new(config.smtp_host.clone())?;

        // ⭐ CORREÇÃO: O tipo de Executor é definido em ::relay()
        // E .build() não tem argumentos.
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds)
            .tls(Tls::Required(tls_params))
            .build(); // ⭐ CORREÇÃO AQUI

        Ok(Self {
            transport,
            from_email: config.from_email,
        })
    }

    /// Método interno para renderizar e enviar
    async fn send_email_task(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        context: TeraContext,
    ) -> anyhow::Result<()> {
        let html_body = TERA
            .render(template, &context)
            .with_context(|| format!("Falha ao renderizar template de email: {}", template))?;

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        // Esta chamada a .send() agora vai funcionar
        self.transport.send(email).await?;

        Ok(())
    }

    /// Envia um email de forma assíncrona ("fire-and-forget")
    pub fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        mut context: TeraContext,
    ) {
        let service = self.clone();
        let template_str = template.to_string();

        let base_url = std::env::var("APP_BASE_URL").unwrap_or("http://localhost:3000".to_string());
        context.insert("base_url", &base_url);

        // O 'spawn' agora vai funcionar
        tokio::spawn(async move {
            tracing::info!(to = %to_email, template = %template_str, "A enviar email...");
            if let Err(e) = service
                .send_email_task(to_email, subject, &template_str, context)
                .await
            {
                tracing::error!(error = ?e, "Falha ao enviar email");
            }
        });
    }

    // --- Helpers de Templates ---

    /// Envia email de Boas-Vindas
    pub fn send_welcome_email(&self, to_email: String, username: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);

        self.send_email(
            to_email,
            "Bem-vindo ao Waterswamp!".to_string(),
            "welcome.html",
            context,
        );
    }
}
