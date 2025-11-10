use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// AUTH - Payloads e Respostas
// =============================================================================

#[derive(Debug, Validate, Deserialize)]
pub struct LoginPayload {
    #[validate(length(min = 3, message = "Username muito curto"))]
    pub username: String,
    #[validate(length(min = 6, message = "Senha muito curta"))]
    pub password: String,
}

/// Resposta de login com access e refresh tokens
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

/// Payload para renovar token
#[derive(Debug, Validate, Deserialize)]
pub struct RefreshTokenPayload {
    #[validate(length(min = 1, message = "Refresh token não pode estar vazio"))]
    pub refresh_token: String,
}

/// Resposta ao renovar token
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

// =============================================================================
// USER
// =============================================================================

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
}

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterPayload {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,

    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

// Regex para username (apenas alfanuméricos e underscore)

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

// =============================================================================
// JWT CLAIMS
// =============================================================================

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
}

// =============================================================================
// POLICIES (CASBIN)
// =============================================================================

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PolicyRequest {
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub object: String,
    #[validate(length(min = 1))]
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

/// DTO de usuário (sem informações sensíveis)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resposta paginada de usuários
#[derive(Debug, Serialize)]
pub struct PaginatedUsers {
    pub users: Vec<UserDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Payload para criar usuário (admin)
#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserPayload {
    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(length(min = 8))]
    pub password: String,
}

/// Payload para atualizar usuário
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUserPayload {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,

    #[validate(length(min = 8))]
    pub password: Option<String>,
}
