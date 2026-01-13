// Enums
export enum ActivityArea {
  Support = 'SUPPORT',
  Core = 'CORE'
}

export enum InternalUnitType {
  Administration = 'ADMINISTRATION',
  Department = 'DEPARTMENT',
  Laboratory = 'LABORATORY',
  Sector = 'SECTOR',
  Council = 'COUNCIL',
  Coordination = 'COORDINATION',
  Center = 'CENTER',
  Division = 'DIVISION'
}

export enum SyncStatus {
  Pending = 'PENDING',
  Processing = 'PROCESSING',
  Completed = 'COMPLETED',
  Failed = 'FAILED',
  Conflict = 'CONFLICT',
  Skipped = 'SKIPPED'
}

export enum SiorgChangeType {
  Creation = 'CREATION',
  Update = 'UPDATE',
  Extinction = 'EXTINCTION',
  HierarchyChange = 'HIERARCHY_CHANGE',
  Merge = 'MERGE',
  Split = 'SPLIT'
}

// Contact Info
export interface ContactInfo {
  phones: string[];
  emails: string[];
  websites: string[];
  address?: string;
}

// System Settings
export interface SystemSetting {
  key: string;
  value: any;
  value_type: string;
  description?: string;
  category?: string;
  is_sensitive: boolean;
  updated_at: string;
  updated_by?: string;
}

export interface CreateSystemSettingPayload {
  key: string;
  value: any;
  value_type: string;
  description?: string;
  category?: string;
  is_sensitive: boolean;
}

// Organization
export interface Organization {
  id: string;
  acronym: string;
  name: string;
  cnpj: string;
  ug_code: number;
  siorg_code?: number;
  is_main_organization: boolean;
  is_active: boolean;
  last_siorg_sync?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateOrganizationPayload {
  acronym: string;
  name: string;
  cnpj: string;
  ug_code: number;
  siorg_code?: number;
  is_main_organization: boolean;
  is_active: boolean;
}

// Organizational Unit Category
export interface OrganizationalUnitCategory {
  id: string;
  name: string;
  description?: string;
  is_active: boolean;
  is_siorg_managed: boolean;
  siorg_code?: number;
  created_at: string;
  updated_at: string;
}

// Organizational Unit Type
export interface OrganizationalUnitType {
  id: string;
  code: string;
  name: string;
  description?: string;
  is_active: boolean;
  is_siorg_managed: boolean;
  siorg_code?: number;
  created_at: string;
  updated_at: string;
}

// Organizational Unit
export interface OrganizationalUnit {
  id: string;
  organization_id: string;
  parent_id?: string;
  category_id: string;
  unit_type_id: string;
  internal_type: InternalUnitType;
  name: string;
  siorg_code?: number;
  activity_area: ActivityArea;
  contact_info: ContactInfo;
  level: number;
  path_ids: string[];
  path_names?: string;
  is_active: boolean;
  last_siorg_sync?: string;
  created_at: string;
  updated_at: string;
}

export interface OrganizationalUnitWithDetails extends OrganizationalUnit {
  organization_name: string;
  parent_name?: string;
  category_name: string;
  unit_type_name: string;
}

export interface OrganizationalUnitTreeNode {
  unit: OrganizationalUnit;
  children: OrganizationalUnitTreeNode[];
  child_count: number;
}

export interface CreateOrganizationalUnitPayload {
  organization_id: string;
  parent_id?: string;
  category_id: string;
  unit_type_id: string;
  internal_type: InternalUnitType;
  name: string;
  siorg_code?: number;
  activity_area: ActivityArea;
  contact_info: ContactInfo;
  is_active: boolean;
}

// SIORG Sync
export interface SyncOrganizationRequest {
  siorg_code: number;
}

export interface SyncUnitRequest {
  siorg_code: number;
}

export interface SyncOrgUnitsRequest {
  org_siorg_code: number;
}

export interface SyncSummary {
  total_processed: number;
  created: number;
  updated: number;
  failed: number;
  errors: string[];
}

export interface SiorgHealthResponse {
  status: string;
  siorg_api: string;
}

// List Responses
export interface SystemSettingsListResponse {
  settings: SystemSetting[];
  total: number;
  limit: number;
  offset: number;
}

export interface OrganizationsListResponse {
  organizations: Organization[];
  total: number;
  limit: number;
  offset: number;
}

export interface OrganizationalUnitsListResponse {
  units: OrganizationalUnitWithDetails[];
  total: number;
  limit: number;
  offset: number;
}

export interface OrganizationalUnitCategoriesListResponse {
  categories: OrganizationalUnitCategory[];
  total: number;
  limit: number;
  offset: number;
}

export interface OrganizationalUnitTypesListResponse {
  types: OrganizationalUnitType[];
  total: number;
  limit: number;
  offset: number;
}
