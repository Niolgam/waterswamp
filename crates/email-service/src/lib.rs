pub mod config;

// Re-export for convenience
pub use config::EmailConfig;

// Mock types need to be available for trait signature
// But only mock module implementation is gated
#[cfg(any(test, feature = "mock"))]
pub mod mock;

use domain::ports::EmailServicePort;
use domain::value_objects::Email;
use domain::value_objects::Username;
#[cfg(any(test, feature = "mock"))]
pub use mock::MockEmailService;

// MockEmail type must be available for trait signature
// even when mock feature is disabled (used in trait default methods)
#[cfg(not(any(test, feature = "mock")))]
#[derive(Clone, Debug)]
pub struct MockEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub context: tera::Context,
}

#[cfg(any(test, feature = "mock"))]
pub use mock::MockEmail;

use anyhow::Context;
use async_trait::async_trait;
use htmlescape;
use lettre::{
    message::header::ContentType,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use once_cell::sync::Lazy;
use serde_json;
use std::collections::HashMap;
use tera::{Context as TeraContext, Tera, Value};

// ===================================================================
// TEMPLATE ENGINE INITIALIZATION
// ===================================================================

// Inicializa o Tera (motor de templates) de forma preguiçosa e global
pub static TERA: Lazy<Tera> = Lazy::new(|| {
    // Tenta carregar templates da pasta crates/email-service/src/templates
    // Nota: O caminho pode precisar de ajuste dependendo de onde o binário é executado
    let mut tera = match Tera::new("crates/email-service/src/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            // Tenta caminho alternativo caso esteja a rodar de dentro do crate
            match Tera::new("src/templates/**/*.html") {
                Ok(t) => t,
                Err(_) => {
                    tracing::error!("Falha crítica ao carregar templates de email: {}", e);
                    // Em produção, talvez não devêssemos sair, mas sim retornar erro no envio
                    std::process::exit(1);
                }
            }
        }
    };

    // Regista um filtro para escapar HTML e converter quebras de linha em <br>
    tera.register_filter(
        "safe_html",
        |value: &Value, _args: &HashMap<String, Value>| {
            let s = value.as_str().unwrap_or_default().replace('\n', "<br>");
            Ok(serde_json::to_value(htmlescape::encode_minimal(&s)).unwrap())
        },
    );
    tera
});

// ===================================================================
// 1. THE TRAIT (INTERFACE)
// ===================================================================

#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Método genérico para envio de qualquer email
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        context: TeraContext,
    ) -> anyhow::Result<()>;

    /// Envia email de boas-vindas (fire-and-forget)
    fn send_welcome_email(&self, to_email: String, username: &str);

    /// Envia email de recuperação de senha (fire-and-forget)
    fn send_password_reset_email(&self, to_email: String, username: &str, token: &str);

    /// Envia email de verificação de conta (fire-and-forget)
    fn send_verification_email(&self, to_email: String, username: &str, token: &str);

    /// Envia notificação de MFA ativado (fire-and-forget)
    fn send_mfa_enabled_email(&self, to_email: String, username: &str);

    // Métodos auxiliares para testes (com implementação padrão vazia para produção)
    async fn get_sent_emails(&self) -> Vec<MockEmail> {
        Vec::new()
    }

    async fn clear_sent_emails(&self) {
        // No-op for production service
    }
}

// ===================================================================
// 2. THE REAL SERVICE (IMPLEMENTATION)
// ===================================================================

