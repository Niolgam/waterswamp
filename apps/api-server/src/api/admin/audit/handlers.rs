use super::contracts::{AuditLogFilter, AuditLogListResponse};
use crate::infra::{errors::AppError, state::AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use domain::models::AuditLogEntry; // Necessário para conversão
use persistence::repositories::audit_logs_repository::AuditLogRepository;

/// GET /admin/audit/logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(filter): Query<AuditLogFilter>,
) -> Result<Json<AuditLogListResponse>, AppError> {
    let page = filter.page.unwrap_or(1).max(1);
    let limit = filter.limit.unwrap_or(20).max(1).min(100);
    let offset = (page - 1) * limit;

    let repo = AuditLogRepository::new(&state.db_pool_logs);

    // Repositório retorna (Vec<AuditLogEntryRow>, i64)
    let (logs_rows, total) = repo
        .list(
            limit,
            offset,
            filter.user_id,
            None,         // action
            None,         // resource
            None,         // ip_address
            None,         // status_code
            None,         // min_status_code
            None,         // max_status_code
            None,         // start_date
            None,         // end_date
            "created_at", // sort_by
            "DESC",       // sort_order
        )
        .await?;

    // Converter Row para DTO de domínio
    let logs: Vec<AuditLogEntry> = logs_rows
        .into_iter()
        .map(|row| AuditLogEntry {
            id: row.id,
            user_id: row.user_id,
            username: row.username,
            action: row.action,
            resource: row.resource,
            method: row.method,
            status_code: row.status_code,
            details: row.details,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            request_id: row.request_id,
            duration_ms: row.duration_ms,
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(AuditLogListResponse {
        data: logs,
        total,
        page,
        limit,
    }))
}
