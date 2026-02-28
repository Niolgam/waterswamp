use chrono::{DateTime, Utc};
use domain::models::requisition::{RequisitionHistoryEntry, RequisitionStatus, RollbackPoint};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RequisitionListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<RequisitionStatus>,
    pub requester_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RollbackPointsQuery {
    pub limit: Option<i32>,
}

// ============================================================================
// Request Payloads
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub history_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteItemRequest {
    pub reason: String,
}

// ============================================================================
// Response DTOs
// ============================================================================

#[derive(Debug, Serialize)]
pub struct RequisitionListResponse {
    pub data: Vec<RequisitionResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct RequisitionResponse {
    pub id: Uuid,
    pub requisition_number: String,
    pub warehouse_id: Uuid,
    pub destination_unit_id: Option<Uuid>,
    pub requester_id: Uuid,
    pub status: String,
    pub priority: String,
    pub total_value: Option<rust_decimal::Decimal>,
    pub request_date: chrono::NaiveDate,
    pub needed_by: Option<chrono::NaiveDate>,
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

#[derive(Debug, Serialize)]
pub struct HistoryListResponse {
    pub data: Vec<HistoryEntry>,
    pub requisition_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct HistoryEntry {
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

#[derive(Debug, Serialize)]
pub struct RollbackPointsResponse {
    pub data: Vec<RollbackPointEntry>,
    pub requisition_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RollbackPointEntry {
    pub history_id: Uuid,
    pub operation: String,
    pub status_after: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub performed_by_name: Option<String>,
    pub changed_fields: Option<Vec<String>>,
    pub can_rollback: bool,
    pub rollback_blocked_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ItemResponse {
    pub id: Uuid,
    pub requisition_id: Uuid,
    pub catmat_item_id: Option<Uuid>,
    pub catser_item_id: Option<Uuid>,
    pub requested_quantity: rust_decimal::Decimal,
    pub approved_quantity: Option<rust_decimal::Decimal>,
    pub fulfilled_quantity: Option<rust_decimal::Decimal>,
    pub unit_value: Option<rust_decimal::Decimal>,
    pub total_value: Option<rust_decimal::Decimal>,
    pub justification: Option<String>,
    pub cut_reason: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub deletion_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Conversions
// ============================================================================

impl From<domain::models::requisition::RequisitionDto> for RequisitionResponse {
    fn from(dto: domain::models::requisition::RequisitionDto) -> Self {
        Self {
            id: dto.id,
            requisition_number: dto.requisition_number,
            warehouse_id: dto.warehouse_id,
            destination_unit_id: dto.destination_unit_id,
            requester_id: dto.requester_id,
            status: format!("{:?}", dto.status),
            priority: format!("{:?}", dto.priority),
            total_value: dto.total_value,
            request_date: dto.request_date.date_naive(),
            needed_by: dto.needed_by,
            approved_by: dto.approved_by,
            approved_at: dto.approved_at,
            fulfilled_by: dto.fulfilled_by,
            fulfilled_at: dto.fulfilled_at,
            rejection_reason: dto.rejection_reason,
            cancellation_reason: dto.cancellation_reason,
            notes: dto.notes,
            internal_notes: dto.internal_notes,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
        }
    }
}

impl From<RequisitionHistoryEntry> for HistoryEntry {
    fn from(entry: RequisitionHistoryEntry) -> Self {
        Self {
            history_id: entry.history_id,
            operation: entry.operation,
            status_before: entry.status_before,
            status_after: entry.status_after,
            changed_fields: entry.changed_fields,
            performed_at: entry.performed_at,
            performed_by: entry.performed_by,
            performed_by_name: entry.performed_by_name,
            reason: entry.reason,
            summary: entry.summary,
        }
    }
}

impl From<RollbackPoint> for RollbackPointEntry {
    fn from(point: RollbackPoint) -> Self {
        Self {
            history_id: point.history_id,
            operation: point.operation,
            status_after: point.status_after,
            performed_at: point.performed_at,
            performed_by_name: point.performed_by_name,
            changed_fields: point.changed_fields,
            can_rollback: point.can_rollback,
            rollback_blocked_reason: point.rollback_blocked_reason,
        }
    }
}

impl From<domain::models::requisition::RequisitionItemDto> for ItemResponse {
    fn from(dto: domain::models::requisition::RequisitionItemDto) -> Self {
        Self {
            id: dto.id,
            requisition_id: dto.requisition_id,
            catmat_item_id: dto.catmat_item_id,
            catser_item_id: dto.catser_item_id,
            requested_quantity: dto.requested_quantity,
            approved_quantity: dto.approved_quantity,
            fulfilled_quantity: dto.fulfilled_quantity,
            unit_value: dto.unit_value,
            total_value: dto.total_value,
            justification: dto.justification,
            cut_reason: dto.cut_reason,
            deleted_at: dto.deleted_at,
            deleted_by: dto.deleted_by,
            deletion_reason: dto.deletion_reason,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
        }
    }
}
