use crate::external::siorg_client::SiorgUnidadeCompleta;
use domain::errors::RepositoryError;
use domain::models::organizational::*;
use domain::ports::{
    OrganizationRepositoryPort, OrganizationalUnitRepositoryPort,
    SiorgHistoryRepositoryPort, SiorgSyncQueueRepositoryPort,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Conflict Resolution Service
// ============================================================================

pub struct ConflictResolutionService {
    sync_queue_repo: Arc<dyn SiorgSyncQueueRepositoryPort>,
    history_repo: Arc<dyn SiorgHistoryRepositoryPort>,
    organization_repo: Arc<dyn OrganizationRepositoryPort>,
    unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
}

impl ConflictResolutionService {
    pub fn new(
        sync_queue_repo: Arc<dyn SiorgSyncQueueRepositoryPort>,
        history_repo: Arc<dyn SiorgHistoryRepositoryPort>,
        organization_repo: Arc<dyn OrganizationRepositoryPort>,
        unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
    ) -> Self {
        Self {
            sync_queue_repo,
            history_repo,
            organization_repo,
            unit_repo,
        }
    }

    // ========================================================================
    // Get Conflict Details with Field-Level Diff
    // ========================================================================

    /// Get detailed conflict information with field-by-field comparison
    pub async fn get_conflict_detail(
        &self,
        queue_item_id: Uuid,
    ) -> Result<ConflictDetail, ServiceError> {
        // Get queue item
        let queue_item = self
            .sync_queue_repo
            .find_by_id(queue_item_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Queue item {} not found", queue_item_id)))?;

        // Ensure it's actually a conflict
        if queue_item.status != SyncStatus::Conflict {
            return Err(ServiceError::InvalidOperation(format!(
                "Queue item {} is not a conflict (status: {:?})",
                queue_item_id, queue_item.status
            )));
        }

        // Parse SIORG data from payload
        match queue_item.entity_type {
            SiorgEntityType::Organization => {
                self.build_organization_conflict_detail(queue_item).await
            }
            SiorgEntityType::Unit => {
                self.build_unit_conflict_detail(queue_item).await
            }
            _ => Err(ServiceError::InvalidOperation(format!(
                "Conflict resolution not supported for {:?}",
                queue_item.entity_type
            ))),
        }
    }

    /// Build conflict detail for organization
    async fn build_organization_conflict_detail(
        &self,
        queue_item: SiorgSyncQueueItem,
    ) -> Result<ConflictDetail, ServiceError> {
        // Parse SIORG data
        let siorg_org: SiorgUnidadeCompleta = serde_json::from_value(queue_item.payload.clone())
            .map_err(|e| ServiceError::InvalidData(format!("Failed to parse SIORG data: {}", e)))?;

        // Get local organization
        let local_org = if let Some(local_id) = queue_item.local_id {
            self.organization_repo.find_by_id(local_id).await?
        } else {
            // Try to find by SIORG code
            self.organization_repo
                .find_by_siorg_code(queue_item.siorg_code)
                .await?
        };

        let local_org = local_org.ok_or_else(|| {
            ServiceError::NotFound("Local organization not found".to_string())
        })?;

        // Compare field by field
        let fields = self.compare_organization_fields(&local_org, &siorg_org);

        Ok(ConflictDetail {
            queue_item,
            entity_type: SiorgEntityType::Organization,
            fields,
            local_entity_name: Some(local_org.name.clone()),
            siorg_entity_name: Some(siorg_org.base.nome.clone()),
        })
    }

    /// Build conflict detail for organizational unit
    async fn build_unit_conflict_detail(
        &self,
        queue_item: SiorgSyncQueueItem,
    ) -> Result<ConflictDetail, ServiceError> {
        // Parse SIORG data
        let siorg_unit: SiorgUnidadeCompleta = serde_json::from_value(queue_item.payload.clone())
            .map_err(|e| ServiceError::InvalidData(format!("Failed to parse SIORG data: {}", e)))?;

        // Get local unit
        let local_unit = if let Some(local_id) = queue_item.local_id {
            self.unit_repo.find_by_id(local_id).await?
        } else {
            // Try to find by SIORG code
            self.unit_repo
                .find_by_siorg_code(queue_item.siorg_code)
                .await?
        };

        let local_unit = local_unit.ok_or_else(|| {
            ServiceError::NotFound("Local organizational unit not found".to_string())
        })?;

        // Compare field by field
        let fields = self.compare_unit_fields(&local_unit, &siorg_unit);

        Ok(ConflictDetail {
            queue_item,
            entity_type: SiorgEntityType::Unit,
            fields,
            local_entity_name: Some(local_unit.name.clone()),
            siorg_entity_name: Some(siorg_unit.base.nome.clone()),
        })
    }

    // ========================================================================
    // Field Comparison Logic
    // ========================================================================

    /// Compare organization fields
    fn compare_organization_fields(
        &self,
        local: &OrganizationDto,
        siorg: &SiorgUnidadeCompleta,
    ) -> Vec<ConflictDiff> {
        let mut fields = Vec::new();

        // Name
        let name_conflict = local.name != siorg.base.nome;
        fields.push(ConflictDiff {
            field: "name".to_string(),
            local_value: Some(json!(local.name)),
            siorg_value: Some(json!(siorg.base.nome)),
            field_type: "string".to_string(),
            has_conflict: name_conflict,
            metadata: None,
        });

        // Acronym
        let siorg_sigla = siorg.base.sigla.as_deref().unwrap_or("");
        let acronym_conflict = local.acronym != siorg_sigla;
        fields.push(ConflictDiff {
            field: "acronym".to_string(),
            local_value: Some(json!(local.acronym)),
            siorg_value: Some(json!(siorg_sigla)),
            field_type: "string".to_string(),
            has_conflict: acronym_conflict,
            metadata: None,
        });

        // CNPJ e UG Code não são fornecidos pela API SIORG pública — omitidos da comparação.

        fields
    }

    /// Compare unit fields
    fn compare_unit_fields(
        &self,
        local: &OrganizationalUnitDto,
        siorg: &SiorgUnidadeCompleta,
    ) -> Vec<ConflictDiff> {
        let mut fields = Vec::new();

        // Name
        let name_conflict = local.name != siorg.base.nome;
        fields.push(ConflictDiff {
            field: "name".to_string(),
            local_value: Some(json!(local.name)),
            siorg_value: Some(json!(siorg.base.nome)),
            field_type: "string".to_string(),
            has_conflict: name_conflict,
            metadata: None,
        });

        // Parent (hierarchy) - this is complex and requires metadata
        if let Some(siorg_parent) = siorg.base.parent_siorg_code() {
            fields.push(ConflictDiff {
                field: "parent".to_string(),
                local_value: local.parent_id.map(|id| json!(id)),
                siorg_value: Some(json!(siorg_parent)),
                field_type: "reference".to_string(),
                has_conflict: true,
                metadata: Some(json!({
                    "type": "organizational_unit",
                    "siorg_parent_code": siorg_parent,
                    "note": "Hierarchy changes require careful review"
                })),
            });
        }

        fields
    }

    // ========================================================================
    // Conflict Resolution
    // ========================================================================

    /// Resolve conflict with user decision
    pub async fn resolve_conflict(
        &self,
        queue_item_id: Uuid,
        payload: ResolveConflictPayload,
        resolved_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        // Get queue item
        let queue_item = self
            .sync_queue_repo
            .find_by_id(queue_item_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Queue item {} not found", queue_item_id)))?;

        info!(
            "Resolving conflict for queue item {} with action {:?}",
            queue_item_id, payload.action
        );

        match payload.action {
            ResolutionAction::AcceptSiorg => {
                self.resolve_accept_siorg(queue_item, resolved_by).await
            }
            ResolutionAction::KeepLocal => {
                self.resolve_keep_local(queue_item, payload.notes, resolved_by)
                    .await
            }
            ResolutionAction::Merge => {
                let field_resolutions = payload.field_resolutions.ok_or_else(|| {
                    ServiceError::InvalidData(
                        "field_resolutions required for MERGE action".to_string(),
                    )
                })?;
                self.resolve_merge(queue_item, field_resolutions, payload.notes, resolved_by)
                    .await
            }
            ResolutionAction::Skip => {
                self.resolve_skip(queue_item, payload.notes, resolved_by)
                    .await
            }
        }
    }

    /// Accept SIORG version (overwrite local)
    async fn resolve_accept_siorg(
        &self,
        queue_item: SiorgSyncQueueItem,
        resolved_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        info!("Accepting SIORG version for item {}", queue_item.id);

        // Mark as pending to be reprocessed by worker
        // Worker will apply SIORG changes
        self.sync_queue_repo
            .update_status(
                queue_item.id,
                SyncStatus::Pending,
                None,
                Some(json!({ "resolution": "ACCEPT_SIORG", "resolved_by": resolved_by })),
            )
            .await?;

        // Create history entry
        self.create_resolution_history(
            &queue_item,
            "ACCEPT_SIORG",
            "Accepted SIORG version, local data will be overwritten",
            resolved_by,
        )
        .await?;

        Ok(())
    }

    /// Keep local version (ignore SIORG)
    async fn resolve_keep_local(
        &self,
        queue_item: SiorgSyncQueueItem,
        notes: Option<String>,
        resolved_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        info!("Keeping local version for item {}", queue_item.id);

        let resolution_notes = notes
            .unwrap_or_else(|| "Kept local version, SIORG changes ignored".to_string());

        // Mark as completed (skip SIORG update)
        self.sync_queue_repo
            .resolve(queue_item.id, resolution_notes.clone(), resolved_by)
            .await?;

        // Create history entry
        self.create_resolution_history(
            &queue_item,
            "KEEP_LOCAL",
            &resolution_notes,
            resolved_by,
        )
        .await?;

        Ok(())
    }

    /// Merge fields (field-by-field resolution)
    async fn resolve_merge(
        &self,
        queue_item: SiorgSyncQueueItem,
        field_resolutions: HashMap<String, FieldResolution>,
        notes: Option<String>,
        resolved_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        info!(
            "Merging fields for item {} with {} field resolutions",
            queue_item.id,
            field_resolutions.len()
        );

        // Build merged payload based on field resolutions
        let merged_payload = self
            .build_merged_payload(&queue_item, &field_resolutions)
            .await?;

        // Update queue item with merged data
        self.sync_queue_repo
            .update_status(
                queue_item.id,
                SyncStatus::Pending, // Will be reprocessed with merged data
                None,
                Some(json!({
                    "resolution": "MERGE",
                    "merged_payload": merged_payload,
                    "field_resolutions": field_resolutions,
                    "resolved_by": resolved_by
                })),
            )
            .await?;

        // Create history entry
        let resolution_notes = notes.unwrap_or_else(|| {
            format!(
                "Merged {} fields: {:?}",
                field_resolutions.len(),
                field_resolutions
            )
        });

        self.create_resolution_history(
            &queue_item,
            "MERGE",
            &resolution_notes,
            resolved_by,
        )
        .await?;

        Ok(())
    }

    /// Skip this conflict (mark as skipped)
    async fn resolve_skip(
        &self,
        queue_item: SiorgSyncQueueItem,
        notes: Option<String>,
        resolved_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        info!("Skipping conflict for item {}", queue_item.id);

        let resolution_notes = notes
            .unwrap_or_else(|| "Conflict skipped, will be reviewed later".to_string());

        // Mark as skipped
        self.sync_queue_repo
            .update_status(
                queue_item.id,
                SyncStatus::Skipped,
                Some(resolution_notes.clone()),
                Some(json!({ "resolution": "SKIP", "resolved_by": resolved_by })),
            )
            .await?;

        // Create history entry
        self.create_resolution_history(
            &queue_item,
            "SKIP",
            &resolution_notes,
            resolved_by,
        )
        .await?;

        Ok(())
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Build merged payload based on field resolutions
    async fn build_merged_payload(
        &self,
        queue_item: &SiorgSyncQueueItem,
        field_resolutions: &HashMap<String, FieldResolution>,
    ) -> Result<serde_json::Value, ServiceError> {
        // This is a simplified version
        // In production, would need type-specific merging logic
        let mut merged = queue_item.payload.clone();

        // Apply field resolutions
        if let Some(obj) = merged.as_object_mut() {
            for (field, resolution) in field_resolutions {
                match resolution {
                    FieldResolution::Local => {
                        // Keep local value (already in place)
                        info!("Field '{}': keeping LOCAL value", field);
                    }
                    FieldResolution::Siorg => {
                        // SIORG value is already in payload
                        info!("Field '{}': using SIORG value", field);
                    }
                }
            }
        }

        Ok(merged)
    }

    /// Create history entry for resolution
    async fn create_resolution_history(
        &self,
        queue_item: &SiorgSyncQueueItem,
        resolution_type: &str,
        notes: &str,
        resolved_by: Option<Uuid>,
    ) -> Result<(), ServiceError> {
        let payload = CreateHistoryItemPayload {
            entity_type: queue_item.entity_type,
            siorg_code: queue_item.siorg_code,
            local_id: queue_item.local_id,
            change_type: SiorgChangeType::Update,
            previous_data: None,
            new_data: Some(json!({
                "resolution": resolution_type,
                "notes": notes
            })),
            affected_fields: vec!["resolution".to_string()],
            siorg_version: None,
            source: "MANUAL".to_string(),
            sync_queue_id: Some(queue_item.id),
            requires_review: false,
            created_by: resolved_by,
        };

        self.history_repo.create(payload).await?;

        Ok(())
    }
}

// ============================================================================
// Service Errors
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => ServiceError::NotFound("Resource not found".to_string()),
            RepositoryError::Duplicate(msg) => ServiceError::InvalidOperation(format!("Duplicate: {}", msg)),
            RepositoryError::Database(msg) => ServiceError::RepositoryError(msg),
            RepositoryError::ForeignKey(msg) => ServiceError::InvalidOperation(format!("Foreign key constraint: {}", msg)),
            RepositoryError::InvalidData(msg) => ServiceError::InvalidOperation(msg),
            RepositoryError::Transaction(msg) => ServiceError::RepositoryError(format!("Transaction error: {}", msg)),
        }
    }
}
