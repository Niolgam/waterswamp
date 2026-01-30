use crate::errors::EmailError;
use crate::value_objects::{Email, Username};
use async_trait::async_trait;

/// Port for email service operations.
///
/// This trait defines the contract for sending transactional emails.
/// Implementations may use SMTP, external APIs, or mock services.
#[async_trait]
pub trait EmailServicePort: Send + Sync {
    /// Sends an email verification link to a new user.
    async fn send_verification_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), EmailError>;

    /// Sends a welcome email after successful registration/verification.
    async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), EmailError>;

    /// Sends a password reset link.
    async fn send_password_reset_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), EmailError>;

    /// Sends a notification that MFA has been enabled.
    async fn send_mfa_enabled_email(&self, to: &Email, username: &Username)
        -> Result<(), EmailError>;
}
