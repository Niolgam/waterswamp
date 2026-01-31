use crate::errors::ServiceError;
use domain::{
    models::requisition::*,
    ports::requisition::*,
};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Requisition Service
// ============================================================================

pub struct RequisitionService {
    requisition_repo: Arc<dyn RequisitionRepositoryPort>,
    item_repo: Arc<dyn RequisitionItemRepositoryPort>,
}

impl RequisitionService {
    pub fn new(
        requisition_repo: Arc<dyn RequisitionRepositoryPort>,
        item_repo: Arc<dyn RequisitionItemRepositoryPort>,
    ) -> Self {
        Self {
            requisition_repo,
            item_repo,
        }
    }

    // ========================================================================
    // AUDIT CONTEXT HELPERS
    // ========================================================================

    /// Sets the audit context before performing operations.
    /// This is essential for the database triggers to capture who performed the action.
    pub async fn set_audit_context(&self, ctx: &AuditContext) -> Result<(), ServiceError> {
        self.requisition_repo
            .set_audit_context(
                ctx.user_id,
                ctx.ip_address.as_deref(),
                ctx.user_agent.as_deref(),
            )
            .await
            .map_err(ServiceError::from)
    }

    // ========================================================================
    // REQUISITION OPERATIONS
    // ========================================================================

    /// Get a requisition by ID
    pub async fn get_requisition(&self, id: Uuid) -> Result<RequisitionDto, ServiceError> {
        self.requisition_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))
    }

    /// Get a requisition by number
    pub async fn get_requisition_by_number(
        &self,
        number: &str,
    ) -> Result<RequisitionDto, ServiceError> {
        self.requisition_repo
            .find_by_number(number)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))
    }

    /// Approve a requisition
    pub async fn approve_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: ApproveRequisitionPayload,
    ) -> Result<RequisitionDto, ServiceError> {
        // Verify requisition exists and is in pending status
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser aprovada: status atual é {:?}",
                requisition.status
            )));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Approve
        self.requisition_repo
            .approve(id, ctx.user_id, payload.notes.as_deref())
            .await
            .map_err(ServiceError::from)
    }

    /// Reject a requisition
    pub async fn reject_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: RejectRequisitionPayload,
    ) -> Result<RequisitionDto, ServiceError> {
        // Verify requisition exists and is in pending status
        let requisition = self.get_requisition(id).await?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser rejeitada: status atual é {:?}",
                requisition.status
            )));
        }

        // Validate reason
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para rejeição".to_string(),
            ));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Reject
        self.requisition_repo
            .reject(id, ctx.user_id, &payload.reason)
            .await
            .map_err(ServiceError::from)
    }

    /// Cancel a requisition using the database function with built-in validations
    pub async fn cancel_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: CancelRequisitionPayload,
    ) -> Result<serde_json::Value, ServiceError> {
        // Validate reason
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para cancelamento".to_string(),
            ));
        }

        // The database function will handle all validations:
        // - Check if requisition exists
        // - Check if status allows cancellation
        // - Check if there are stock movements
        // - Release reserves if any
        self.requisition_repo
            .cancel(id, &payload.reason, ctx.user_id)
            .await
            .map_err(|e| {
                // Map database errors to friendly messages
                if let domain::errors::RepositoryError::Database(ref msg) = e {
                    return ServiceError::BadRequest(msg.clone());
                }
                ServiceError::from(e)
            })
    }

    /// Rollback a requisition to a previous state
    pub async fn rollback_requisition(
        &self,
        id: Uuid,
        ctx: &AuditContext,
        payload: RollbackPayload,
    ) -> Result<serde_json::Value, ServiceError> {
        // Validate reason
        if payload.reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para rollback".to_string(),
            ));
        }

        // The database function will handle all validations:
        // - Check if requisition exists
        // - Check if history point is valid
        // - Check if status allows rollback
        // - Check for stock movements after target point
        self.requisition_repo
            .rollback(id, payload.history_id, &payload.reason, ctx.user_id)
            .await
            .map_err(|e| {
                if let domain::errors::RepositoryError::Database(ref msg) = e {
                    return ServiceError::BadRequest(msg.clone());
                }
                ServiceError::from(e)
            })
    }

    /// List requisitions with pagination and filters
    pub async fn list_requisitions(
        &self,
        limit: i64,
        offset: i64,
        status: Option<RequisitionStatus>,
        requester_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<RequisitionDto>, i64), ServiceError> {
        self.requisition_repo
            .list(limit, offset, status, requester_id, warehouse_id)
            .await
            .map_err(ServiceError::from)
    }

    // ========================================================================
    // AUDIT / HISTORY OPERATIONS
    // ========================================================================

    /// Get the audit history for a requisition
    pub async fn get_requisition_history(
        &self,
        id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<RequisitionHistoryEntry>, ServiceError> {
        // Verify requisition exists (or existed)
        // Note: We allow getting history even for deleted requisitions

        self.requisition_repo
            .get_history(id, limit.unwrap_or(50))
            .await
            .map_err(ServiceError::from)
    }

    /// Get available rollback points for a requisition
    pub async fn get_rollback_points(
        &self,
        id: Uuid,
        limit: Option<i32>,
    ) -> Result<Vec<RollbackPoint>, ServiceError> {
        // Verify requisition exists
        let _ = self.get_requisition(id).await?;

        self.requisition_repo
            .get_rollback_points(id, limit.unwrap_or(20))
            .await
            .map_err(ServiceError::from)
    }

    // ========================================================================
    // REQUISITION ITEM OPERATIONS
    // ========================================================================

    /// Get items for a requisition
    pub async fn get_requisition_items(
        &self,
        requisition_id: Uuid,
    ) -> Result<Vec<RequisitionItemDto>, ServiceError> {
        // Verify requisition exists
        let _ = self.get_requisition(requisition_id).await?;

        self.item_repo
            .find_by_requisition_id(requisition_id)
            .await
            .map_err(ServiceError::from)
    }

    /// Soft delete a requisition item
    pub async fn soft_delete_item(
        &self,
        item_id: Uuid,
        ctx: &AuditContext,
        reason: &str,
    ) -> Result<(), ServiceError> {
        // Validate reason
        if reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Justificativa é obrigatória para exclusão de item".to_string(),
            ));
        }

        // Verify item exists
        let item = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or(ServiceError::NotFound("Item não encontrado".to_string()))?;

        // Verify item is not already deleted
        if item.deleted_at.is_some() {
            return Err(ServiceError::BadRequest(
                "Item já foi excluído".to_string(),
            ));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Soft delete
        self.item_repo
            .soft_delete(item_id, ctx.user_id, reason)
            .await
            .map_err(ServiceError::from)
    }

    /// Restore a soft-deleted requisition item
    pub async fn restore_item(
        &self,
        item_id: Uuid,
        ctx: &AuditContext,
    ) -> Result<RequisitionItemDto, ServiceError> {
        // Verify item exists
        let item = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or(ServiceError::NotFound("Item não encontrado".to_string()))?;

        // Verify item is deleted
        if item.deleted_at.is_none() {
            return Err(ServiceError::BadRequest(
                "Item não está excluído".to_string(),
            ));
        }

        // Set audit context
        self.set_audit_context(ctx).await?;

        // Restore
        self.item_repo
            .restore(item_id)
            .await
            .map_err(ServiceError::from)
    }
}
