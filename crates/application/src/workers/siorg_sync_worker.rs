use crate::external::{SiorgClient, SiorgSyncService};
use domain::models::organizational::*;
use domain::ports::{
    OrganizationRepositoryPort, OrganizationalUnitCategoryRepositoryPort,
    OrganizationalUnitRepositoryPort, OrganizationalUnitTypeRepositoryPort,
    SiorgHistoryRepositoryPort, SiorgSyncQueueRepositoryPort,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

// ============================================================================
// Worker Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Number of items to process in each batch
    pub batch_size: i32,
    /// Interval between polls when queue is empty (seconds)
    pub poll_interval_secs: u64,
    /// Maximum number of retry attempts per item
    pub max_retries: i32,
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay_ms: u64,
    /// Maximum delay for exponential backoff (milliseconds)
    pub retry_max_delay_ms: u64,
    /// Whether to run cleanup of expired items
    pub enable_cleanup: bool,
    /// Cleanup interval (seconds)
    pub cleanup_interval_secs: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            poll_interval_secs: 5,
            max_retries: 3,
            retry_base_delay_ms: 1000, // 1 second
            retry_max_delay_ms: 60000,  // 1 minute
            enable_cleanup: true,
            cleanup_interval_secs: 3600, // 1 hour
        }
    }
}

// ============================================================================
// Processing Statistics
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    pub processed: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub conflicts: u64,
    pub skipped: u64,
}

impl ProcessingStats {
    pub fn increment_success(&mut self) {
        self.processed += 1;
        self.succeeded += 1;
    }

    pub fn increment_failure(&mut self) {
        self.processed += 1;
        self.failed += 1;
    }

    pub fn increment_conflict(&mut self) {
        self.processed += 1;
        self.conflicts += 1;
    }

    pub fn increment_skip(&mut self) {
        self.processed += 1;
        self.skipped += 1;
    }
}

// ============================================================================
// SIORG Sync Worker Core
// ============================================================================

pub struct SiorgSyncWorkerCore {
    config: WorkerConfig,
    sync_queue_repo: Arc<dyn SiorgSyncQueueRepositoryPort>,
    history_repo: Arc<dyn SiorgHistoryRepositoryPort>,
    sync_service: Arc<SiorgSyncService>,
}

impl SiorgSyncWorkerCore {
    pub fn new(
        config: WorkerConfig,
        sync_queue_repo: Arc<dyn SiorgSyncQueueRepositoryPort>,
        history_repo: Arc<dyn SiorgHistoryRepositoryPort>,
        sync_service: Arc<SiorgSyncService>,
    ) -> Self {
        Self {
            config,
            sync_queue_repo,
            history_repo,
            sync_service,
        }
    }

    /// Process next batch of pending items from queue
    pub async fn process_next_batch(&self) -> Result<ProcessingStats, String> {
        let mut stats = ProcessingStats::default();

        // Poll next batch with FOR UPDATE SKIP LOCKED
        let items = self
            .sync_queue_repo
            .poll_next_batch(self.config.batch_size)
            .await
            .map_err(|e| format!("Failed to poll queue: {}", e))?;

        if items.is_empty() {
            info!("No pending items in sync queue");
            return Ok(stats);
        }

        info!("Processing {} items from sync queue", items.len());

        // Process each item
        for item in items {
            match self.process_item(&item).await {
                Ok(ProcessingResult::Success) => {
                    stats.increment_success();
                    info!("Successfully processed item {}", item.id);
                }
                Ok(ProcessingResult::Conflict(changes)) => {
                    stats.increment_conflict();
                    warn!("Conflict detected for item {}", item.id);
                    if let Err(e) = self.sync_queue_repo.mark_conflict(item.id, changes).await {
                        error!("Failed to mark item {} as conflict: {}", item.id, e);
                    }
                }
                Ok(ProcessingResult::Skip(reason)) => {
                    stats.increment_skip();
                    info!("Skipped item {}: {}", item.id, reason);
                    if let Err(e) = self
                        .sync_queue_repo
                        .update_status(item.id, SyncStatus::Skipped, Some(reason), None)
                        .await
                    {
                        error!("Failed to mark item {} as skipped: {}", item.id, e);
                    }
                }
                Err(e) => {
                    stats.increment_failure();
                    error!("Failed to process item {}: {}", item.id, e);

                    // Apply exponential backoff based on attempts
                    let delay = self.calculate_backoff_delay(item.attempts);
                    info!("Will retry item {} after {}ms", item.id, delay.as_millis());

                    // Mark as failed (will retry if under max attempts)
                    let error_details = serde_json::json!({
                        "error": e.to_string(),
                        "attempt": item.attempts + 1,
                        "next_retry_delay_ms": delay.as_millis(),
                    });

                    if let Err(mark_err) = self
                        .sync_queue_repo
                        .mark_failed(item.id, e.to_string(), Some(error_details))
                        .await
                    {
                        error!("Failed to mark item {} as failed: {}", item.id, mark_err);
                    }
                }
            }
        }

        info!(
            "Batch complete: {} processed, {} succeeded, {} failed, {} conflicts, {} skipped",
            stats.processed, stats.succeeded, stats.failed, stats.conflicts, stats.skipped
        );

        Ok(stats)
    }

