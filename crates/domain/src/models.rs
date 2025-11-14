use chrono;
use lazy_static;
use regex;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx;
use uuid::Uuid;
use validator::Validate;

lazy_static::lazy_static! {
    /// Regex estático para validação de papéis
    static ref ROLE_REGEX: Regex =
        Regex::new(r"^(admin|user)$").unwrap();
}

// =============================================================================
// AUTH - Payloads e Respostas
// (Assumindo que estes já cá estavam)
// =============================================================================

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

// =============================================================================
// USER
// (Assumindo que estes já cá estavam)
// =============================================================================

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RegisterPayload {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

// =============================================================================
// JWT CLAIMS
// (Assumindo que estes já cá estavam)
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
// (Assumindo que estes já cá estavam)
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

// =============================================================================
// ADMIN - User CRUD (TAREFA 4)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

/// DTO de usuário (sem informações sensíveis)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO detalhado do usuário, incluindo papéis
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailDto {
    #[serde(flatten)]
    pub user: UserDto,
    /// Lista de papéis (ex: "admin", "user") atribuídos a este usuário
    pub roles: Vec<String>,
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

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    // ⭐ CORREÇÃO: A macro #[validate] vai agora encontrar o "ROLE_REGEX"
    // que foi definido no topo do ficheiro.
    #[validate(
        length(min = 1),
        regex(path = *ROLE_REGEX, message = "O papel deve ser 'admin' ou 'user'")
    )]
    pub role: String,
}

/// Payload para atualizar usuário
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUserPayload {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,

    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,

    #[validate(length(min = 8))]
    pub password: Option<String>,

    // ⭐ CORREÇÃO: Idem
    #[validate(
        length(min = 1),
        regex(path = *ROLE_REGEX, message = "O papel deve ser 'admin' ou 'user'")
    )]
    pub role: Option<String>,
}
