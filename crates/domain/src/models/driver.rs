use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "driver_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriverType {
    Outsourced,
    Server,
}

// ============================
// Driver DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct DriverDto {
    pub id: Uuid,
    pub driver_type: DriverType,
    pub full_name: String,
    pub cpf: String,
    pub cnh_number: String,
    pub cnh_category: String,
    pub cnh_expiration: NaiveDate,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDriverPayload {
    pub driver_type: DriverType,
    pub full_name: String,
    pub cpf: String,
    pub cnh_number: String,
    pub cnh_category: String,
    pub cnh_expiration: NaiveDate,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateDriverPayload {
    pub driver_type: Option<DriverType>,
    pub full_name: Option<String>,
    pub cpf: Option<String>,
    pub cnh_number: Option<String>,
    pub cnh_category: Option<String>,
    pub cnh_expiration: Option<NaiveDate>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}