    /// Calculate exponential backoff delay based on attempt number
    fn calculate_backoff_delay(&self, attempts: i32) -> Duration {
        let base_delay = self.config.retry_base_delay_ms;
        let max_delay = self.config.retry_max_delay_ms;

        // Exponential: delay = base * 2^attempts
        let delay_ms = base_delay * 2_u64.pow(attempts.max(0) as u32);
        let capped_delay = delay_ms.min(max_delay);

        Duration::from_millis(capped_delay)
    }

    /// Process a single queue item
    async fn process_item(&self, item: &SiorgSyncQueueItem) -> Result<ProcessingResult, String> {
        info!(
            "Processing {:?} {} for {:?} (attempt {}/{})",
            item.operation, item.siorg_code, item.entity_type, item.attempts, item.max_attempts
        );

        match item.entity_type {
            SiorgEntityType::Organization => self.process_organization(item).await,
            SiorgEntityType::Unit => self.process_unit(item).await,
            SiorgEntityType::Category => {
                // Categories são gerenciadas localmente, skip
                Ok(ProcessingResult::Skip(
                    "Categories are managed locally".to_string(),
                ))
            }
            SiorgEntityType::Type => {
                // Types são gerenciados localmente, skip
                Ok(ProcessingResult::Skip(
                    "Types are managed locally".to_string(),
                ))
            }
        }
    }

    /// Process organization sync
    async fn process_organization(
        &self,
        item: &SiorgSyncQueueItem,
    ) -> Result<ProcessingResult, String> {
        match item.operation {
            SiorgChangeType::Creation | SiorgChangeType::Update => {
                // Sync organization from SIORG
                let org = self
                    .sync_service
                    .sync_organization(item.siorg_code)
                    .await
                    .map_err(|e| format!("Sync failed: {}", e))?;

                // Create history entry
                let history_payload = CreateHistoryItemPayload {
                    entity_type: SiorgEntityType::Organization,
                    siorg_code: item.siorg_code,
                    local_id: Some(org.id),
                    change_type: if item.operation == SiorgChangeType::Creation {
                        SiorgChangeType::Creation
                    } else {
                        SiorgChangeType::Update
                    },
                    previous_data: item.detected_changes.clone(),
                    new_data: Some(serde_json::to_value(&org).unwrap()),
                    affected_fields: vec![], // TODO: compute diff
                    siorg_version: None,
                    source: "SYNC".to_string(),
                    sync_queue_id: Some(item.id),
                    requires_review: false,
                    created_by: None,
                };

                self.history_repo
                    .create(history_payload)
                    .await
                    .map_err(|e| format!("Failed to create history: {}", e))?;

                // Mark as completed
                self.sync_queue_repo
                    .mark_completed(item.id, None)
                    .await
                    .map_err(|e| format!("Failed to mark completed: {}", e))?;

                Ok(ProcessingResult::Success)
            }
            SiorgChangeType::Extinction => {
                // For now, just mark organization as inactive
                // Could also be handled as conflict requiring manual review
                Ok(ProcessingResult::Skip(
                    "Extinction requires manual review".to_string(),
                ))
            }
            _ => Ok(ProcessingResult::Skip(format!(
                "Operation {:?} not implemented for organizations",
                item.operation
            ))),
        }
    }

