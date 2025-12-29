use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::{CatmatCode, MaterialCode, UnitOfMeasure};

// ============================
// Material Group Models
// ============================

/// DTO completo do Grupo de Material retornado do banco de dados
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MaterialGroupDto {
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

/// Resposta paginada de grupos de materiais
#[derive(Debug, Serialize)]
pub struct PaginatedMaterialGroups {
    pub material_groups: Vec<MaterialGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query params para listagem de grupos de materiais
#[derive(Debug, Deserialize)]
pub struct ListMaterialGroupsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub is_personnel_exclusive: Option<bool>,
    pub is_active: Option<bool>,
}

/// Payload para criação de grupo de material
#[derive(Debug, Validate, Deserialize)]
pub struct CreateMaterialGroupPayload {
    pub code: MaterialCode,
    #[validate(length(min = 3, max = 200, message = "Nome deve ter entre 3 e 200 caracteres"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Descrição deve ter no máximo 1000 caracteres"))]
    pub description: Option<String>,
    #[validate(length(max = 200, message = "Elemento de despesa deve ter no máximo 200 caracteres"))]
    pub expense_element: Option<String>,
    pub is_personnel_exclusive: Option<bool>,
}

/// Payload para atualização de grupo de material
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateMaterialGroupPayload {
    pub code: Option<MaterialCode>,
    #[validate(length(min = 3, max = 200, message = "Nome deve ter entre 3 e 200 caracteres"))]
    pub name: Option<String>,
    #[validate(length(max = 1000, message = "Descrição deve ter no máximo 1000 caracteres"))]
    pub description: Option<String>,
    #[validate(length(max = 200, message = "Elemento de despesa deve ter no máximo 200 caracteres"))]
    pub expense_element: Option<String>,
    pub is_personnel_exclusive: Option<bool>,
    pub is_active: Option<bool>,
}

// ============================
// Material Models
// ============================

/// DTO completo do Material/Serviço retornado do banco de dados
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MaterialDto {
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

/// DTO com informações do grupo de material incluídas
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaterialWithGroupDto {
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

/// Resposta paginada de materiais
#[derive(Debug, Serialize)]
pub struct PaginatedMaterials {
    pub materials: Vec<MaterialWithGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query params para listagem de materiais
#[derive(Debug, Deserialize)]
pub struct ListMaterialsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub material_group_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

/// Payload para criação de material/serviço
#[derive(Debug, Validate, Deserialize)]
pub struct CreateMaterialPayload {
    pub material_group_id: Uuid,
    #[validate(length(min = 3, max = 200, message = "Denominação deve ter entre 3 e 200 caracteres"))]
    pub name: String,
    #[validate(range(min = 0.0, message = "Valor estimado deve ser maior ou igual a zero"))]
    pub estimated_value: rust_decimal::Decimal,
    pub unit_of_measure: UnitOfMeasure,
    #[validate(length(min = 10, max = 2000, message = "Especificação deve ter entre 10 e 2000 caracteres"))]
    pub specification: String,
    #[validate(length(max = 1000, message = "Links de busca devem ter no máximo 1000 caracteres"))]
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    #[validate(url(message = "URL da foto inválida"))]
    pub photo_url: Option<String>,
}

/// Payload para atualização de material/serviço
#[derive(Debug, Validate, Deserialize)]
pub struct UpdateMaterialPayload {
    pub material_group_id: Option<Uuid>,
    #[validate(length(min = 3, max = 200, message = "Denominação deve ter entre 3 e 200 caracteres"))]
    pub name: Option<String>,
    #[validate(range(min = 0.0, message = "Valor estimado deve ser maior ou igual a zero"))]
    pub estimated_value: Option<rust_decimal::Decimal>,
    pub unit_of_measure: Option<UnitOfMeasure>,
    #[validate(length(min = 10, max = 2000, message = "Especificação deve ter entre 10 e 2000 caracteres"))]
    pub specification: Option<String>,
    #[validate(length(max = 1000, message = "Links de busca devem ter no máximo 1000 caracteres"))]
    pub search_links: Option<String>,
    pub catmat_code: Option<CatmatCode>,
    #[validate(url(message = "URL da foto inválida"))]
    pub photo_url: Option<String>,
    pub is_active: Option<bool>,
}
