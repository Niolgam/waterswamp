use async_trait::async_trait;
use std::sync::Arc;
use tera::Context as TeraContext;
use tokio::sync::Mutex;

use crate::EmailSender;

/// Mock email for testing purposes
#[derive(Clone, Debug)]
pub struct MockEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub context: TeraContext,
}

/// Mock email service for testing
///
/// Stores all sent emails in memory for verification in tests
#[derive(Clone, Default)]
pub struct MockEmailService {
    pub messages: Arc<Mutex<Vec<MockEmail>>>,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all sent emails
    ///
    /// Useful for assertions in tests
    pub async fn get_messages(&self) -> Vec<MockEmail> {
        self.messages.lock().await.clone()
    }

    /// Clear all sent emails
    ///
    /// Should be called between tests to reset state
    pub async fn clear(&self) {
        self.messages.lock().await.clear();
    }

    /// Get count of sent emails
    ///
    /// Quick way to verify number of emails sent
    pub async fn count(&self) -> usize {
        self.messages.lock().await.len()
    }

    /// Find emails by recipient
    ///
    /// Returns all emails sent to the specified address
    pub async fn find_by_recipient(&self, to: &str) -> Vec<MockEmail> {
        self.messages
            .lock()
            .await
            .iter()
            .filter(|email| email.to == to)
            .cloned()
            .collect()
    }

    /// Find emails by template
    ///
    /// Returns all emails using the specified template
    pub async fn find_by_template(&self, template: &str) -> Vec<MockEmail> {
        self.messages
            .lock()
            .await
            .iter()
            .filter(|email| email.template == template)
            .cloned()
            .collect()
    }

    /// Get the last sent email
    ///
    /// Returns None if no emails have been sent
    pub async fn last_email(&self) -> Option<MockEmail> {
        self.messages.lock().await.last().cloned()
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

    // Implement test helper methods
    async fn get_sent_emails(&self) -> Vec<MockEmail> {
        self.messages.lock().await.clone()
    }

    async fn clear_sent_emails(&self) {
        self.messages.lock().await.clear();
    }
}
