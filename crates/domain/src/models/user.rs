use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{Email, Username};

lazy_static! {
    static ref ROLE_REGEX: Regex = Regex::new(r"^(admin|user)$").unwrap();
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub username: Username, // Value Object
    pub email: Email,       // Value Object
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct UserDtoExtended {
    pub id: Uuid,
    pub username: Username, // Value Object
    pub email: Email,       // Value Object
    pub role: String,
    pub email_verified: bool,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub mfa_enabled: bool,
    pub is_banned: bool,
    pub banned_at: Option<chrono::DateTime<chrono::Utc>>,
    pub banned_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO específico para login (contém informações de autenticação)
#[derive(Debug, FromRow)]
pub struct UserLoginInfo {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub mfa_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserDetailDto {
    #[serde(flatten)]
    pub user: UserDto,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedUsers {
    pub users: Vec<UserDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>, // Busca é string pura
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateUserPayload {
    pub username: Username,
    pub email: Email,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1), regex(path = *ROLE_REGEX))]
    pub role: String,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct UpdateUserPayload {
    // Usamos Option<ValueObject> para update parcial
    pub username: Option<Username>,
    pub email: Option<Email>,
    #[validate(length(min = 8))]
    pub password: Option<String>,
    #[validate(length(min = 1), regex(path = *ROLE_REGEX))]
    pub role: Option<String>,
}
