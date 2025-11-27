use async_trait::async_trait;
use std::sync::Arc;
use tera::Context as TeraContext;
use tokio::sync::Mutex;

// ============================================================================
// TRAIT EmailSender
// ============================================================================

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        context: TeraContext,
    ) -> anyhow::Result<()>;

    fn send_welcome_email(&self, to_email: String, username: &str);
    fn send_password_reset_email(&self, to_email: String, username: &str, token: &str);
    fn send_verification_email(&self, to_email: String, username: &str, token: &str);
    fn send_mfa_enabled_email(&self, to_email: String, username: &str);

    // Métodos para testes (implementação padrão vazia)
    async fn get_sent_emails(&self) -> Vec<SentEmail> {
        vec![]
    }

    async fn clear_sent_emails(&self) {
        // Default vazio
    }
}

// ============================================================================
// SentEmail (para testes)
// ============================================================================

#[derive(Clone, Debug)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
}

// ============================================================================
// MockEmailService (para testes)
// ============================================================================

#[derive(Clone, Debug)]
pub struct MockEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub context: TeraContext,
}

#[derive(Clone, Default)]
pub struct MockEmailService {
    messages: Arc<Mutex<Vec<MockEmail>>>,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EmailSender for MockEmailService {
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        context: TeraContext,
    ) -> anyhow::Result<()> {
        let mut guard = self.messages.lock().await;
        guard.push(MockEmail {
            to: to_email,
            subject,
            template: template.to_string(),
            context,
        });
        Ok(())
    }

    fn send_welcome_email(&self, to_email: String, username: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        let service = self.clone();
        let subject = "Bem-vindo ao Waterswamp!".to_string();
        let template = "welcome.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    fn send_password_reset_email(&self, to_email: String, username: &str, token: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        let reset_link = format!("http://mock.test/reset?token={}", token);
        context.insert("reset_link", &reset_link);
        let service = self.clone();
        let subject = "Redefina sua senha do Waterswamp".to_string();
        let template = "reset_password.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    fn send_verification_email(&self, to_email: String, username: &str, token: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        let verification_link = format!("http://mock.test/verify-email?token={}", token);
        context.insert("verification_link", &verification_link);
        let service = self.clone();
        let subject = "Verifique seu email - Waterswamp".to_string();
        let template = "email_verification.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    fn send_mfa_enabled_email(&self, to_email: String, username: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        context.insert("enabled_at", "2025-01-01 00:00:00 UTC");
        let service = self.clone();
        let subject = "MFA Ativado - Waterswamp".to_string();
        let template = "mfa_enabled.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    // Implementar métodos de teste
    async fn get_sent_emails(&self) -> Vec<SentEmail> {
        let guard = self.messages.lock().await;
        guard
            .iter()
            .map(|email| SentEmail {
                to: email.to.clone(),
                subject: email.subject.clone(),
                template: email.template.clone(),
            })
            .collect()
    }

    async fn clear_sent_emails(&self) {
        let mut guard = self.messages.lock().await;
        guard.clear();
    }
}
