import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../../environments/environment';
import {
  Organization,
  OrganizationsListResponse,
  CreateOrganizationPayload,
  OrganizationalUnit,
  OrganizationalUnitWithDetails,
  OrganizationalUnitsListResponse,
  OrganizationalUnitTreeNode,
  CreateOrganizationalUnitPayload,
  OrganizationalUnitCategory,
  OrganizationalUnitCategoriesListResponse,
  OrganizationalUnitType,
  OrganizationalUnitTypesListResponse,
  SystemSetting,
  SystemSettingsListResponse,
  CreateSystemSettingPayload,
  SyncOrganizationRequest,
  SyncUnitRequest,
  SyncOrgUnitsRequest,
  SyncSummary,
  SiorgHealthResponse
} from '../models/organizational.models';

@Injectable({
  providedIn: 'root'
})
export class OrganizationalService {
  private baseUrl = `${environment.apiUrl}/api/admin/organizational`;

  constructor(private http: HttpClient) {}

  // ========================================================================
  // System Settings
  // ========================================================================

  listSystemSettings(params?: { category?: string; limit?: number; offset?: number }): Observable<SystemSettingsListResponse> {
    let httpParams = new HttpParams();
    if (params?.category) httpParams = httpParams.set('category', params.category);
    if (params?.limit) httpParams = httpParams.set('limit', params.limit.toString());
    if (params?.offset) httpParams = httpParams.set('offset', params.offset.toString());

    return this.http.get<SystemSettingsListResponse>(`${this.baseUrl}/settings`, { params: httpParams });
  }

  getSystemSetting(key: string): Observable<SystemSetting> {
    return this.http.get<SystemSetting>(`${this.baseUrl}/settings/${key}`);
  }

  createSystemSetting(payload: CreateSystemSettingPayload): Observable<SystemSetting> {
    return this.http.post<SystemSetting>(`${this.baseUrl}/settings`, payload);
  }

  updateSystemSetting(key: string, payload: Partial<CreateSystemSettingPayload>): Observable<SystemSetting> {
    return this.http.put<SystemSetting>(`${this.baseUrl}/settings/${key}`, payload);
  }

