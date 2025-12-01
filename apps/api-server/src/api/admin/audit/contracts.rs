use chrono::{DateTime, Utc};
use domain::models::AuditLogEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuditLogFilter {
    pub page: Option<i64>,
    pub offset: Option<i64>, // Added to support explicit offset from tests
    pub limit: Option<i64>,
    pub user_id: Option<uuid::Uuid>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub ip_address: Option<String>,
    pub status_code: Option<i32>,
    pub min_status_code: Option<i32>,
    pub max_status_code: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLogEntry>, // Renamed from 'data' to 'logs'
    pub total: i64,
    pub offset: i64, // Changed 'page' to 'offset'
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    pub retention_days: i64,
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
pub struct SuspiciousIpDto {
    pub ip_address: Option<String>,
    pub failed_attempts: i64,
    pub unique_usernames: i64,
    pub first_attempt: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
}
