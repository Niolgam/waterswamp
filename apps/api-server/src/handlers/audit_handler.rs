use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{error::AppError, state::AppState};
use persistence::repositories::audit_logs_repository::{AuditLogEntryRow, AuditLogRepository};

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

/// Query parameters for listing audit logs
#[derive(Debug, Deserialize, Validate)]
pub struct ListAuditLogsQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,

    #[validate(range(min = 0))]
    pub offset: Option<i64>,

    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub ip_address: Option<String>,
    pub status_code: Option<i32>,
    pub min_status_code: Option<i32>,
    pub max_status_code: Option<i32>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub sort_order: Option<String>,
    pub sort_by: Option<String>,
}

/// Response for paginated audit logs
#[derive(Debug, Serialize)]
pub struct PaginatedAuditLogsResponse {
    pub logs: Vec<AuditLogDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// DTO for audit log entry
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub action: String,
    pub resource: String,
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub duration_ms: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response for audit log statistics
#[derive(Debug, Serialize)]
pub struct AuditLogStatsResponse {
    pub total_logs: i64,
    pub logs_today: i64,
    pub logs_this_week: i64,
    pub failed_logins_today: i64,
    pub unique_users_today: i64,
    pub top_actions: Vec<ActionCountDto>,
    pub top_resources: Vec<ResourceCountDto>,
}

#[derive(Debug, Serialize)]
pub struct ActionCountDto {
    pub action: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ResourceCountDto {
    pub resource: String,
    pub count: i64,
}

/// Response for suspicious IPs
#[derive(Debug, Serialize)]
pub struct SuspiciousIpDto {
    pub ip_address: Option<String>,
    pub failed_attempts: i64,
    pub unique_usernames: i64,
    pub first_attempt: chrono::DateTime<chrono::Utc>,
    pub last_attempt: chrono::DateTime<chrono::Utc>,
}

/// Query for suspicious IPs
#[derive(Debug, Deserialize, Validate)]
pub struct SuspiciousIpsQuery {
    #[validate(range(min = 1, max = 168))]
    pub hours: Option<i64>,

    #[validate(range(min = 1, max = 100))]
    pub threshold: Option<i64>,
}

/// Query for failed logins
#[derive(Debug, Deserialize, Validate)]
pub struct FailedLoginsQuery {
    #[validate(range(min = 1, max = 168))]
    pub hours: Option<i64>,

    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<i64>,
}

/// Request for cleaning up old logs
#[derive(Debug, Deserialize, Validate)]
pub struct CleanupLogsRequest {
    #[validate(range(min = 1, max = 365))]
    pub retention_days: i64,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// GET /api/admin/audit-logs
/// Lists audit logs with filtering, pagination, and sorting.
pub async fn list_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<ListAuditLogsQuery>,
) -> Result<Json<PaginatedAuditLogsResponse>, AppError> {
    params.validate()?;

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");

    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let (logs, total) = audit_repo
        .list(
            limit,
            offset,
            params.user_id,
            params.action.as_deref(),
            params.resource.as_deref(),
            params.ip_address.as_deref(),
            params.status_code,
            params.min_status_code,
            params.max_status_code,
            params.start_date,
            params.end_date,
            sort_by,
            sort_order,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "Failed to fetch audit logs");
            AppError::Database(e)
        })?;

    let log_dtos: Vec<AuditLogDto> = logs.into_iter().map(|row| row.into()).collect();

    Ok(Json(PaginatedAuditLogsResponse {
        logs: log_dtos,
        total,
        limit,
        offset,
    }))
}

