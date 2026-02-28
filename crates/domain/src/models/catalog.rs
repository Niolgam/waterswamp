use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Unit of Measure DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UnitOfMeasureDto {
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub is_base_unit: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUnitOfMeasurePayload {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub is_base_unit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUnitOfMeasurePayload {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub is_base_unit: Option<bool>,
}

// ============================
// Unit Conversion DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UnitConversionDto {
    pub id: Uuid,
    pub from_unit_id: Uuid,
    pub to_unit_id: Uuid,
    pub conversion_factor: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnitConversionWithDetailsDto {
    pub id: Uuid,
    pub from_unit_id: Uuid,
    pub from_unit_name: String,
    pub from_unit_symbol: String,
    pub to_unit_id: Uuid,
    pub to_unit_name: String,
    pub to_unit_symbol: String,
    pub conversion_factor: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUnitConversionPayload {
    pub from_unit_id: Uuid,
    pub to_unit_id: Uuid,
    pub conversion_factor: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUnitConversionPayload {
    pub conversion_factor: rust_decimal::Decimal,
}

// ============================
// CATMAT DTOs (Catálogo de Materiais)
// ============================

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatGroupDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatGroupPayload {
    pub code: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatGroupPayload {
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Classes ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatClassDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatClassWithDetailsDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub budget_classification_name: Option<String>,
    pub budget_classification_code: Option<String>,
    pub is_active: bool,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatClassPayload {
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatClassPayload {
    pub group_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

// --- Items (PDM) ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatItemDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub code: String,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub is_sustainable: bool,
    pub specification: Option<String>,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_permanent: bool,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatItemWithDetailsDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub class_name: String,
    pub class_code: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub unit_of_measure_id: Uuid,
    pub unit_name: String,
    pub unit_symbol: String,
    pub code: String,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub is_sustainable: bool,
    pub specification: Option<String>,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_permanent: bool,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatItemPayload {
    pub class_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub code: String,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub is_sustainable: bool,
    pub specification: Option<String>,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_permanent: bool,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatItemPayload {
    pub class_id: Option<Uuid>,
    pub unit_of_measure_id: Option<Uuid>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub supplementary_description: Option<String>,
    pub is_sustainable: Option<bool>,
    pub specification: Option<String>,
    pub estimated_value: Option<rust_decimal::Decimal>,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_permanent: Option<bool>,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: Option<bool>,
    pub is_active: Option<bool>,
}

// --- Tree ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatGroupTreeNode {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub classes: Vec<CatmatClassTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatClassTreeNode {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub budget_classification_name: Option<String>,
    pub is_active: bool,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// CATSER DTOs (Catálogo de Serviços)
// ============================

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserGroupDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserGroupPayload {
    pub code: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserGroupPayload {
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Classes ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserClassDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserClassWithDetailsDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub budget_classification_name: Option<String>,
    pub budget_classification_code: Option<String>,
    pub is_active: bool,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserClassPayload {
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserClassPayload {
    pub group_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

// --- Items (Serviços) ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserItemDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub code: String,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserItemWithDetailsDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub class_name: String,
    pub class_code: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub unit_of_measure_id: Uuid,
    pub unit_name: String,
    pub unit_symbol: String,
    pub code: String,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserItemPayload {
    pub class_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub code: String,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserItemPayload {
    pub class_id: Option<Uuid>,
    pub unit_of_measure_id: Option<Uuid>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub estimated_value: Option<rust_decimal::Decimal>,
    pub search_links: Option<String>,
    pub is_active: Option<bool>,
}

// --- Tree ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserGroupTreeNode {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub classes: Vec<CatserClassTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserClassTreeNode {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub budget_classification_id: Option<Uuid>,
    pub budget_classification_name: Option<String>,
    pub is_active: bool,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
