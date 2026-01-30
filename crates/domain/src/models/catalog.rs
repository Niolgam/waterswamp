use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "item_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemType {
    Material,
    Service,
}

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
// Catalog Group DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatalogGroupDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub item_type: ItemType,
    pub budget_classification_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogGroupWithDetailsDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub item_type: ItemType,
    pub budget_classification_id: Uuid,
    pub budget_classification_name: String,
    pub budget_classification_code: String,
    pub parent_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogGroupTreeNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub item_type: ItemType,
    pub budget_classification_id: Uuid,
    pub budget_classification_name: String,
    pub budget_classification_code: String,
    pub is_active: bool,
    #[schema(no_recursion)]
    pub children: Vec<CatalogGroupTreeNode>,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatalogGroupPayload {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub item_type: ItemType,
    pub budget_classification_id: Uuid,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatalogGroupPayload {
    pub parent_id: Option<Option<Uuid>>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub item_type: Option<ItemType>,
    pub budget_classification_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

// ============================
// Catalog Item DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatalogItemDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub name: String,
    pub catmat_code: Option<String>,
    pub specification: String,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_stockable: bool,
    pub is_permanent: bool,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogItemWithDetailsDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub unit_of_measure_id: Uuid,
    pub unit_name: String,
    pub unit_symbol: String,
    pub name: String,
    pub catmat_code: Option<String>,
    pub specification: String,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_stockable: bool,
    pub is_permanent: bool,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatalogItemPayload {
    pub group_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub name: String,
    pub catmat_code: Option<String>,
    pub specification: String,
    pub estimated_value: rust_decimal::Decimal,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_stockable: bool,
    pub is_permanent: bool,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatalogItemPayload {
    pub group_id: Option<Uuid>,
    pub unit_of_measure_id: Option<Uuid>,
    pub name: Option<String>,
    pub catmat_code: Option<String>,
    pub specification: Option<String>,
    pub estimated_value: Option<rust_decimal::Decimal>,
    pub search_links: Option<String>,
    pub photo_url: Option<String>,
    pub is_stockable: Option<bool>,
    pub is_permanent: Option<bool>,
    pub shelf_life_days: Option<i32>,
    pub requires_batch_control: Option<bool>,
    pub is_active: Option<bool>,
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
// Catalog-specific Operations
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MergeCatalogGroupsPayload {
    pub source_group_id: Uuid,
    pub target_group_id: Uuid,
}
