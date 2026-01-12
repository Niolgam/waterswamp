use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "activity_area_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityArea {
    Support,  // Área meio (administrativo)
    Core,     // Área fim (acadêmico/pesquisa)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "internal_unit_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InternalUnitType {
    Administration,
    Department,
    Laboratory,
    Sector,
    Council,
    Coordination,
    Center,
    Division,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "siorg_entity_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SiorgEntityType {
    Organization,
    Unit,
    Category,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "siorg_change_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SiorgChangeType {
    Creation,
    Update,
    Extinction,
    HierarchyChange,
    Merge,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "sync_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Conflict,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "mapping_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MappingStatus {
    Active,
    Deprecated,
    Merged,
    Unmapped,
}

// ============================================================================
// System Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemSettingDto {
    pub key: String,
    #[schema(value_type = Object)]
    pub value: serde_json::Value,
    pub value_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_sensitive: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSystemSettingPayload {
    pub key: String,
    #[schema(value_type = Object)]
    pub value: serde_json::Value,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub is_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSystemSettingPayload {
    #[schema(value_type = Object)]
    pub value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_sensitive: Option<bool>,
}

fn default_value_type() -> String {
    "string".to_string()
}

// ============================================================================
// Organization
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationDto {
    pub id: Uuid,
    pub acronym: String,
    pub name: String,
    pub cnpj: String,
    pub ug_code: i32,
    pub siorg_code: i32,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub is_main: bool,
    pub is_active: bool,
    pub siorg_synced_at: Option<DateTime<Utc>>,
    #[schema(value_type = Object)]
    pub siorg_raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationPayload {
    pub acronym: String,
    pub name: String,
    pub cnpj: String,
    pub ug_code: i32,
    pub siorg_code: i32,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationPayload {
    pub acronym: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Organizational Unit Category
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitCategoryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub siorg_code: Option<i32>,
    pub siorg_name: Option<String>,
    pub is_siorg_managed: bool,
    pub display_order: i32,
    pub is_active: bool,
    pub siorg_synced_at: Option<DateTime<Utc>>,
    pub siorg_sync_status: SyncStatus,
    #[schema(value_type = Object)]
    pub siorg_raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationalUnitCategoryPayload {
    pub name: String,
    pub description: Option<String>,
    pub siorg_code: Option<i32>,
    #[serde(default)]
    pub display_order: i32,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationalUnitCategoryPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Organizational Unit Type
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitTypeDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub siorg_code: Option<i32>,
    pub siorg_name: Option<String>,
    pub is_siorg_managed: bool,
    pub is_active: bool,
    pub siorg_synced_at: Option<DateTime<Utc>>,
    pub siorg_sync_status: SyncStatus,
    #[schema(value_type = Object)]
    pub siorg_raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationalUnitTypePayload {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub siorg_code: Option<i32>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationalUnitTypePayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Organizational Unit
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContactInfo {
    #[serde(default)]
    pub phones: Vec<String>,
    #[serde(default)]
    pub emails: Vec<String>,
    #[serde(default)]
    pub websites: Vec<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub category_id: Uuid,
    pub unit_type_id: Uuid,
    pub internal_type: InternalUnitType,
    pub name: String,
    pub formal_name: Option<String>,
    pub acronym: Option<String>,
    pub siorg_code: Option<i32>,
    pub siorg_parent_code: Option<i32>,
    pub siorg_url: Option<String>,
    pub siorg_last_version: Option<String>,
    pub is_siorg_managed: bool,
    pub activity_area: ActivityArea,
    pub contact_info: ContactInfo,
    pub level: i32,
    pub path_ids: Vec<Uuid>,
    pub path_names: Option<String>,
    pub is_active: bool,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub deactivation_reason: Option<String>,
    pub siorg_synced_at: Option<DateTime<Utc>>,
    pub siorg_sync_status: SyncStatus,
    #[schema(value_type = Object)]
    pub siorg_raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitWithDetailsDto {
    #[serde(flatten)]
    pub unit: OrganizationalUnitDto,
    pub organization_name: String,
    pub organization_acronym: String,
    pub parent_name: Option<String>,
    pub parent_acronym: Option<String>,
    pub category_name: String,
    pub unit_type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationalUnitTreeNode {
    #[serde(flatten)]
    pub unit: OrganizationalUnitDto,
    pub children: Vec<OrganizationalUnitTreeNode>,
    pub child_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationalUnitPayload {
    pub organization_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub category_id: Uuid,
    pub unit_type_id: Uuid,
    pub internal_type: InternalUnitType,
    pub name: String,
    pub formal_name: Option<String>,
    pub acronym: Option<String>,
    pub siorg_code: Option<i32>,
    pub activity_area: ActivityArea,
    #[serde(default)]
    pub contact_info: ContactInfo,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationalUnitPayload {
    pub parent_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub unit_type_id: Option<Uuid>,
    pub internal_type: Option<InternalUnitType>,
    pub name: Option<String>,
    pub formal_name: Option<String>,
    pub acronym: Option<String>,
    pub activity_area: Option<ActivityArea>,
    pub contact_info: Option<ContactInfo>,
    pub is_active: Option<bool>,
    pub deactivation_reason: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for ContactInfo {
    fn default() -> Self {
        Self {
            phones: Vec::new(),
            emails: Vec::new(),
            websites: Vec::new(),
            address: None,
        }
    }
}
