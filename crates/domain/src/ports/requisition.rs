use crate::errors::RepositoryError;
use crate::models::requisition::*;
use async_trait::async_trait;
use uuid::Uuid;

// ============================================================================
// Requisition Repository Port
// ============================================================================

#[async_trait]
pub trait RequisitionRepositoryPort: Send + Sync {
    /// Sets the audit context for the current database session/transaction
    async fn set_audit_context(
        &self,
        user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), RepositoryError>;

    /// Find requisition by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RequisitionDto>, RepositoryError>;

    /// Find requisition by number
    async fn find_by_number(&self, number: &str) -> Result<Option<RequisitionDto>, RepositoryError>;

    /// Update requisition status to approved
    async fn approve(
        &self,
        id: Uuid,
        approved_by: Uuid,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError>;

    /// Update requisition status to rejected
    async fn reject(
        &self,
        id: Uuid,
        rejected_by: Uuid,
        reason: &str,
    ) -> Result<RequisitionDto, RepositoryError>;

    /// Cancel requisition using the database function
    async fn cancel(
        &self,
        id: Uuid,
        reason: &str,
        user_id: Uuid,
    ) -> Result<serde_json::Value, RepositoryError>;

    /// Rollback requisition to a previous state
    async fn rollback(
        &self,
        id: Uuid,
        history_id: Uuid,
        reason: &str,
        user_id: Uuid,
    ) -> Result<serde_json::Value, RepositoryError>;

    /// Get requisition audit history
    async fn get_history(
        &self,
        id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequisitionHistoryEntry>, RepositoryError>;

    /// Get available rollback points
    async fn get_rollback_points(
        &self,
        id: Uuid,
        limit: i32,
    ) -> Result<Vec<RollbackPoint>, RepositoryError>;

    /// List requisitions with pagination and filters
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        status: Option<RequisitionStatus>,
        requester_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<RequisitionDto>, i64), RepositoryError>;
}

// ============================================================================
// Requisition Item Repository Port
// ============================================================================

#[async_trait]
pub trait RequisitionItemRepositoryPort: Send + Sync {
    /// Find item by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RequisitionItemDto>, RepositoryError>;

    /// Find all items for a requisition
    async fn find_by_requisition_id(
        &self,
        requisition_id: Uuid,
    ) -> Result<Vec<RequisitionItemDto>, RepositoryError>;

    /// Soft delete an item
    async fn soft_delete(
        &self,
        id: Uuid,
        deleted_by: Uuid,
        reason: &str,
    ) -> Result<(), RepositoryError>;

    /// Restore a soft-deleted item
    async fn restore(&self, id: Uuid) -> Result<RequisitionItemDto, RepositoryError>;
}
