use domain::value_objects::Email;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request para verificar o email com token
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyEmailRequest {
    #[validate(length(min = 1, message = "Token não pode estar vazio"))]
    pub token: String,
}

/// Response para verificação de email
#[derive(Debug, Serialize)]
pub struct EmailVerificationResponse {
    pub verified: bool,
    pub message: String,
}

/// Request para reenviar email de verificação
#[derive(Debug, Deserialize, Validate)]
pub struct ResendVerificationRequest {
    pub email: Email,
}

/// Response para status de verificação
#[derive(Debug, Serialize)]
pub struct VerificationStatusResponse {
    pub email_verified: bool,
}