  deleteSystemSetting(key: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/settings/${key}`);
  }

  // ========================================================================
  // Organizations
  // ========================================================================

  listOrganizations(params?: { is_active?: boolean; limit?: number; offset?: number }): Observable<OrganizationsListResponse> {
    let httpParams = new HttpParams();
    if (params?.is_active !== undefined) httpParams = httpParams.set('is_active', params.is_active.toString());
    if (params?.limit) httpParams = httpParams.set('limit', params.limit.toString());
    if (params?.offset) httpParams = httpParams.set('offset', params.offset.toString());

    return this.http.get<OrganizationsListResponse>(`${this.baseUrl}/organizations`, { params: httpParams });
  }

  getOrganization(id: string): Observable<Organization> {
    return this.http.get<Organization>(`${this.baseUrl}/organizations/${id}`);
  }

  createOrganization(payload: CreateOrganizationPayload): Observable<Organization> {
    return this.http.post<Organization>(`${this.baseUrl}/organizations`, payload);
  }

  updateOrganization(id: string, payload: Partial<CreateOrganizationPayload>): Observable<Organization> {
    return this.http.put<Organization>(`${this.baseUrl}/organizations/${id}`, payload);
  }

  deleteOrganization(id: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/organizations/${id}`);
  }

  // ========================================================================
  // Unit Categories
  // ========================================================================

  listUnitCategories(params?: { is_active?: boolean; is_siorg_managed?: boolean; limit?: number; offset?: number }): Observable<OrganizationalUnitCategoriesListResponse> {
    let httpParams = new HttpParams();
    if (params?.is_active !== undefined) httpParams = httpParams.set('is_active', params.is_active.toString());
    if (params?.is_siorg_managed !== undefined) httpParams = httpParams.set('is_siorg_managed', params.is_siorg_managed.toString());
    if (params?.limit) httpParams = httpParams.set('limit', params.limit.toString());
    if (params?.offset) httpParams = httpParams.set('offset', params.offset.toString());

    return this.http.get<OrganizationalUnitCategoriesListResponse>(`${this.baseUrl}/unit-categories`, { params: httpParams });
  }

  getUnitCategory(id: string): Observable<OrganizationalUnitCategory> {
    return this.http.get<OrganizationalUnitCategory>(`${this.baseUrl}/unit-categories/${id}`);
  }

  // ========================================================================
  // Unit Types
  // ========================================================================

  listUnitTypes(params?: { is_active?: boolean; is_siorg_managed?: boolean; limit?: number; offset?: number }): Observable<OrganizationalUnitTypesListResponse> {
    let httpParams = new HttpParams();
    if (params?.is_active !== undefined) httpParams = httpParams.set('is_active', params.is_active.toString());
    if (params?.is_siorg_managed !== undefined) httpParams = httpParams.set('is_siorg_managed', params.is_siorg_managed.toString());
    if (params?.limit) httpParams = httpParams.set('limit', params.limit.toString());
    if (params?.offset) httpParams = httpParams.set('offset', params.offset.toString());

    return this.http.get<OrganizationalUnitTypesListResponse>(`${this.baseUrl}/unit-types`, { params: httpParams });
  }

  getUnitType(id: string): Observable<OrganizationalUnitType> {
    return this.http.get<OrganizationalUnitType>(`${this.baseUrl}/unit-types/${id}`);
  }

  // ========================================================================
  // Organizational Units
  // ========================================================================

  listOrganizationalUnits(params?: {
    organization_id?: string;
    parent_id?: string;
    category_id?: string;
    unit_type_id?: string;
    activity_area?: string;
    internal_type?: string;
    is_active?: boolean;
    is_siorg_managed?: boolean;
    search?: string;
    limit?: number;
    offset?: number;
  }): Observable<OrganizationalUnitsListResponse> {
    let httpParams = new HttpParams();
    if (params?.organization_id) httpParams = httpParams.set('organization_id', params.organization_id);
    if (params?.parent_id) httpParams = httpParams.set('parent_id', params.parent_id);
    if (params?.category_id) httpParams = httpParams.set('category_id', params.category_id);
    if (params?.unit_type_id) httpParams = httpParams.set('unit_type_id', params.unit_type_id);
    if (params?.activity_area) httpParams = httpParams.set('activity_area', params.activity_area);
    if (params?.internal_type) httpParams = httpParams.set('internal_type', params.internal_type);
    if (params?.is_active !== undefined) httpParams = httpParams.set('is_active', params.is_active.toString());
    if (params?.is_siorg_managed !== undefined) httpParams = httpParams.set('is_siorg_managed', params.is_siorg_managed.toString());
    if (params?.search) httpParams = httpParams.set('search', params.search);
    if (params?.limit) httpParams = httpParams.set('limit', params.limit.toString());
    if (params?.offset) httpParams = httpParams.set('offset', params.offset.toString());

    return this.http.get<OrganizationalUnitsListResponse>(`${this.baseUrl}/units`, { params: httpParams });
  }

  getOrganizationalUnit(id: string): Observable<OrganizationalUnitWithDetails> {
    return this.http.get<OrganizationalUnitWithDetails>(`${this.baseUrl}/units/${id}`);
  }

  getOrganizationalUnitsTree(params?: { organization_id?: string }): Observable<OrganizationalUnitTreeNode[]> {
    let httpParams = new HttpParams();
    if (params?.organization_id) httpParams = httpParams.set('organization_id', params.organization_id);

    return this.http.get<OrganizationalUnitTreeNode[]>(`${this.baseUrl}/units/tree`, { params: httpParams });
  }

  getOrganizationalUnitChildren(id: string): Observable<OrganizationalUnit[]> {
    return this.http.get<OrganizationalUnit[]>(`${this.baseUrl}/units/${id}/children`);
  }

  getOrganizationalUnitPath(id: string): Observable<OrganizationalUnit[]> {
    return this.http.get<OrganizationalUnit[]>(`${this.baseUrl}/units/${id}/path`);
  }

  createOrganizationalUnit(payload: CreateOrganizationalUnitPayload): Observable<OrganizationalUnit> {
    return this.http.post<OrganizationalUnit>(`${this.baseUrl}/units`, payload);
  }

  updateOrganizationalUnit(id: string, payload: Partial<CreateOrganizationalUnitPayload>): Observable<OrganizationalUnit> {
    return this.http.put<OrganizationalUnit>(`${this.baseUrl}/units/${id}`, payload);
  }

  deleteOrganizationalUnit(id: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/units/${id}`);
  }

  deactivateOrganizationalUnit(id: string, reason?: string): Observable<void> {
    return this.http.post<void>(`${this.baseUrl}/units/${id}/deactivate`, reason);
  }

  activateOrganizationalUnit(id: string): Observable<void> {
    return this.http.post<void>(`${this.baseUrl}/units/${id}/activate`, {});
  }

  // ========================================================================
  // SIORG Sync
  // ========================================================================

  syncOrganization(request: SyncOrganizationRequest): Observable<Organization> {
    return this.http.post<Organization>(`${this.baseUrl}/sync/organization`, request);
  }

  syncUnit(request: SyncUnitRequest): Observable<OrganizationalUnit> {
    return this.http.post<OrganizationalUnit>(`${this.baseUrl}/sync/unit`, request);
  }

  syncOrganizationUnits(request: SyncOrgUnitsRequest): Observable<SyncSummary> {
    return this.http.post<SyncSummary>(`${this.baseUrl}/sync/org-units`, request);
  }

  checkSiorgHealth(): Observable<SiorgHealthResponse> {
    return this.http.get<SiorgHealthResponse>(`${this.baseUrl}/sync/health`);
  }
}
