use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "requisition_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequisitionStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Processing,
    Fulfilled,
    PartiallyFulfilled,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "requisition_priority_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequisitionPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_operation_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditOperation {
    Insert,
    Update,
    Delete,
    SoftDelete,
    Restore,
    Rollback,
    StatusChange,
    Approval,
    Rejection,
    Cancellation,
}

// ============================================================================
// REQUISITION DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequisitionDto {
    pub id: Uuid,
    pub requisition_number: String,
    pub warehouse_id: Uuid,
    pub destination_unit_id: Option<Uuid>,
    pub requester_id: Uuid,
    pub status: RequisitionStatus,
    pub priority: RequisitionPriority,
    pub total_value: Option<Decimal>,
    pub request_date: NaiveDate,
    pub needed_by: Option<NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub fulfilled_by: Option<Uuid>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub cancellation_reason: Option<String>,
    pub notes: Option<String>,
    pub internal_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequisitionItemDto {
    pub id: Uuid,
    pub requisition_id: Uuid,
    pub catalog_item_id: Uuid,
    pub requested_quantity: Decimal,
    pub approved_quantity: Option<Decimal>,
    pub fulfilled_quantity: Option<Decimal>,
    pub unit_value: Option<Decimal>,
    pub total_value: Option<Decimal>,
    pub justification: Option<String>,
    pub cut_reason: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub deletion_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// AUDIT / HISTORY DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequisitionHistoryEntry {
    pub history_id: Uuid,
    pub operation: String,
    pub status_before: Option<String>,
    pub status_after: Option<String>,
    pub changed_fields: Option<Vec<String>>,
    pub performed_at: DateTime<Utc>,
    pub performed_by: Uuid,
    pub performed_by_name: Option<String>,
    pub reason: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RollbackPoint {
    pub history_id: Uuid,
    pub operation: String,
    pub status_after: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub performed_by_name: Option<String>,
    pub changed_fields: Option<Vec<String>>,
    pub can_rollback: bool,
    pub rollback_blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub requisition_id: Uuid,
    pub rollback_history_id: Option<Uuid>,
    pub rolled_back_to: Option<Uuid>,
    pub previous_status: Option<String>,
    pub restored_status: Option<String>,
    pub changed_fields: Option<Vec<String>>,
    pub performed_at: Option<DateTime<Utc>>,
    pub performed_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResult {
    pub success: bool,
    pub requisition_id: Uuid,
    pub previous_status: String,
    pub new_status: String,
    pub reason: String,
}

// ============================================================================
// REQUEST PAYLOADS
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct RollbackPayload {
    pub history_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelRequisitionPayload {
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApproveRequisitionPayload {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RejectRequisitionPayload {
    pub reason: String,
}

// ============================================================================
// AUDIT CONTEXT
// ============================================================================

#[derive(Debug, Clone)]
pub struct AuditContext {
    pub user_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditContext {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_ip(mut self, ip: Option<String>) -> Self {
        self.ip_address = ip;
        self
    }

    pub fn with_user_agent(mut self, ua: Option<String>) -> Self {
        self.user_agent = ua;
        self
    }
}
