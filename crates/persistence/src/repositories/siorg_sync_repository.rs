use async_trait::async_trait;
use chrono::Utc;
use domain::errors::RepositoryError;
use domain::models::organizational::*;
use domain::ports::{SiorgHistoryRepositoryPort, SiorgSyncQueueRepositoryPort};
use sqlx::PgPool;
use tracing::{error, warn};
use uuid::Uuid;

// ============================================================================
// SIORG Sync Queue Repository Implementation
// ============================================================================

pub struct SiorgSyncQueueRepository {
    pool: PgPool,
}

impl SiorgSyncQueueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SiorgSyncQueueRepositoryPort for SiorgSyncQueueRepository {
    async fn poll_next_batch(
        &self,
        batch_size: i32,
    ) -> Result<Vec<SiorgSyncQueueItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgSyncQueueItem,
            r#"
            UPDATE siorg_sync_queue
            SET status = 'PROCESSING'::sync_status_enum,
                last_attempt_at = NOW(),
                attempts = attempts + 1
            WHERE id IN (
                SELECT id
                FROM siorg_sync_queue
                WHERE status = 'PENDING'::sync_status_enum
                  AND scheduled_for <= NOW()
                  AND (expires_at IS NULL OR expires_at > NOW())
                  AND attempts < max_attempts
                ORDER BY priority ASC, scheduled_for ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                operation as "operation: SiorgChangeType",
                priority,
                payload,
                detected_changes,
                status as "status: SyncStatus",
                attempts,
                max_attempts,
                last_attempt_at,
                last_error,
                error_details,
                processed_at,
                processed_by,
                resolution_notes,
                scheduled_for,
                expires_at,
                created_at
            "#,
            batch_size
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to poll next batch from sync queue: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiorgSyncQueueItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgSyncQueueItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                operation as "operation: SiorgChangeType",
                priority,
                payload,
                detected_changes,
                status as "status: SyncStatus",
                attempts,
                max_attempts,
                last_attempt_at,
                last_error,
                error_details,
                processed_at,
                processed_by,
                resolution_notes,
                scheduled_for,
                expires_at,
                created_at
            FROM siorg_sync_queue
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to find sync queue item by id {}: {}", id, e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn list(
        &self,
        status: Option<SyncStatus>,
        entity_type: Option<SiorgEntityType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiorgSyncQueueItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgSyncQueueItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                operation as "operation: SiorgChangeType",
                priority,
                payload,
                detected_changes,
                status as "status: SyncStatus",
                attempts,
                max_attempts,
                last_attempt_at,
                last_error,
                error_details,
                processed_at,
                processed_by,
                resolution_notes,
                scheduled_for,
                expires_at,
                created_at
            FROM siorg_sync_queue
            WHERE ($1::sync_status_enum IS NULL OR status = $1)
              AND ($2::siorg_entity_type_enum IS NULL OR entity_type = $2)
            ORDER BY priority ASC, scheduled_for ASC
            LIMIT $3 OFFSET $4
            "#,
            status as Option<SyncStatus>,
            entity_type as Option<SiorgEntityType>,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to list sync queue items: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn count_by_status(&self, status: SyncStatus) -> Result<i64, RepositoryError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM siorg_sync_queue
            WHERE status = $1
            "#,
            status as SyncStatus
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to count sync queue items by status: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn get_conflicts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiorgSyncQueueItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgSyncQueueItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                operation as "operation: SiorgChangeType",
                priority,
                payload,
                detected_changes,
                status as "status: SyncStatus",
                attempts,
                max_attempts,
                last_attempt_at,
                last_error,
                error_details,
                processed_at,
                processed_by,
                resolution_notes,
                scheduled_for,
                expires_at,
                created_at
            FROM siorg_sync_queue
            WHERE status = 'CONFLICT'::sync_status_enum
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get conflicts: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn create(
        &self,
        payload: CreateSyncQueueItemPayload,
    ) -> Result<SiorgSyncQueueItem, RepositoryError> {
        let scheduled_for = payload.scheduled_for.unwrap_or_else(Utc::now);

        let result = sqlx::query_as!(
            SiorgSyncQueueItem,
            r#"
            INSERT INTO siorg_sync_queue (
                entity_type, siorg_code, local_id, operation, priority,
                payload, detected_changes, scheduled_for, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                operation as "operation: SiorgChangeType",
                priority,
                payload,
                detected_changes,
                status as "status: SyncStatus",
                attempts,
                max_attempts,
                last_attempt_at,
                last_error,
                error_details,
                processed_at,
                processed_by,
                resolution_notes,
                scheduled_for,
                expires_at,
                created_at
            "#,
            payload.entity_type as SiorgEntityType,
            payload.siorg_code,
            payload.local_id,
            payload.operation as SiorgChangeType,
            payload.priority,
            payload.payload,
            payload.detected_changes,
            scheduled_for,
            payload.expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create sync queue item: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: SyncStatus,
        error: Option<String>,
        error_details: Option<serde_json::Value>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE siorg_sync_queue
            SET status = $2,
                last_error = $3,
                error_details = $4
            WHERE id = $1
            "#,
            id,
            status as SyncStatus,
            error,
            error_details
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to update sync queue item status: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn mark_processing(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE siorg_sync_queue
            SET status = 'PROCESSING'::sync_status_enum,
                last_attempt_at = NOW(),
                attempts = attempts + 1
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to mark sync queue item as processing: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn mark_completed(
        &self,
        id: Uuid,
        processed_by: Option<Uuid>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE siorg_sync_queue
            SET status = 'COMPLETED'::sync_status_enum,
                processed_at = NOW(),
                processed_by = $2
            WHERE id = $1
            "#,
            id,
            processed_by
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to mark sync queue item as completed: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn mark_failed(
        &self,
        id: Uuid,
        error: String,
        error_details: Option<serde_json::Value>,
    ) -> Result<(), RepositoryError> {
        // Check if max attempts reached
        let item = self.find_by_id(id).await?;
        let item = item.ok_or_else(|| RepositoryError::NotFound)?;

        let new_status = if item.attempts + 1 >= item.max_attempts {
            SyncStatus::Failed
        } else {
            SyncStatus::Pending // Will retry
        };

        sqlx::query!(
            r#"
            UPDATE siorg_sync_queue
            SET status = $2,
                last_error = $3,
                error_details = $4,
                attempts = attempts + 1
            WHERE id = $1
            "#,
            id,
            new_status as SyncStatus,
            error,
            error_details
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to mark sync queue item as failed: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn mark_conflict(
        &self,
        id: Uuid,
        detected_changes: serde_json::Value,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE siorg_sync_queue
            SET status = 'CONFLICT'::sync_status_enum,
                detected_changes = $2
            WHERE id = $1
            "#,
            id,
            detected_changes
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to mark sync queue item as conflict: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn resolve(
        &self,
        id: Uuid,
        resolution_notes: String,
        processed_by: Option<Uuid>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE siorg_sync_queue
            SET status = 'COMPLETED'::sync_status_enum,
                resolution_notes = $2,
                processed_at = NOW(),
                processed_by = $3
            WHERE id = $1
            "#,
            id,
            resolution_notes,
            processed_by
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to resolve sync queue item: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            DELETE FROM siorg_sync_queue
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to delete sync queue item: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn clean_expired(&self) -> Result<i64, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM siorg_sync_queue
            WHERE expires_at IS NOT NULL
              AND expires_at < NOW()
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to clean expired sync queue items: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result.rows_affected() as i64)
    }
}

// ============================================================================
// SIORG History Repository Implementation
// ============================================================================

pub struct SiorgHistoryRepository {
    pool: PgPool,
}

impl SiorgHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SiorgHistoryRepositoryPort for SiorgHistoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SiorgHistoryItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgHistoryItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                change_type as "change_type: SiorgChangeType",
                previous_data,
                new_data,
                affected_fields,
                siorg_version,
                source,
                sync_queue_id,
                requires_review,
                reviewed_at,
                reviewed_by,
                review_notes,
                created_at,
                created_by
            FROM siorg_history
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to find history item by id {}: {}", id, e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn list(
        &self,
        entity_type: Option<SiorgEntityType>,
        siorg_code: Option<i32>,
        change_type: Option<SiorgChangeType>,
        requires_review: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiorgHistoryItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgHistoryItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                change_type as "change_type: SiorgChangeType",
                previous_data,
                new_data,
                affected_fields,
                siorg_version,
                source,
                sync_queue_id,
                requires_review,
                reviewed_at,
                reviewed_by,
                review_notes,
                created_at,
                created_by
            FROM siorg_history
            WHERE ($1::siorg_entity_type_enum IS NULL OR entity_type = $1)
              AND ($2::INTEGER IS NULL OR siorg_code = $2)
              AND ($3::siorg_change_type_enum IS NULL OR change_type = $3)
              AND ($4::BOOLEAN IS NULL OR requires_review = $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            entity_type as Option<SiorgEntityType>,
            siorg_code,
            change_type as Option<SiorgChangeType>,
            requires_review,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to list history items: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn get_entity_history(
        &self,
        entity_type: SiorgEntityType,
        siorg_code: i32,
        limit: i64,
    ) -> Result<Vec<SiorgHistoryItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgHistoryItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                change_type as "change_type: SiorgChangeType",
                previous_data,
                new_data,
                affected_fields,
                siorg_version,
                source,
                sync_queue_id,
                requires_review,
                reviewed_at,
                reviewed_by,
                review_notes,
                created_at,
                created_by
            FROM siorg_history
            WHERE entity_type = $1 AND siorg_code = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            entity_type as SiorgEntityType,
            siorg_code,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Failed to get entity history for {:?} code {}: {}",
                entity_type, siorg_code, e
            );
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn get_pending_reviews(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiorgHistoryItem>, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgHistoryItem,
            r#"
            SELECT
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                change_type as "change_type: SiorgChangeType",
                previous_data,
                new_data,
                affected_fields,
                siorg_version,
                source,
                sync_queue_id,
                requires_review,
                reviewed_at,
                reviewed_by,
                review_notes,
                created_at,
                created_by
            FROM siorg_history
            WHERE requires_review = TRUE AND reviewed_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get pending reviews: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn create(
        &self,
        payload: CreateHistoryItemPayload,
    ) -> Result<SiorgHistoryItem, RepositoryError> {
        let result = sqlx::query_as!(
            SiorgHistoryItem,
            r#"
            INSERT INTO siorg_history (
                entity_type, siorg_code, local_id, change_type,
                previous_data, new_data, affected_fields,
                siorg_version, source, sync_queue_id,
                requires_review, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id,
                entity_type as "entity_type: SiorgEntityType",
                siorg_code,
                local_id,
                change_type as "change_type: SiorgChangeType",
                previous_data,
                new_data,
                affected_fields,
                siorg_version,
                source,
                sync_queue_id,
                requires_review,
                reviewed_at,
                reviewed_by,
                review_notes,
                created_at,
                created_by
            "#,
            payload.entity_type as SiorgEntityType,
            payload.siorg_code,
            payload.local_id,
            payload.change_type as SiorgChangeType,
            payload.previous_data,
            payload.new_data,
            &payload.affected_fields,
            payload.siorg_version,
            payload.source,
            payload.sync_queue_id,
            payload.requires_review,
            payload.created_by
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create history item: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }

    async fn mark_reviewed(
        &self,
        id: Uuid,
        reviewed_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE siorg_history
            SET reviewed_at = NOW(),
                reviewed_by = $2,
                review_notes = $3
            WHERE id = $1
            "#,
            id,
            reviewed_by,
            notes
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to mark history item as reviewed: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(())
    }

    async fn count(
        &self,
        entity_type: Option<SiorgEntityType>,
    ) -> Result<i64, RepositoryError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM siorg_history
            WHERE ($1::siorg_entity_type_enum IS NULL OR entity_type = $1)
            "#,
            entity_type as Option<SiorgEntityType>
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to count history items: {}", e);
            RepositoryError::Database(e.to_string())
        })?;

        Ok(result)
    }
}
