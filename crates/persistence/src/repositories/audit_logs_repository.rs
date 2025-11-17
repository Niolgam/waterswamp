use sqlx::PgPool;
use uuid::Uuid;

// Import from domain models (add to domain/src/models.rs)
// use domain::models::{AuditLogEntry, CreateAuditLogDto, ListAuditLogsQuery, PaginatedAuditLogs, AuditLogStats, ActionCount, ResourceCount};

/// Repository for audit log operations.
/// Note: This connects to db_pool_logs, NOT db_pool_auth
pub struct AuditLogRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AuditLogRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new audit log entry.
    pub async fn create(
        &self,
        user_id: Option<Uuid>,
        username: Option<&str>,
        action: &str,
        resource: &str,
        method: Option<&str>,
        status_code: Option<i32>,
        details: Option<serde_json::Value>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        duration_ms: Option<i32>,
    ) -> Result<Uuid, sqlx::Error> {
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO audit_logs (
                user_id, username, action, resource, method, 
                status_code, details, ip_address, user_agent, 
                request_id, duration_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::INET, $9, $10, $11)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(action)
        .bind(resource)
        .bind(method)
        .bind(status_code)
        .bind(details)
        .bind(ip_address)
        .bind(user_agent)
        .bind(request_id)
        .bind(duration_ms)
        .fetch_one(self.pool)
        .await?;

        Ok(id)
    }

    /// Finds an audit log entry by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntryRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogEntryRow>(
            r#"
            SELECT 
                id, user_id, username, action, resource, method,
                status_code, details, ip_address::TEXT as ip_address, 
                user_agent, request_id, duration_ms, created_at
            FROM audit_logs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
    }

    /// Lists audit logs with filtering, pagination, and sorting.
    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
        user_id: Option<Uuid>,
        action: Option<&str>,
        resource: Option<&str>,
        ip_address: Option<&str>,
        status_code: Option<i32>,
        min_status_code: Option<i32>,
        max_status_code: Option<i32>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<(Vec<AuditLogEntryRow>, i64), sqlx::Error> {
        // Build WHERE clauses dynamically
        let mut conditions = Vec::new();
        let mut param_count = 0;

        if user_id.is_some() {
            param_count += 1;
            conditions.push(format!("user_id = ${}", param_count));
        }

        if action.is_some() {
            param_count += 1;
            conditions.push(format!("action = ${}", param_count));
        }

        if resource.is_some() {
            param_count += 1;
            conditions.push(format!("resource ILIKE ${}", param_count));
        }

        if ip_address.is_some() {
            param_count += 1;
            conditions.push(format!("ip_address::TEXT = ${}", param_count));
        }

        if status_code.is_some() {
            param_count += 1;
            conditions.push(format!("status_code = ${}", param_count));
        }

        if min_status_code.is_some() {
            param_count += 1;
            conditions.push(format!("status_code >= ${}", param_count));
        }

        if max_status_code.is_some() {
            param_count += 1;
            conditions.push(format!("status_code <= ${}", param_count));
        }

        if start_date.is_some() {
            param_count += 1;
            conditions.push(format!("created_at >= ${}", param_count));
        }

        if end_date.is_some() {
            param_count += 1;
            conditions.push(format!("created_at <= ${}", param_count));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Validate sort parameters
        let valid_sort_by = match sort_by {
            "created_at" | "action" | "user_id" | "resource" | "status_code" => sort_by,
            _ => "created_at",
        };

        let valid_sort_order = match sort_order.to_uppercase().as_str() {
            "ASC" => "ASC",
            _ => "DESC",
        };

        // Build query for data
        let query_str = format!(
            r#"
            SELECT 
                id, user_id, username, action, resource, method,
                status_code, details, ip_address::TEXT as ip_address, 
                user_agent, request_id, duration_ms, created_at
            FROM audit_logs
            {}
            ORDER BY {} {}
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            valid_sort_by,
            valid_sort_order,
            param_count + 1,
            param_count + 2
        );

        // Build query for count
        let count_query_str = format!(
            r#"
            SELECT COUNT(*)
            FROM audit_logs
            {}
            "#,
            where_clause
        );

        // Execute queries with dynamic binding
        // Due to SQLx limitations, we need to build queries differently
        // Here's a simplified approach using raw queries

        let logs = self
            .execute_list_query(
                &query_str,
                user_id,
                action,
                resource,
                ip_address,
                status_code,
                min_status_code,
                max_status_code,
                start_date,
                end_date,
                limit,
                offset,
            )
            .await?;

        let total = self
            .execute_count_query(
                &count_query_str,
                user_id,
                action,
                resource,
                ip_address,
                status_code,
                min_status_code,
                max_status_code,
                start_date,
                end_date,
            )
            .await?;

        Ok((logs, total))
    }

    // Helper to execute the list query with all bindings
    async fn execute_list_query(
        &self,
        query_str: &str,
        user_id: Option<Uuid>,
        action: Option<&str>,
        resource: Option<&str>,
        ip_address: Option<&str>,
        status_code: Option<i32>,
        min_status_code: Option<i32>,
        max_status_code: Option<i32>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntryRow>, sqlx::Error> {
        let mut query = sqlx::query_as::<_, AuditLogEntryRow>(query_str);

        if let Some(uid) = user_id {
            query = query.bind(uid);
        }
        if let Some(act) = action {
            query = query.bind(act);
        }
        if let Some(res) = resource {
            query = query.bind(format!("%{}%", res));
        }
        if let Some(ip) = ip_address {
            query = query.bind(ip);
        }
        if let Some(sc) = status_code {
            query = query.bind(sc);
        }
        if let Some(min_sc) = min_status_code {
            query = query.bind(min_sc);
        }
        if let Some(max_sc) = max_status_code {
            query = query.bind(max_sc);
        }
        if let Some(sd) = start_date {
            query = query.bind(sd);
        }
        if let Some(ed) = end_date {
            query = query.bind(ed);
        }

        query = query.bind(limit).bind(offset);

        query.fetch_all(self.pool).await
    }

    // Helper to execute the count query with all bindings
    async fn execute_count_query(
        &self,
        query_str: &str,
        user_id: Option<Uuid>,
        action: Option<&str>,
        resource: Option<&str>,
        ip_address: Option<&str>,
        status_code: Option<i32>,
        min_status_code: Option<i32>,
        max_status_code: Option<i32>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = sqlx::query_scalar::<_, i64>(query_str);

        if let Some(uid) = user_id {
            query = query.bind(uid);
        }
        if let Some(act) = action {
            query = query.bind(act);
        }
        if let Some(res) = resource {
            query = query.bind(format!("%{}%", res));
        }
        if let Some(ip) = ip_address {
            query = query.bind(ip);
        }
        if let Some(sc) = status_code {
            query = query.bind(sc);
        }
        if let Some(min_sc) = min_status_code {
            query = query.bind(min_sc);
        }
        if let Some(max_sc) = max_status_code {
            query = query.bind(max_sc);
        }
        if let Some(sd) = start_date {
            query = query.bind(sd);
        }
        if let Some(ed) = end_date {
            query = query.bind(ed);
        }

        query.fetch_one(self.pool).await
    }

    /// Gets audit log statistics.
    pub async fn get_stats(&self) -> Result<AuditLogStatsRow, sqlx::Error> {
        let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(self.pool)
            .await?;

        let logs_today: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE created_at >= CURRENT_DATE")
                .fetch_one(self.pool)
                .await?;

        let logs_this_week: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'",
        )
        .fetch_one(self.pool)
        .await?;

        let failed_logins_today: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM audit_logs 
            WHERE action = 'login_failed' 
            AND created_at >= CURRENT_DATE
            "#,
        )
        .fetch_one(self.pool)
        .await?;

        let unique_users_today: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT user_id) FROM audit_logs 
            WHERE user_id IS NOT NULL 
            AND created_at >= CURRENT_DATE
            "#,
        )
        .fetch_one(self.pool)
        .await?;

        let top_actions: Vec<ActionCountRow> = sqlx::query_as(
            r#"
            SELECT action, COUNT(*) as count
            FROM audit_logs
            WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
            GROUP BY action
            ORDER BY count DESC
            LIMIT 10
            "#,
        )
        .fetch_all(self.pool)
        .await?;

        let top_resources: Vec<ResourceCountRow> = sqlx::query_as(
            r#"
            SELECT resource, COUNT(*) as count
            FROM audit_logs
            WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
            GROUP BY resource
            ORDER BY count DESC
            LIMIT 10
            "#,
        )
        .fetch_all(self.pool)
        .await?;

        Ok(AuditLogStatsRow {
            total_logs,
            logs_today,
            logs_this_week,
            failed_logins_today,
            unique_users_today,
            top_actions,
            top_resources,
        })
    }

    /// Gets logs for a specific user.
    pub async fn get_user_logs(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLogEntryRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogEntryRow>(
            r#"
            SELECT 
                id, user_id, username, action, resource, method,
                status_code, details, ip_address::TEXT as ip_address, 
                user_agent, request_id, duration_ms, created_at
            FROM audit_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    /// Gets failed login attempts for security monitoring.
    pub async fn get_failed_logins(
        &self,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<AuditLogEntryRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogEntryRow>(
            r#"
            SELECT 
                id, user_id, username, action, resource, method,
                status_code, details, ip_address::TEXT as ip_address, 
                user_agent, request_id, duration_ms, created_at
            FROM audit_logs
            WHERE action = 'login_failed'
            AND created_at >= NOW() - ($1 || ' hours')::INTERVAL
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(hours.to_string())
        .bind(limit)
        .fetch_all(self.pool)
        .await
    }

    /// Gets suspicious activity (multiple failed logins from same IP).
    pub async fn get_suspicious_ips(
        &self,
        hours: i64,
        threshold: i64,
    ) -> Result<Vec<SuspiciousIpRow>, sqlx::Error> {
        sqlx::query_as::<_, SuspiciousIpRow>(
            r#"
            SELECT 
                ip_address::TEXT as ip_address,
                COUNT(*) as failed_attempts,
                COUNT(DISTINCT username) as unique_usernames,
                MIN(created_at) as first_attempt,
                MAX(created_at) as last_attempt
            FROM audit_logs
            WHERE action = 'login_failed'
            AND ip_address IS NOT NULL
            AND created_at >= NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY ip_address
            HAVING COUNT(*) >= $2
            ORDER BY failed_attempts DESC
            "#,
        )
        .bind(hours.to_string())
        .bind(threshold)
        .fetch_all(self.pool)
        .await
    }

    /// Cleans up old audit logs based on retention policy.
    pub async fn cleanup_old_logs(&self, retention_days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs 
            WHERE created_at < NOW() - ($1 || ' days')::INTERVAL
            "#,
        )
        .bind(retention_days.to_string())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

// =============================================================================
// ROW TYPES (for sqlx::FromRow)
// =============================================================================

#[derive(Debug, sqlx::FromRow)]
pub struct AuditLogEntryRow {
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

#[derive(Debug, sqlx::FromRow)]
pub struct ActionCountRow {
    pub action: String,
    pub count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ResourceCountRow {
    pub resource: String,
    pub count: i64,
}

#[derive(Debug)]
pub struct AuditLogStatsRow {
    pub total_logs: i64,
    pub logs_today: i64,
    pub logs_this_week: i64,
    pub failed_logins_today: i64,
    pub unique_users_today: i64,
    pub top_actions: Vec<ActionCountRow>,
    pub top_resources: Vec<ResourceCountRow>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SuspiciousIpRow {
    pub ip_address: Option<String>,
    pub failed_attempts: i64,
    pub unique_usernames: i64,
    pub first_attempt: chrono::DateTime<chrono::Utc>,
    pub last_attempt: chrono::DateTime<chrono::Utc>,
}
