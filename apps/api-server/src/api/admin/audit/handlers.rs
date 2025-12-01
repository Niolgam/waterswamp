use super::contracts::{
    ActionCountDto, AuditLogFilter, AuditLogListResponse, AuditLogStatsResponse, CleanupRequest,
    ResourceCountDto, SuspiciousIpDto,
};
use crate::infra::{errors::AppError, state::AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use domain::models::AuditLogEntry;
use persistence::repositories::audit_logs_repository::AuditLogRepository;
use serde::Deserialize;
use uuid::Uuid;

fn map_log_entry(
    row: persistence::repositories::audit_logs_repository::AuditLogEntryRow,
) -> AuditLogEntry {
    AuditLogEntry {
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

/// GET /admin/audit-logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(filter): Query<AuditLogFilter>,
) -> Result<Json<AuditLogListResponse>, AppError> {
    let limit = filter.limit.unwrap_or(20).max(1).min(100);

    // Tests use 'offset' directly, but we support 'page' for convenience.
    let offset = if let Some(o) = filter.offset {
        o.max(0)
    } else {
        let page = filter.page.unwrap_or(1).max(1);
        (page - 1) * limit
    };

    let repo = AuditLogRepository::new(&state.db_pool_logs);

    let (logs_rows, total) = repo
        .list(
            limit,
            offset,
            filter.user_id,
            filter.action.as_deref(),
            filter.resource.as_deref(),
            filter.ip_address.as_deref(),
            filter.status_code,
            filter.min_status_code,
            filter.max_status_code,
            filter.start_date,
            filter.end_date,
            filter.sort_by.as_deref().unwrap_or("created_at"),
            filter.sort_order.as_deref().unwrap_or("DESC"),
        )
        .await?;

    let logs: Vec<AuditLogEntry> = logs_rows.into_iter().map(map_log_entry).collect();

    Ok(Json(AuditLogListResponse {
        logs, // Renamed from data
        total,
        offset, // Renamed from page
        limit,
    }))
}

/// GET /admin/audit-logs/:id
pub async fn get_log(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditLogEntry>, AppError> {
    let repo = AuditLogRepository::new(&state.db_pool_logs);

    let log = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Audit log {} not found", id)))?;

    Ok(Json(map_log_entry(log)))
}

/// GET /admin/audit-logs/stats
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<AuditLogStatsResponse>, AppError> {
    let repo = AuditLogRepository::new(&state.db_pool_logs);
    let stats = repo.get_stats().await?;

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

/// GET /admin/audit-logs/failed-logins
#[derive(Debug, Deserialize)]
pub struct FailedLoginsQuery {
    pub hours: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_failed_logins(
    State(state): State<AppState>,
    Query(query): Query<FailedLoginsQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, AppError> {
    let repo = AuditLogRepository::new(&state.db_pool_logs);
    let hours = query.hours.unwrap_or(24);
    let limit = query.limit.unwrap_or(20);

    let logs = repo.get_failed_logins(hours, limit).await?;

    Ok(Json(logs.into_iter().map(map_log_entry).collect()))
}

/// GET /admin/audit-logs/suspicious-ips
#[derive(Debug, Deserialize)]
pub struct SuspiciousIpsQuery {
    pub hours: Option<i64>,
    pub threshold: Option<i64>,
}

pub async fn get_suspicious_ips(
    State(state): State<AppState>,
    Query(query): Query<SuspiciousIpsQuery>,
) -> Result<Json<Vec<SuspiciousIpDto>>, AppError> {
    let repo = AuditLogRepository::new(&state.db_pool_logs);
    let hours = query.hours.unwrap_or(24);
    let threshold = query.threshold.unwrap_or(5);

    let ips = repo.get_suspicious_ips(hours, threshold).await?;

    Ok(Json(
        ips.into_iter()
            .map(|ip| SuspiciousIpDto {
                ip_address: ip.ip_address,
                failed_attempts: ip.failed_attempts,
                unique_usernames: ip.unique_usernames,
                first_attempt: ip.first_attempt,
                last_attempt: ip.last_attempt,
            })
            .collect(),
    ))
}

/// GET /admin/audit-logs/user/:user_id
#[derive(Debug, Deserialize)]
pub struct UserLogsQuery {
    pub limit: Option<i64>,
}

pub async fn get_user_logs(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<UserLogsQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, AppError> {
    let repo = AuditLogRepository::new(&state.db_pool_logs);
    let limit = query.limit.unwrap_or(50);

    let logs = repo.get_user_logs(user_id, limit).await?;

    Ok(Json(logs.into_iter().map(map_log_entry).collect()))
}

/// POST /admin/audit-logs/cleanup
pub async fn cleanup_logs(
    State(state): State<AppState>,
    Json(payload): Json<CleanupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = AuditLogRepository::new(&state.db_pool_logs);
    let deleted = repo.cleanup_old_logs(payload.retention_days).await?;

    Ok(Json(serde_json::json!({
        "deleted_count": deleted,
        "message": format!("Deleted {} logs older than {} days", deleted, payload.retention_days)
    })))
}
