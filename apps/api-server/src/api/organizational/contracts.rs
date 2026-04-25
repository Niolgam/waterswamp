use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export domain types for API use
pub use domain::models::organizational::{
    ActivityArea, CreateOrganizationPayload, CreateOrganizationalUnitCategoryPayload,
    CreateOrganizationalUnitPayload, CreateOrganizationalUnitTypePayload,
    CreateSiorgEsferaPayload, CreateSiorgNaturezaJuridicaPayload, CreateSiorgPoderPayload,
    CreateSystemSettingPayload, OrganizationDto, OrganizationalUnitCategoryDto,
    OrganizationalUnitDto, OrganizationalUnitTreeNode, OrganizationalUnitTypeDto,
    OrganizationalUnitWithDetailsDto, SiorgEsferaDto, SiorgNaturezaJuridicaDto, SiorgPoderDto,
    SystemSettingDto, UpdateOrganizationPayload, UpdateOrganizationalUnitCategoryPayload,
    UpdateOrganizationalUnitPayload, UpdateOrganizationalUnitTypePayload,
    UpdateSiorgEsferaPayload, UpdateSiorgNaturezaJuridicaPayload, UpdateSiorgPoderPayload,
    UpdateSystemSettingPayload,
};

// ============================
// List Responses
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemSettingsListResponse {
    pub data: Vec<SystemSettingDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationsListResponse {
    pub data: Vec<OrganizationDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitCategoriesListResponse {
    pub data: Vec<OrganizationalUnitCategoryDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitTypesListResponse {
    pub data: Vec<OrganizationalUnitTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitsListResponse {
    pub data: Vec<OrganizationalUnitWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SiorgNaturezaJuridicaListResponse {
    pub data: Vec<SiorgNaturezaJuridicaDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SiorgPoderListResponse {
    pub data: Vec<SiorgPoderDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SiorgEsferaListResponse {
    pub data: Vec<SiorgEsferaDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
