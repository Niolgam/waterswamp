import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../../environments/environment';
import {
  SiorgSyncQueueItem,
  QueueStatsResponse,
  ConflictDetail,
  ResolveConflictPayload,
  SiorgHistoryItem,
  ReviewHistoryPayload,
  ConflictListFilters,
  HistoryListFilters,
  SyncStatus,
  SiorgEntityType,
} from '../models/sync.models';

@Injectable({
  providedIn: 'root'
})
export class SyncService {
  private readonly baseUrl = `${environment.apiUrl}/api/admin/organizational/sync`;

  constructor(private http: HttpClient) {}

  // ========================================================================
  // Queue Management
  // ========================================================================

  listQueueItems(
    status?: SyncStatus,
    entityType?: SiorgEntityType,
    limit: number = 50,
    offset: number = 0
  ): Observable<SiorgSyncQueueItem[]> {
    let params = new HttpParams()
      .set('limit', limit.toString())
      .set('offset', offset.toString());

    if (status) {
      params = params.set('status', status);
    }
    if (entityType) {
      params = params.set('entity_type', entityType);
    }

    return this.http.get<SiorgSyncQueueItem[]>(`${this.baseUrl}/queue`, { params });
  }

  getQueueStats(): Observable<QueueStatsResponse> {
    return this.http.get<QueueStatsResponse>(`${this.baseUrl}/queue/stats`);
  }

  getQueueItem(id: string): Observable<SiorgSyncQueueItem> {
    return this.http.get<SiorgSyncQueueItem>(`${this.baseUrl}/queue/${id}`);
  }

  deleteQueueItem(id: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/queue/${id}`);
  }

  // ========================================================================
  // Conflict Resolution
  // ========================================================================

  listConflicts(filters: ConflictListFilters): Observable<SiorgSyncQueueItem[]> {
    const params = new HttpParams()
      .set('limit', filters.limit.toString())
      .set('offset', filters.offset.toString());

    return this.http.get<SiorgSyncQueueItem[]>(`${this.baseUrl}/conflicts`, { params });
  }

  getConflictDetail(id: string): Observable<ConflictDetail> {
    return this.http.get<ConflictDetail>(`${this.baseUrl}/conflicts/${id}`);
  }

  resolveConflict(id: string, payload: ResolveConflictPayload): Observable<void> {
    return this.http.post<void>(`${this.baseUrl}/conflicts/${id}/resolve`, payload);
  }

  // ========================================================================
  // History
  // ========================================================================

  listHistory(filters: HistoryListFilters): Observable<SiorgHistoryItem[]> {
    let params = new HttpParams()
      .set('limit', filters.limit.toString())
      .set('offset', filters.offset.toString());

    if (filters.entity_type) {
      params = params.set('entity_type', filters.entity_type);
    }
    if (filters.siorg_code !== undefined) {
      params = params.set('siorg_code', filters.siorg_code.toString());
    }
    if (filters.change_type) {
      params = params.set('change_type', filters.change_type);
    }
    if (filters.requires_review !== undefined) {
      params = params.set('requires_review', filters.requires_review.toString());
    }

    return this.http.get<SiorgHistoryItem[]>(`${this.baseUrl}/history`, { params });
  }

  getHistoryItem(id: string): Observable<SiorgHistoryItem> {
    return this.http.get<SiorgHistoryItem>(`${this.baseUrl}/history/${id}`);
  }

  getEntityHistory(
    entityType: SiorgEntityType,
    siorgCode: number,
    limit: number = 50
  ): Observable<SiorgHistoryItem[]> {
    const params = new HttpParams().set('limit', limit.toString());

    return this.http.get<SiorgHistoryItem[]>(
      `${this.baseUrl}/history/entity/${entityType}/${siorgCode}`,
      { params }
    );
  }

  reviewHistoryItem(id: string, payload: ReviewHistoryPayload): Observable<void> {
    return this.http.post<void>(`${this.baseUrl}/history/${id}/review`, payload);
  }
}