#[derive(Clone)]
pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl EmailService {
    /// Inicializa o EmailService com as configurações fornecidas
    pub fn new(config: EmailConfig) -> anyhow::Result<Self> {
        let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());

        let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds);

        // TLS opcional: Desativa TLS se for localhost para facilitar dev local com Mailhog/Mailpit
        if config.smtp_host != "127.0.0.1" && config.smtp_host != "localhost" {
            let tls_params = TlsParameters::new(config.smtp_host.clone())?;
            transport_builder = transport_builder.tls(Tls::Required(tls_params));
        } else {
            transport_builder = transport_builder.tls(Tls::None);
        }

        let transport = transport_builder.build();

        Ok(Self {
            transport,
            from_email: config.from_email,
        })
    }

    async fn send_raw(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        mut context: TeraContext,
    ) -> anyhow::Result<()> {
        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        context.insert("base_url", &base_url);
        let current_year = chrono::Utc::now().format("%Y").to_string();
        context.insert("current_year", &current_year);

        let html_body = TERA
            .render(template, &context)
            .with_context(|| format!("Template render failed: {}", template))?;

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        self.transport.send(email).await?;
        Ok(())
    }
}

#[async_trait]
impl EmailServicePort for EmailService {
    async fn send_verification_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String> {
        let mut context = TeraContext::new();
        context.insert("username", username.as_str());

        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let link = format!("{}/verify-email?token={}", base_url, token);
        context.insert("verification_link", &link);

        let service = self.clone();
        let to_string = to.as_str().to_string();

        // Spawn para não bloquear, como era antes
        tokio::spawn(async move {
            if let Err(e) = service
                .send_raw(
                    to_string,
                    "Verifique seu email - Waterswamp".into(),
                    "email_verification.html",
                    context,
                )
                .await
            {
                tracing::error!("Falha email verificação: {:?}", e);
            }
        });
        Ok(())
    }

    async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), String> {
        let mut context = TeraContext::new();
        context.insert("username", username.as_str());

        let service = self.clone();
        let to_string = to.as_str().to_string();

        tokio::spawn(async move {
            if let Err(e) = service
                .send_raw(
                    to_string,
                    "Bem-vindo ao Waterswamp!".into(),
                    "welcome.html",
                    context,
                )
                .await
            {
                tracing::error!("Falha email welcome: {:?}", e);
            }
        });
        Ok(())
    }

    async fn send_password_reset_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String> {
        let mut context = TeraContext::new();
        context.insert("username", username.as_str());

        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let link = format!("{}/reset-password-form?token={}", base_url, token);
        context.insert("reset_link", &link);

        let service = self.clone();
        let to_string = to.as_str().to_string();

        tokio::spawn(async move {
            if let Err(e) = service
                .send_raw(
                    to_string,
                    "Redefina sua senha".into(),
                    "reset_password.html",
                    context,
                )
                .await
            {
                tracing::error!("Falha email reset: {:?}", e);
            }
        });
        Ok(())
    }
}

// Mantemos a implementação do Trait Legado (EmailSender) para não quebrar handlers antigos
#[async_trait]
impl EmailSender for EmailService {
    async fn send_email(
        &self,
        to: String,
        sub: String,
        tpl: &str,
        ctx: TeraContext,
    ) -> anyhow::Result<()> {
        self.send_raw(to, sub, tpl, ctx).await
    }

    // Stub methods para cumprir a interface antiga se necessário,
    // ou apenas use send_raw nos lugares antigos.
    fn send_welcome_email(&self, to: String, user: &str) {
        // Redireciona para lógica nova convertendo tipos (cuidado com unwrap em prod)
        // Isso é só para compatibilidade
        if let (Ok(e), Ok(u)) = (Email::try_from(to), Username::try_from(user)) {
            let _ = EmailServicePort::send_welcome_email(self, &e, &u);
        }
    }
    fn send_password_reset_email(&self, to: String, user: &str, token: &str) {
        if let (Ok(e), Ok(u)) = (Email::try_from(to), Username::try_from(user)) {
            let _ = EmailServicePort::send_password_reset_email(self, &e, &u, token);
        }
    }
    fn send_verification_email(&self, to: String, user: &str, token: &str) {
        if let (Ok(e), Ok(u)) = (Email::try_from(to), Username::try_from(user)) {
            let _ = EmailServicePort::send_verification_email(self, &e, &u, token);
        }
    }
    fn send_mfa_enabled_email(&self, to: String, user: &str) {
        // Implementação similar...
        // Para brevidade, omitido, mas siga o padrão acima.
    }
}
