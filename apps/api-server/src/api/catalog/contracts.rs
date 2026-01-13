use domain::models::catalog::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export domain types for API use
pub use domain::models::catalog::{
    CatalogGroupTreeNode, CatalogGroupWithDetailsDto, CatalogItemWithDetailsDto,
    CreateCatalogGroupPayload, CreateCatalogItemPayload, CreateUnitConversionPayload,
    CreateUnitOfMeasurePayload, ItemType, UnitConversionWithDetailsDto, UnitOfMeasureDto,
    UpdateCatalogGroupPayload, UpdateCatalogItemPayload, UpdateUnitConversionPayload,
    UpdateUnitOfMeasurePayload,
};

// ============================
// List Responses
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnitsOfMeasureListResponse {
    pub units: Vec<UnitOfMeasureDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogGroupsListResponse {
    pub groups: Vec<CatalogGroupWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogItemsListResponse {
    pub items: Vec<CatalogItemWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnitConversionsListResponse {
    pub conversions: Vec<UnitConversionWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
