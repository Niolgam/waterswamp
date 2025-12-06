use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// City entity - represents a city within a state
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct City {
    pub id: Uuid,
    pub name: String,
    pub state_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCityDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    pub state_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCityDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,

    pub state_id: Option<Uuid>,
}
