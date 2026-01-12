// ============================================================================
// Sync Queue Models
// ============================================================================

export type SyncStatus = 'PENDING' | 'PROCESSING' | 'COMPLETED' | 'FAILED' | 'CONFLICT' | 'SKIPPED';
export type SiorgEntityType = 'ORGANIZATION' | 'UNIT' | 'CATEGORY' | 'TYPE';
export type SiorgChangeType = 'CREATION' | 'UPDATE' | 'EXTINCTION' | 'HIERARCHY_CHANGE' | 'MERGE' | 'SPLIT';

export interface SiorgSyncQueueItem {
  id: string;
  entity_type: SiorgEntityType;
  siorg_code: number;
  local_id?: string;
  operation: SiorgChangeType;
  priority: number;
  payload: any;
  detected_changes?: any;
  status: SyncStatus;
  attempts: number;
  max_attempts: number;
  last_attempt_at?: string;
  last_error?: string;
  error_details?: any;
  processed_at?: string;
  processed_by?: string;
  resolution_notes?: string;
  scheduled_for: string;
  expires_at?: string;
  created_at: string;
}

export interface QueueStatsResponse {
  pending: number;
  processing: number;
  completed: number;
  failed: number;
  conflicts: number;
  skipped: number;
}

// ============================================================================
// Conflict Resolution Models
// ============================================================================

export interface ConflictDiff {
  field: string;
  local_value?: any;
  siorg_value?: any;
  field_type: string;
  has_conflict: boolean;
  metadata?: any;
}

export interface ConflictDetail {
  queue_item: SiorgSyncQueueItem;
  entity_type: SiorgEntityType;
  fields: ConflictDiff[];
  local_entity_name?: string;
  siorg_entity_name?: string;
}

export type ResolutionAction = 'ACCEPT_SIORG' | 'KEEP_LOCAL' | 'MERGE' | 'SKIP';
export type FieldResolution = 'LOCAL' | 'SIORG';

export interface ResolveConflictPayload {
  action: ResolutionAction;
  field_resolutions?: Record<string, FieldResolution>;
  notes?: string;
}

// ============================================================================
// History Models
// ============================================================================

export interface SiorgHistoryItem {
  id: string;
  entity_type: SiorgEntityType;
  siorg_code: number;
  local_id?: string;
  change_type: SiorgChangeType;
  previous_data?: any;
  new_data?: any;
  affected_fields: string[];
  siorg_version?: string;
  source: string;
  sync_queue_id?: string;
  requires_review: boolean;
  reviewed_at?: string;
  reviewed_by?: string;
  review_notes?: string;
  created_at: string;
  created_by?: string;
}

export interface ReviewHistoryPayload {
  notes?: string;
}

// ============================================================================
// UI Helper Models
// ============================================================================

export interface ConflictListFilters {
  limit: number;
  offset: number;
}

export interface HistoryListFilters {
  entity_type?: SiorgEntityType;
  siorg_code?: number;
  change_type?: SiorgChangeType;
  requires_review?: boolean;
  limit: number;
  offset: number;
}
