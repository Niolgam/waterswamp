use crate::value_objects::Email;
use crate::value_objects::Username;
use serde::{Deserialize, Serialize};
use sqlx;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct LoginPayload {
    #[validate(length(min = 3, message = "Username muito curto"))]
    pub username: String,
    #[validate(length(min = 6, message = "Senha muito curta"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RefreshTokenPayload {
    #[validate(length(min = 1, message = "Refresh token não pode estar vazio"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl RefreshTokenResponse {
    pub fn new(access_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

// --- Register ---

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RegisterPayload {
    pub username: Username,
    pub email: Email,
    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

// --- Passwords ---

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ForgotPasswordPayload {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResetPasswordPayload {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(length(min = 10))]
    pub new_password: String,
}

// --- JWT Claims ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
    PasswordReset,
}

// --- Persistence Entity ---
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked: bool,
    pub family_id: Uuid,
    pub parent_token_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
