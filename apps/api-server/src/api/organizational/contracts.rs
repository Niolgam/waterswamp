use domain::models::organizational::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export domain types for API use
pub use domain::models::organizational::{
    ActivityArea, CreateOrganizationPayload, CreateOrganizationalUnitCategoryPayload,
    CreateOrganizationalUnitPayload, CreateOrganizationalUnitTypePayload,
    CreateSystemSettingPayload, InternalUnitType, OrganizationDto,
    OrganizationalUnitCategoryDto, OrganizationalUnitDto, OrganizationalUnitTreeNode,
    OrganizationalUnitTypeDto, OrganizationalUnitWithDetailsDto, SystemSettingDto,
    UpdateOrganizationPayload, UpdateOrganizationalUnitCategoryPayload,
    UpdateOrganizationalUnitPayload, UpdateOrganizationalUnitTypePayload,
    UpdateSystemSettingPayload,
};

// ============================
// List Responses
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemSettingsListResponse {
    pub settings: Vec<SystemSettingDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationsListResponse {
    pub organizations: Vec<OrganizationDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitCategoriesListResponse {
    pub categories: Vec<OrganizationalUnitCategoryDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitTypesListResponse {
    pub types: Vec<OrganizationalUnitTypeDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitsListResponse {
    pub units: Vec<OrganizationalUnitWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
