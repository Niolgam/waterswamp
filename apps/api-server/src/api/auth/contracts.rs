use domain::value_objects::{Email, Username};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// REQUESTS

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    pub username: Username,
    pub email: Email,

    // ADICIONADO: message customizada para passar no teste "weak_password"
    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

#[derive(Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    pub email: Email,
}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1))]
    pub token: String,

    // ADICIONADO: message customizada para passar no teste "reset_password_weak"
    #[validate(length(min = 8, message = "Nova senha deve ter no mínimo 8 caracteres"))]
    pub new_password: String,
}

// RESPONSES

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl LoginResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    // ADICIONADO: Campo message para passar no teste de sucesso
    pub message: String,
}

impl RegisterResponse {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user_id: Uuid,
        username: Username,
        email: String,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user_id,
            username: username.as_str().to_string(),
            email,
            // Valor padrão para a mensagem
            message: "Usuário criado com sucesso.".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl RefreshTokenResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

impl Default for LogoutResponse {
    fn default() -> Self {
        Self {
            message: "Logout realizado com sucesso.".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

impl Default for ForgotPasswordResponse {
    fn default() -> Self {
        Self {
            // Essa é a mensagem exata que está implementada e que o teste deve esperar
            message: "Se o email existir, você receberá instruções de recuperação.".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

impl Default for ResetPasswordResponse {
    fn default() -> Self {
        Self {
            message: "Senha redefinida com sucesso.".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub mfa_token: String,
    pub message: String,
}

impl MfaRequiredResponse {
    pub fn new(mfa_token: String) -> Self {
        Self {
            mfa_required: true,
            mfa_token,
            message: "Autenticação de dois fatores necessária.".to_string(),
        }
    }
}
