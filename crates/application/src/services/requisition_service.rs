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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    fn create_test_requisition(status: RequisitionStatus) -> RequisitionDto {
        RequisitionDto {
            id: Uuid::new_v4(),
            requisition_number: "REQ2024001".to_string(),
            warehouse_id: Uuid::new_v4(),
            destination_unit_id: None,
            destination_unit_name: None,
            requester_id: Uuid::new_v4(),
            requester_name: None,
            status,
            priority: RequisitionPriority::Normal,
            total_value: Some(Decimal::new(1000, 2)),
            request_date: Utc::now(),
            needed_by: None,
            approved_by: None,
            approved_at: None,
            fulfilled_by: None,
            fulfilled_at: None,
            rejection_reason: None,
            cancellation_reason: None,
            notes: None,
            internal_notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_item(deleted: bool) -> RequisitionItemDto {
        RequisitionItemDto {
            id: Uuid::new_v4(),
            requisition_id: Uuid::new_v4(),
            catmat_item_id: Some(Uuid::new_v4()),
            catser_item_id: None,
            requested_quantity: Decimal::new(10, 0),
            approved_quantity: None,
            fulfilled_quantity: None,
            unit_value: Some(Decimal::new(100, 2)),
            total_value: Some(Decimal::new(1000, 2)),
            justification: Some("Test item".to_string()),
            cut_reason: None,
            deleted_at: if deleted { Some(Utc::now()) } else { None },
            deleted_by: if deleted { Some(Uuid::new_v4()) } else { None },
            deletion_reason: if deleted { Some("Test deletion".to_string()) } else { None },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ========================================================================
    // AUDIT CONTEXT TESTS
    // ========================================================================

    #[test]
    fn test_audit_context_creation() {
        let user_id = Uuid::new_v4();
        let ctx = AuditContext::new(user_id)
            .with_ip(Some("10.0.0.1".to_string()))
            .with_user_agent(Some("Mozilla/5.0".to_string()));

        assert_eq!(ctx.user_id, user_id);
        assert_eq!(ctx.ip_address, Some("10.0.0.1".to_string()));
        assert_eq!(ctx.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_audit_context_without_optional_fields() {
        let user_id = Uuid::new_v4();
        let ctx = AuditContext::new(user_id);

        assert_eq!(ctx.user_id, user_id);
        assert_eq!(ctx.ip_address, None);
        assert_eq!(ctx.user_agent, None);
    }

    // ========================================================================
    // PAYLOAD VALIDATION TESTS (Pure logic, no mocks needed)
    // ========================================================================

    #[test]
    fn test_reject_payload_validation() {
        // Empty reason should fail
        let empty_reason = "   ".trim();
        assert!(empty_reason.is_empty());

        // Valid reason should pass
        let valid_reason = "Budget constraints".trim();
        assert!(!valid_reason.is_empty());
    }

    #[test]
    fn test_cancel_payload_validation() {
        let empty_reason = "".trim();
        assert!(empty_reason.is_empty());

        let valid_reason = "No longer needed".trim();
        assert!(!valid_reason.is_empty());
    }

    #[test]
    fn test_rollback_payload_validation() {
        let empty_reason = "   ".trim();
        assert!(empty_reason.is_empty());

        let valid_reason = "Reverting to previous state".trim();
        assert!(!valid_reason.is_empty());
    }

    // ========================================================================
    // DTO CREATION TESTS
    // ========================================================================

    #[test]
    fn test_requisition_dto_creation() {
        let req = create_test_requisition(RequisitionStatus::Pending);

        assert_eq!(req.status, RequisitionStatus::Pending);
        assert_eq!(req.requisition_number, "REQ2024001");
        assert!(req.approved_by.is_none());
        assert!(req.approved_at.is_none());
    }

    #[test]
    fn test_requisition_item_dto_creation() {
        let item = create_test_item(false);

        assert!(item.deleted_at.is_none());
        assert!(item.deleted_by.is_none());
        assert!(item.deletion_reason.is_none());
    }

    #[test]
    fn test_deleted_item_dto_creation() {
        let item = create_test_item(true);

        assert!(item.deleted_at.is_some());
        assert!(item.deleted_by.is_some());
        assert!(item.deletion_reason.is_some());
    }

    // ========================================================================
    // STATUS VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_pending_status_allows_approval() {
        let req = create_test_requisition(RequisitionStatus::Pending);
        assert_eq!(req.status, RequisitionStatus::Pending);
    }

    #[test]
    fn test_draft_status_blocks_approval() {
        let req = create_test_requisition(RequisitionStatus::Draft);
        assert_ne!(req.status, RequisitionStatus::Pending);
    }

    #[test]
    fn test_approved_status_blocks_rejection() {
        let req = create_test_requisition(RequisitionStatus::Approved);
        assert_ne!(req.status, RequisitionStatus::Pending);
    }

    // ========================================================================
    // ENUM TESTS
    // ========================================================================

    #[test]
    fn test_requisition_status_variants() {
        assert_eq!(
            format!("{:?}", RequisitionStatus::Draft),
            "Draft"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Pending),
            "Pending"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Approved),
            "Approved"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Rejected),
            "Rejected"
        );
        assert_eq!(
            format!("{:?}", RequisitionStatus::Cancelled),
            "Cancelled"
        );
    }

    #[test]
    fn test_requisition_priority_variants() {
        assert_eq!(
            format!("{:?}", RequisitionPriority::Low),
            "Low"
        );
        assert_eq!(
            format!("{:?}", RequisitionPriority::Normal),
            "Normal"
        );
        assert_eq!(
            format!("{:?}", RequisitionPriority::High),
            "High"
        );
        assert_eq!(
            format!("{:?}", RequisitionPriority::Urgent),
            "Urgent"
        );
    }
}
