use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "supplier_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SupplierType {
    Individual,
    LegalEntity,
    GovernmentUnit,
}

// ============================
// Supplier DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SupplierDto {
    pub id: Uuid,
    pub supplier_type: SupplierType,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub document_number: String,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
    pub is_international_neighborhood: bool,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Supplier with city/state names joined
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SupplierWithDetailsDto {
    pub id: Uuid,
    pub supplier_type: SupplierType,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub document_number: String,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
    pub is_international_neighborhood: bool,
    pub city_id: Option<Uuid>,
    pub city_name: Option<String>,
    pub state_abbreviation: Option<String>,
    pub zip_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSupplierPayload {
    pub supplier_type: SupplierType,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub document_number: String,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
    pub is_international_neighborhood: Option<bool>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSupplierPayload {
    pub supplier_type: Option<SupplierType>,
    pub legal_name: Option<String>,
    pub trade_name: Option<String>,
    pub document_number: Option<String>,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
    pub is_international_neighborhood: Option<bool>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}
