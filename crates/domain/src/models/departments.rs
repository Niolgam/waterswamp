use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::LocationName;

// ============================
// Department Category Models
// ============================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DepartmentCategoryDto {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListDepartmentCategoriesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateDepartmentCategoryPayload {
    pub name: LocationName,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateDepartmentCategoryPayload {
    pub name: Option<LocationName>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// ============================
// Department Models
// ============================
// TODO: Department models will be added when Phase 4 is implemented
