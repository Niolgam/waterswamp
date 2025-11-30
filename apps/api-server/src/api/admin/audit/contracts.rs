use domain::models::AuditLogEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuditLogFilter {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub user_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogListResponse {
    pub data: Vec<AuditLogEntry>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}
