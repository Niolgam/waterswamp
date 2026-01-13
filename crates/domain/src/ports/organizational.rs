use crate::models::organizational::*;
use crate::errors::RepositoryError;
use async_trait::async_trait;
use uuid::Uuid;

// ============================================================================
// System Settings Repository Port
// ============================================================================

#[async_trait]
pub trait SystemSettingsRepositoryPort: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<SystemSettingDto>, RepositoryError>;

    async fn list(
        &self,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SystemSettingDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateSystemSettingPayload) -> Result<SystemSettingDto, RepositoryError>;

    async fn update(
        &self,
        key: &str,
        payload: UpdateSystemSettingPayload,
        updated_by: Option<Uuid>,
    ) -> Result<SystemSettingDto, RepositoryError>;

    async fn delete(&self, key: &str) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organization Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn find_by_cnpj(&self, cnpj: &str) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn find_main(&self) -> Result<Option<OrganizationDto>, RepositoryError>;

    async fn list(
        &self,
        is_active: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateOrganizationPayload) -> Result<OrganizationDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationPayload) -> Result<OrganizationDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organizational Unit Category Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationalUnitCategoryRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError>;

    async fn find_by_name(&self, name: &str) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError>;

    async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitCategoryDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateOrganizationalUnitCategoryPayload) -> Result<OrganizationalUnitCategoryDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationalUnitCategoryPayload) -> Result<OrganizationalUnitCategoryDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organizational Unit Type Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationalUnitTypeRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError>;

    async fn find_by_code(&self, code: &str) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError>;

    async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitTypeDto>, i64), RepositoryError>;

    async fn create(&self, payload: CreateOrganizationalUnitTypePayload) -> Result<OrganizationalUnitTypeDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationalUnitTypePayload) -> Result<OrganizationalUnitTypeDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// Organizational Unit Repository Port
// ============================================================================

#[async_trait]
pub trait OrganizationalUnitRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationalUnitDto>, RepositoryError>;

    async fn find_by_id_with_details(&self, id: Uuid) -> Result<Option<OrganizationalUnitWithDetailsDto>, RepositoryError>;

    async fn find_by_siorg_code(&self, siorg_code: i32) -> Result<Option<OrganizationalUnitDto>, RepositoryError>;

    async fn list(
        &self,
        organization_id: Option<Uuid>,
        parent_id: Option<Uuid>,
        category_id: Option<Uuid>,
        unit_type_id: Option<Uuid>,
        activity_area: Option<ActivityArea>,
        internal_type: Option<InternalUnitType>,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitWithDetailsDto>, i64), RepositoryError>;

    async fn get_tree(&self, organization_id: Option<Uuid>) -> Result<Vec<OrganizationalUnitTreeNode>, RepositoryError>;

    async fn get_children(&self, parent_id: Uuid) -> Result<Vec<OrganizationalUnitDto>, RepositoryError>;

    async fn has_children(&self, id: Uuid) -> Result<bool, RepositoryError>;

    async fn get_path_to_root(&self, id: Uuid) -> Result<Vec<OrganizationalUnitDto>, RepositoryError>;

    async fn create(&self, payload: CreateOrganizationalUnitPayload) -> Result<OrganizationalUnitDto, RepositoryError>;

    async fn update(&self, id: Uuid, payload: UpdateOrganizationalUnitPayload) -> Result<OrganizationalUnitDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    async fn deactivate(&self, id: Uuid, reason: Option<String>) -> Result<(), RepositoryError>;

    async fn activate(&self, id: Uuid) -> Result<(), RepositoryError>;
}

// ============================================================================
// SIORG Sync Queue Repository Port
// ============================================================================

#[async_trait]
pub trait SiorgSyncQueueRepositoryPort: Send + Sync {
    /// Poll next pending items from queue (with FOR UPDATE SKIP LOCKED)
    async fn poll_next_batch(&self, batch_size: i32) -> Result<Vec<SiorgSyncQueueItem>, RepositoryError>;

    /// Get queue item by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiorgSyncQueueItem>, RepositoryError>;

    /// List queue items with filters
    async fn list(
        &self,
        status: Option<SyncStatus>,
        entity_type: Option<SiorgEntityType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiorgSyncQueueItem>, RepositoryError>;

    /// Count items by status
    async fn count_by_status(&self, status: SyncStatus) -> Result<i64, RepositoryError>;

    /// Get conflicts (items with CONFLICT status)
    async fn get_conflicts(&self, limit: i64, offset: i64) -> Result<Vec<SiorgSyncQueueItem>, RepositoryError>;

    /// Create queue item
    async fn create(&self, payload: CreateSyncQueueItemPayload) -> Result<SiorgSyncQueueItem, RepositoryError>;

    /// Update queue item status
    async fn update_status(
        &self,
        id: Uuid,
        status: SyncStatus,
        error: Option<String>,
        error_details: Option<serde_json::Value>,
    ) -> Result<(), RepositoryError>;

    /// Mark as processing (sets status and last_attempt_at)
    async fn mark_processing(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Mark as completed
    async fn mark_completed(&self, id: Uuid, processed_by: Option<Uuid>) -> Result<(), RepositoryError>;

    /// Mark as failed (increments attempts)
    async fn mark_failed(&self, id: Uuid, error: String, error_details: Option<serde_json::Value>) -> Result<(), RepositoryError>;

    /// Mark as conflict
    async fn mark_conflict(&self, id: Uuid, detected_changes: serde_json::Value) -> Result<(), RepositoryError>;

    /// Resolve conflict
    async fn resolve(
        &self,
        id: Uuid,
        resolution_notes: String,
        processed_by: Option<Uuid>,
    ) -> Result<(), RepositoryError>;

    /// Delete queue item
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Clean expired items
    async fn clean_expired(&self) -> Result<i64, RepositoryError>;
}

// ============================================================================
// SIORG History Repository Port
// ============================================================================

#[async_trait]
pub trait SiorgHistoryRepositoryPort: Send + Sync {
    /// Get history item by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiorgHistoryItem>, RepositoryError>;

    /// List history with filters
    async fn list(
        &self,
        entity_type: Option<SiorgEntityType>,
        siorg_code: Option<i32>,
        change_type: Option<SiorgChangeType>,
        requires_review: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiorgHistoryItem>, RepositoryError>;

    /// Get history for a specific entity
    async fn get_entity_history(
        &self,
        entity_type: SiorgEntityType,
        siorg_code: i32,
        limit: i64,
    ) -> Result<Vec<SiorgHistoryItem>, RepositoryError>;

    /// Get pending reviews
    async fn get_pending_reviews(&self, limit: i64, offset: i64) -> Result<Vec<SiorgHistoryItem>, RepositoryError>;

    /// Create history entry
    async fn create(&self, payload: CreateHistoryItemPayload) -> Result<SiorgHistoryItem, RepositoryError>;

    /// Mark as reviewed
    async fn mark_reviewed(
        &self,
        id: Uuid,
        reviewed_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), RepositoryError>;

    /// Count history entries
    async fn count(&self, entity_type: Option<SiorgEntityType>) -> Result<i64, RepositoryError>;
}
