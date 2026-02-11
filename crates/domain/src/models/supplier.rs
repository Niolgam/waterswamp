use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Supplier DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct SupplierDto {
    pub id: Uuid,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub document_number: String,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
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
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub document_number: String,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
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
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub document_number: String,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSupplierPayload {
    pub legal_name: Option<String>,
    pub trade_name: Option<String>,
    pub document_number: Option<String>,
    pub representative_name: Option<String>,
    pub address: Option<String>,
    pub neighborhood: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}
