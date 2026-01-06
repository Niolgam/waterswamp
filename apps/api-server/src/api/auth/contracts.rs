use domain::value_objects::{Email, Username};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use validator::Validate;

// REQUESTS

#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Nome de usuário ou email
    #[validate(length(min = 3))]
    pub username: String,
    /// Senha do usuário
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// Nome de usuário único
    pub username: Username,
    /// Endereço de email válido
    pub email: Email,

    /// Senha (mínimo 8 caracteres)
    // ADICIONADO: message customizada para passar no teste "weak_password"
    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct RefreshTokenRequest {
    /// Refresh token válido
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct LogoutRequest {
    /// Refresh token a ser revogado
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    /// Email cadastrado na conta
    pub email: Email,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    /// Token recebido por email
    #[validate(length(min = 1))]
    pub token: String,

    /// Nova senha (mínimo 8 caracteres)
    // ADICIONADO: message customizada para passar no teste "reset_password_weak"
    #[validate(length(min = 8, message = "Nova senha deve ter no mínimo 8 caracteres"))]
    pub new_password: String,
}

// RESPONSES

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    /// Token de acesso JWT
    pub access_token: String,
    /// Token de atualização
    pub refresh_token: String,
    /// Tipo do token (sempre "Bearer")
    pub token_type: String,
    /// Tempo de expiração em segundos
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

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    /// Token de acesso JWT
    pub access_token: String,
    /// Token de atualização
    pub refresh_token: String,
    /// Tipo do token (sempre "Bearer")
    pub token_type: String,
    /// Tempo de expiração em segundos
    pub expires_in: i64,
    /// ID do usuário criado
    pub user_id: Uuid,
    /// Nome de usuário
    pub username: String,
    /// Email do usuário
    pub email: String,
    /// Mensagem de sucesso
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

#[derive(Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    /// Novo token de acesso JWT
    pub access_token: String,
    /// Novo token de atualização
    pub refresh_token: String,
    /// Tipo do token (sempre "Bearer")
    pub token_type: String,
    /// Tempo de expiração em segundos
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

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    /// Mensagem de confirmação
    pub message: String,
}

impl Default for LogoutResponse {
    fn default() -> Self {
        Self {
            message: "Logout realizado com sucesso.".to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForgotPasswordResponse {
    /// Mensagem genérica (não revela se o email existe)
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

#[derive(Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    /// Mensagem de confirmação
    pub message: String,
}

impl Default for ResetPasswordResponse {
    fn default() -> Self {
        Self {
            message: "Senha redefinida com sucesso.".to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct MfaRequiredResponse {
    /// Indica que MFA é necessário
    pub mfa_required: bool,
    /// Token de desafio MFA
    pub mfa_token: String,
    /// Mensagem informativa
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
