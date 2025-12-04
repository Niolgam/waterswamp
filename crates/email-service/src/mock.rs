use crate::EmailSender;
use async_trait::async_trait;
use domain::ports::EmailServicePort;
use domain::value_objects::{Email, Username};
use std::sync::{Arc, Mutex};
use tera::Context;

/// Mock email for testing purposes
#[derive(Clone, Debug)]
pub struct MockEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub context: Context,
}

/// Mock email service for testing
#[derive(Clone, Default)]
pub struct MockEmailService {
    pub messages: Arc<Mutex<Vec<MockEmail>>>,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_messages(&self) -> Vec<MockEmail> {
        self.messages.lock().unwrap().clone()
    }

    pub async fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }
}

// --- IMPLEMENTAÇÃO DA PORTA DO DOMÍNIO ---
#[async_trait]
impl EmailServicePort for MockEmailService {
    async fn send_verification_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String> {
        // Delega para o método legado
        EmailSender::send_verification_email(
            self,
            to.as_str().to_string(),
            username.as_str(),
            token,
        );
        Ok(())
    }

    async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), String> {
        EmailSender::send_welcome_email(self, to.as_str().to_string(), username.as_str());
        Ok(())
    }

    async fn send_password_reset_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String> {
        EmailSender::send_password_reset_email(
            self,
            to.as_str().to_string(),
            username.as_str(),
            token,
        );
        Ok(())
    }

    async fn send_mfa_enabled_email(&self, to: &Email, username: &Username) -> Result<(), String> {
        EmailSender::send_mfa_enabled_email(self, to.as_str().to_string(), username.as_str());
        Ok(())
    }
}

// --- IMPLEMENTAÇÃO DO TRAIT LEGADO (USANDO TOKIO::SPAWN) ---
#[async_trait]
impl EmailSender for MockEmailService {
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        context: Context,
    ) -> anyhow::Result<()> {
        let mut guard = self.messages.lock().unwrap();
        guard.push(MockEmail {
            to: to_email,
            subject,
            template: template.to_string(),
            context,
        });
        Ok(())
    }

    // [FIX] Override default implementation to actually return messages
    async fn get_sent_emails(&self) -> Vec<MockEmail> {
        self.messages.lock().unwrap().clone()
    }

    // [FIX] Override default implementation to actually clear messages
    async fn clear_sent_emails(&self) {
        self.messages.lock().unwrap().clear();
    }

    fn send_welcome_email(&self, to_email: String, username: &str) {
        let service = self.clone();
        let username = username.to_string(); // Capture username
        tokio::spawn(async move {
            let mut ctx = Context::new();
            ctx.insert("username", &username); // Insert into context
            let _ = service
                .send_email(
                    to_email,
                    "Bem-vindo ao Waterswamp!".to_string(),
                    "welcome.html",
                    ctx,
                )
                .await;
        });
    }

    fn send_password_reset_email(&self, to_email: String, username: &str, token: &str) {
        let service = self.clone();
        let token = token.to_string();
        let username = username.to_string(); // Capture username
        tokio::spawn(async move {
            let mut ctx = Context::new();
            ctx.insert(
                "reset_link",
                &format!("http://mock.test/reset?token={}", token),
            );
            ctx.insert("username", &username); // Insert into context
            let _ = service
                .send_email(
                    to_email,
                    "Redefina sua senha do Waterswamp".to_string(),
                    "reset_password.html",
                    ctx,
                )
                .await;
        });
    }

    fn send_verification_email(&self, to_email: String, username: &str, token: &str) {
        let service = self.clone();
        let token = token.to_string();
        let username = username.to_string(); // Capture username
        tokio::spawn(async move {
            let mut ctx = Context::new();
            ctx.insert(
                "verification_link",
                &format!("http://mock.test/verify-email?token={}", token),
            );
            ctx.insert("username", &username); // Insert into context
            let _ = service
                .send_email(
                    to_email,
                    "Verifique seu email - Waterswamp".to_string(),
                    "email_verification.html",
                    ctx,
                )
                .await;
        });
    }

    fn send_mfa_enabled_email(&self, to_email: String, username: &str) {
        let service = self.clone();
        let username = username.to_string();
        tokio::spawn(async move {
            let mut ctx = Context::new();
            ctx.insert("username", &username);
            let _ = service
                .send_email(
                    to_email,
                    "MFA Ativado - Waterswamp".to_string(),
                    "mfa_enabled.html",
                    ctx,
                )
                .await;
        });
    }
}
