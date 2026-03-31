use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::catalog::{
    CatmatClassWithDetailsDto,
    // CATMAT
    CatmatGroupDto,
    CatmatGroupTreeNode,
    CatmatItemWithDetailsDto,
    CatmatPdmDto,
    CatmatPdmWithDetailsDto,
    CatserClassWithDetailsDto,
    CatserDivisionDto,
    CatserDivisionWithDetailsDto,
    CatserGroupDto,
    CatserGroupTreeNode,
    CatserItemWithDetailsDto,
    // CATSER
    CatserSectionDto,
    CatserSectionTreeNode,
    CatserSectionWithDetailsDto,
    CreateCatmatClassPayload,
    CreateCatmatGroupPayload,
    CreateCatmatItemPayload,
    CreateCatmatPdmPayload,
    CreateCatserClassPayload,
    CreateCatserDivisionPayload,
    CreateCatserGroupPayload,
    CreateCatserItemPayload,
    CreateCatserSectionPayload,
    CreateUnitConversionPayload,
    CreateUnitOfMeasurePayload,
    UnitConversionWithDetailsDto,
    // Units
    UnitOfMeasureDto,
    UpdateCatmatClassPayload,
    UpdateCatmatGroupPayload,
    UpdateCatmatItemPayload,
    UpdateCatmatPdmPayload,
    UpdateCatserClassPayload,
    UpdateCatserDivisionPayload,
    UpdateCatserGroupPayload,
    UpdateCatserItemPayload,
    UpdateCatserSectionPayload,
    UpdateUnitConversionPayload,
    UpdateUnitOfMeasurePayload,
};

// ============================
// List Responses
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnitsOfMeasureListResponse {
    pub data: Vec<UnitOfMeasureDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnitConversionsListResponse {
    pub data: Vec<UnitConversionWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatGroupsListResponse {
    pub data: Vec<CatmatGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatClassesListResponse {
    pub data: Vec<CatmatClassWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatPdmsListResponse {
    pub data: Vec<CatmatPdmWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatItemsListResponse {
    pub data: Vec<CatmatItemWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserSectionsListResponse {
    pub data: Vec<CatserSectionWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserDivisionsListResponse {
    pub data: Vec<CatserDivisionWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserGroupsListResponse {
    pub data: Vec<CatserGroupDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserClassesListResponse {
    pub data: Vec<CatserClassWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserItemsListResponse {
    pub data: Vec<CatserItemWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
