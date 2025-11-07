use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct LoginPayload {
    #[validate(length(min = 3, message = "Username muito curto "))]
    pub username: String,
    #[validate(length(min = 6, message = "Senha muito curta"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    token: String,
}

impl LoginResponse {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PolicyRequest {
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub object: String,
    #[validate(length(min = 1))]
    pub action: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    // A função continua 'async fn'
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