    /// Process unit sync
    async fn process_unit(&self, item: &SiorgSyncQueueItem) -> Result<ProcessingResult, String> {
        match item.operation {
            SiorgChangeType::Creation | SiorgChangeType::Update => {
                // Sync unit from SIORG
                let unit = self
                    .sync_service
                    .sync_unit(item.siorg_code)
                    .await
                    .map_err(|e| format!("Sync failed: {}", e))?;

                // Create history entry
                let history_payload = CreateHistoryItemPayload {
                    entity_type: SiorgEntityType::Unit,
                    siorg_code: item.siorg_code,
                    local_id: Some(unit.id),
                    change_type: if item.operation == SiorgChangeType::Creation {
                        SiorgChangeType::Creation
                    } else {
                        SiorgChangeType::Update
                    },
                    previous_data: item.detected_changes.clone(),
                    new_data: Some(serde_json::to_value(&unit).unwrap()),
                    affected_fields: vec![], // TODO: compute diff
                    siorg_version: None,
                    source: "SYNC".to_string(),
                    sync_queue_id: Some(item.id),
                    requires_review: false,
                    created_by: None,
                };

                self.history_repo
                    .create(history_payload)
                    .await
                    .map_err(|e| format!("Failed to create history: {}", e))?;

                // Mark as completed
                self.sync_queue_repo
                    .mark_completed(item.id, None)
                    .await
                    .map_err(|e| format!("Failed to mark completed: {}", e))?;

                Ok(ProcessingResult::Success)
            }
            SiorgChangeType::Extinction => {
                Ok(ProcessingResult::Skip(
                    "Extinction requires manual review".to_string(),
                ))
            }
            SiorgChangeType::HierarchyChange => {
                // Hierarchy changes may cause conflicts, requires review
                Ok(ProcessingResult::Skip(
                    "Hierarchy changes require manual review".to_string(),
                ))
            }
            _ => Ok(ProcessingResult::Skip(format!(
                "Operation {:?} not implemented for units",
                item.operation
            ))),
        }
    }

    /// Clean expired items from queue
    pub async fn cleanup_expired(&self) -> Result<i64, String> {
        info!("Running cleanup of expired items");

        let deleted = self
            .sync_queue_repo
            .clean_expired()
            .await
            .map_err(|e| format!("Cleanup failed: {}", e))?;

        if deleted > 0 {
            info!("Cleaned up {} expired items", deleted);
        }

        Ok(deleted)
    }

    /// Run worker loop continuously
    pub async fn run_forever(&self) -> Result<(), String> {
        info!("Starting SIORG sync worker (continuous mode)");
        info!("Config: {:?}", self.config);

        let mut cleanup_counter = 0u64;
        let poll_interval = Duration::from_secs(self.config.poll_interval_secs);
        let cleanup_threshold =
            self.config.cleanup_interval_secs / self.config.poll_interval_secs;

        loop {
            // Process next batch
            match self.process_next_batch().await {
                Ok(stats) if stats.processed == 0 => {
                    // Queue is empty, wait before next poll
                    sleep(poll_interval).await;
                }
                Ok(_stats) => {
                    // Items were processed, immediately check for more
                    // (no sleep, maximize throughput)
                }
                Err(e) => {
                    error!("Error processing batch: {}", e);
                    // Wait before retrying to avoid spinning
                    sleep(poll_interval).await;
                }
            }

            // Periodic cleanup
            if self.config.enable_cleanup {
                cleanup_counter += 1;
                if cleanup_counter >= cleanup_threshold {
                    cleanup_counter = 0;
                    if let Err(e) = self.cleanup_expired().await {
                        error!("Cleanup failed: {}", e);
                    }
                }
            }
        }
    }

    /// Run worker for a single pass (useful for testing or cron jobs)
    pub async fn run_once(&self) -> Result<ProcessingStats, String> {
        info!("Starting SIORG sync worker (single pass)");
        self.process_next_batch().await
    }
}

// ============================================================================
// Processing Result
// ============================================================================

enum ProcessingResult {
    Success,
    Conflict(serde_json::Value),
    Skip(String),
}
