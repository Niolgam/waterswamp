use domain::models::UserDtoExtended;
use domain::value_objects::{Email, Username}; // Importando tipos fortes
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    /// Nome de usuário
    pub username: String,
    /// Email do usuário
    pub email: String,
    /// Papel do usuário (user, admin, etc.)
    pub role: String,
    /// Indica se o email foi verificado
    pub email_verified: bool,
    /// Indica se MFA está habilitado
    pub mfa_enabled: bool,
}

impl From<UserDtoExtended> for ProfileResponse {
    fn from(user: UserDtoExtended) -> Self {
        Self {
            username: user.username.as_str().to_string(),
            email: user.email.as_str().to_string(),
            role: user.role,
            email_verified: user.email_verified,
            mfa_enabled: user.mfa_enabled,
        }
    }
}

#[derive(Deserialize, Validate, Debug, ToSchema)]
pub struct UpdateProfileRequest {
    /// Novo nome de usuário (opcional)
    pub username: Option<Username>, // Tipo forte opcional
    /// Novo email (opcional)
    pub email: Option<Email>,       // Tipo forte opcional
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    /// Senha atual
    #[validate(length(min = 1))]
    pub current_password: String,

    /// Nova senha (mínimo 8 caracteres)
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Serialize, ToSchema)]
pub struct ChangePasswordResponse {
    /// Mensagem de confirmação
    pub message: String,
}

impl Default for ChangePasswordResponse {
    fn default() -> Self {
        Self {
            message: "Senha alterada com sucesso.".to_string(),
        }
    }
}
