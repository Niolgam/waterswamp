use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::catalog::{
    // Units
    UnitOfMeasureDto, CreateUnitOfMeasurePayload, UpdateUnitOfMeasurePayload,
    UnitConversionWithDetailsDto, CreateUnitConversionPayload, UpdateUnitConversionPayload,
    // CATMAT
    CatmatGroupDto, CreateCatmatGroupPayload, UpdateCatmatGroupPayload, CatmatGroupTreeNode,
    CatmatClassWithDetailsDto, CreateCatmatClassPayload, UpdateCatmatClassPayload,
    CatmatPdmDto, CatmatPdmWithDetailsDto, CreateCatmatPdmPayload, UpdateCatmatPdmPayload,
    CatmatItemWithDetailsDto, CreateCatmatItemPayload, UpdateCatmatItemPayload,
    // CATSER
    CatserGroupDto, CreateCatserGroupPayload, UpdateCatserGroupPayload, CatserGroupTreeNode,
    CatserClassWithDetailsDto, CreateCatserClassPayload, UpdateCatserClassPayload,
    CatserItemWithDetailsDto, CreateCatserItemPayload, UpdateCatserItemPayload,
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
pub struct UnitConversionsListResponse {
    pub conversions: Vec<UnitConversionWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatGroupsListResponse {
    pub groups: Vec<CatmatGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatClassesListResponse {
    pub classes: Vec<CatmatClassWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatPdmsListResponse {
    pub pdms: Vec<CatmatPdmWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatItemsListResponse {
    pub items: Vec<CatmatItemWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserGroupsListResponse {
    pub groups: Vec<CatserGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserClassesListResponse {
    pub classes: Vec<CatserClassWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserItemsListResponse {
    pub items: Vec<CatserItemWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
