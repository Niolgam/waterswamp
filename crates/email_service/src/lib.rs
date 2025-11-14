// crates/email_service/src/lib.rs

pub mod config;

use crate::config::EmailConfig;
use anyhow::Context; // ⭐ RE-ADICIONADO: Usado em .with_context()
use async_trait::async_trait;
use htmlescape;
use lettre::{
    message::header::ContentType, // ⭐ USADO
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
        AsyncSmtpTransportBuilder,
    },
    AsyncSmtpTransport,
    AsyncTransport, // ⭐ USADO
    Message,
    Tokio1Executor, // O Executor correto
};
use once_cell::sync::Lazy;
use serde_json; // ⭐ RE-ADICIONADO: Usado no filtro
use std::collections::HashMap; // ⭐ RE-ADICIONADO: Usado no filtro
use tera::{Context as TeraContext, Tera, Value};
// use tokio::runtime::Handle; // REMOVIDO (Warning)

// Inicializa o Tera (motor de templates)
pub static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = match Tera::new("crates/email_service/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Falha ao carregar templates de email: {}", e);
            std::process::exit(1);
        }
    };

    // ⭐ CORREÇÃO (Erro FnOnce/Fn): Adicionar tipo explícito ao segundo argumento
    tera.register_filter(
        "safe_html",
        |value: &Value, _args: &HashMap<String, Value>| {
            let s = value.as_str().unwrap_or_default().replace('\n', "<br>");
            // CORREÇÃO (Erro E0425): Usar 'encode_minimal'
            Ok(serde_json::to_value(htmlescape::encode_minimal(&s)).unwrap())
        },
    );
    tera
});

// ===================================================================
// 1. A NOVA TRAIT (INTERFACE)
// ===================================================================
#[async_trait]
pub trait EmailSender {
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        // ⭐ CORREÇÃO AQUI: Adicionar 'mut'
        mut context: TeraContext,
    ) -> anyhow::Result<()>;

    fn send_welcome_email(&self, to_email: String, username: &str);
}

// ===================================================================
// 2. O SERVIÇO REAL (IMPLEMENTAÇÃO)
// ===================================================================

#[derive(Clone)]
pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl EmailService {
    /// Inicializa o EmailService
    pub fn new(config: EmailConfig) -> anyhow::Result<Self> {
        let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());

        // ⭐ CORREÇÃO (Erros E0624, E0599, E0061, E0107)
        // 1. O Executor é um tipo genérico no `relay()`, não um valor em `new()`.
        // 2. O `relay()` só aceita o 'host' como argumento.
        let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds);

        // TLS opcional (para o Mailtrap, mas não para mocks)
        if config.smtp_host != "127.0.0.1" {
            let tls_params = TlsParameters::new(config.smtp_host.clone())?;
            transport_builder = transport_builder.tls(Tls::Required(tls_params));
        }

        // 3. O .build() não aceita argumentos
        let transport = transport_builder.build();

        Ok(Self {
            transport,
            from_email: config.from_email,
        })
    }
}

#[async_trait]
impl EmailSender for EmailService {
    /// Método interno para renderizar e enviar
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        mut context: TeraContext,
    ) -> anyhow::Result<()> {
        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        context.insert("base_url", &base_url);

        // ⭐ USADO: `anyhow::Context`
        let html_body = TERA
            .render(template, &context)
            .with_context(|| format!("Falha ao renderizar template de email: {}", template))?;

        // ⭐ USADO: `Message`, `ContentType`
        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        // ⭐ USADO: `AsyncTransport`
        self.transport.send(email).await?;
        Ok(())
    }

    /// Envia um email de forma assíncrona ("fire-and-forget")
    fn send_welcome_email(&self, to_email: String, username: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);

        let service = self.clone();
        let subject = "Bem-vindo ao Waterswamp!".to_string();
        let template = "welcome.html".to_string();

        tokio::spawn(async move {
            tracing::info!(to = %to_email, template = %template, "A enviar email de boas-vindas...");
            if let Err(e) = service
                .send_email(to_email, subject, &template, context)
                .await
            {
                tracing::error!(error = ?e, "Falha ao enviar email");
            }
        });
    }
}
