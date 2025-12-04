use crate::value_objects::{Email, Username};
use async_trait::async_trait;

#[async_trait]
pub trait EmailServicePort: Send + Sync {
    async fn send_verification_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String>;
    async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), String>;
    async fn send_password_reset_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String>;
    async fn send_mfa_enabled_email(&self, to: &Email, username: &Username) -> Result<(), String>;
}
