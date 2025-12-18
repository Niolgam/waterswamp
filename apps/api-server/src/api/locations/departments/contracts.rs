use domain::value_objects::LocationName;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================
// Department Category Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentCategoryResponse {
    pub id: Uuid,
    pub name: LocationName,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
