use domain::models::UserDtoExtended;
use domain::value_objects::{Email, Username}; // Importando tipos fortes
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize)]
pub struct ProfileResponse {
    pub username: String,
    pub email: String,
    pub role: String,
    pub email_verified: bool,
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

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateProfileRequest {
    pub username: Option<Username>, // Tipo forte opcional
    pub email: Option<Email>,       // Tipo forte opcional
}

#[derive(Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1))]
    pub current_password: String,

    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}

impl Default for ChangePasswordResponse {
    fn default() -> Self {
        Self {
            message: "Senha alterada com sucesso.".to_string(),
        }
    }
}