/// GET /api/admin/audit-logs/{id}
/// Gets a specific audit log entry by ID.
pub async fn get_audit_log(
    State(state): State<AppState>,
    Path(log_id): Path<Uuid>,
) -> Result<Json<AuditLogDto>, AppError> {
    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let log = audit_repo
        .find_by_id(log_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Audit log entry not found".to_string()))?;

    Ok(Json(log.into()))
}

/// GET /api/admin/audit-logs/stats
/// Gets audit log statistics.
pub async fn get_audit_stats(
    State(state): State<AppState>,
) -> Result<Json<AuditLogStatsResponse>, AppError> {
    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let stats = audit_repo.get_stats().await.map_err(|e| {
        tracing::error!(error = ?e, "Failed to fetch audit stats");
        AppError::Database(e)
    })?;

    Ok(Json(AuditLogStatsResponse {
        total_logs: stats.total_logs,
        logs_today: stats.logs_today,
        logs_this_week: stats.logs_this_week,
        failed_logins_today: stats.failed_logins_today,
        unique_users_today: stats.unique_users_today,
        top_actions: stats
            .top_actions
            .into_iter()
            .map(|a| ActionCountDto {
                action: a.action,
                count: a.count,
            })
            .collect(),
        top_resources: stats
            .top_resources
            .into_iter()
            .map(|r| ResourceCountDto {
                resource: r.resource,
                count: r.count,
            })
            .collect(),
    }))
}

/// GET /api/admin/audit-logs/user/{user_id}
/// Gets audit logs for a specific user.
pub async fn get_user_audit_logs(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<FailedLoginsQuery>,
) -> Result<Json<Vec<AuditLogDto>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(1000);

    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let logs = audit_repo.get_user_logs(user_id, limit).await?;

    let log_dtos: Vec<AuditLogDto> = logs.into_iter().map(|row| row.into()).collect();

    Ok(Json(log_dtos))
}

/// GET /api/admin/audit-logs/failed-logins
/// Gets recent failed login attempts.
pub async fn get_failed_logins(
    State(state): State<AppState>,
    Query(params): Query<FailedLoginsQuery>,
) -> Result<Json<Vec<AuditLogDto>>, AppError> {
    params.validate()?;

    let hours = params.hours.unwrap_or(24);
    let limit = params.limit.unwrap_or(100);

    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let logs = audit_repo.get_failed_logins(hours, limit).await?;

    let log_dtos: Vec<AuditLogDto> = logs.into_iter().map(|row| row.into()).collect();

    Ok(Json(log_dtos))
}

/// GET /api/admin/audit-logs/suspicious-ips
/// Gets IPs with multiple failed login attempts (potential attacks).
pub async fn get_suspicious_ips(
    State(state): State<AppState>,
    Query(params): Query<SuspiciousIpsQuery>,
) -> Result<Json<Vec<SuspiciousIpDto>>, AppError> {
    params.validate()?;

    let hours = params.hours.unwrap_or(24);
    let threshold = params.threshold.unwrap_or(5);

    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let suspicious = audit_repo.get_suspicious_ips(hours, threshold).await?;

    let suspicious_dtos: Vec<SuspiciousIpDto> = suspicious
        .into_iter()
        .map(|row| SuspiciousIpDto {
            ip_address: row.ip_address,
            failed_attempts: row.failed_attempts,
            unique_usernames: row.unique_usernames,
            first_attempt: row.first_attempt,
            last_attempt: row.last_attempt,
        })
        .collect();

    Ok(Json(suspicious_dtos))
}

/// POST /api/admin/audit-logs/cleanup
/// Cleans up old audit logs based on retention policy.
/// WARNING: This is a destructive operation!
pub async fn cleanup_old_logs(
    State(state): State<AppState>,
    Json(payload): Json<CleanupLogsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;

    let audit_repo = AuditLogRepository::new(&state.db_pool_logs);

    let deleted_count = audit_repo.cleanup_old_logs(payload.retention_days).await?;

    tracing::warn!(
        deleted_count = deleted_count,
        retention_days = payload.retention_days,
        event_type = "audit_logs_cleanup",
        "Old audit logs cleaned up"
    );

    Ok(Json(serde_json::json!({
        "message": format!("Cleaned up {} old audit logs", deleted_count),
        "deleted_count": deleted_count,
        "retention_days": payload.retention_days
    })))
}

// =============================================================================
// CONVERSIONS
// =============================================================================

impl From<AuditLogEntryRow> for AuditLogDto {
    fn from(row: AuditLogEntryRow) -> Self {
        Self {
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
        }
    }
}
