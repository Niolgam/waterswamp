use domain::value_objects::{CatmatCode, MaterialCode, UnitOfMeasure};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================
// Material Group Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialGroupResponse {
    pub id: Uuid,
    pub code: MaterialCode,
    pub name: String,
    pub description: Option<String>,
    pub expense_element: Option<String>,
    pub is_personnel_exclusive: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// Material Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialResponse {
    pub id: Uuid,
    pub material_group_id: Uuid,
    pub name: String,
    pub estimated_value: rust_decimal::Decimal,
    pub unit_of_measure: UnitOfMeasure,
    pub specification: String,
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    pub photo_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialWithGroupResponse {
    pub id: Uuid,
    pub material_group_id: Uuid,
    pub material_group_code: MaterialCode,
    pub material_group_name: String,
    pub name: String,
    pub estimated_value: rust_decimal::Decimal,
    pub unit_of_measure: UnitOfMeasure,
    pub specification: String,
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    pub photo_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
