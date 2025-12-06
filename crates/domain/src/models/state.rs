use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// State entity - represents a Brazilian state
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct State {
    pub id: Uuid,
    pub name: String,        // e.g., "São Paulo"
    pub code: String,        // e.g., "SP" (2 characters)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateStateDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(length(equal = 2, message = "Code must be exactly 2 characters"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateStateDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,
}
