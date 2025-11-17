use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// This service wraps the repository and provides convenient methods
// for logging audit events throughout the application.

/// Service for logging audit events.
/// Thread-safe and cloneable for use across the application.
#[derive(Clone)]
pub struct AuditService {
    pool: Arc<PgPool>,
}

impl AuditService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    pub fn from_arc(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Logs a generic audit event.
    pub async fn log_event(
        &self,
        user_id: Option<Uuid>,
        username: Option<String>,
        action: &str,
        resource: &str,
        details: Option<serde_json::Value>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let pool = self.pool.clone();
        let action = action.to_string();
        let resource = resource.to_string();

        // Spawn task to avoid blocking the request
        tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO audit_logs (
                    user_id, username, action, resource, details, ip_address, user_agent
                )
                VALUES ($1, $2, $3, $4, $5, $6::INET, $7)
                "#,
            )
            .bind(user_id)
            .bind(username)
            .bind(&action)
            .bind(&resource)
            .bind(details)
            .bind(ip_address)
            .bind(user_agent)
            .execute(pool.as_ref())
            .await
            {
                tracing::error!(error = ?e, action = %action, "Failed to write audit log");
            }
        });
    }

    /// Logs a successful login.
    pub async fn log_login_success(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "login",
            "/login",
            None,
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            user_id = %user_id,
            username = %username,
            event_type = "audit_login_success",
            "User logged in successfully"
        );
    }

    /// Logs a failed login attempt.
    pub async fn log_login_failed(
        &self,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
        reason: &str,
    ) {
        let details = serde_json::json!({
            "reason": reason,
            "attempted_username": username
        });

        self.log_event(
            None,
            Some(username.to_string()),
            "login_failed",
            "/login",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            username = %username,
            reason = %reason,
            event_type = "audit_login_failed",
            "Failed login attempt"
        );
    }

    /// Logs a logout.
    pub async fn log_logout(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "logout",
            "/logout",
            None,
            ip_address,
            user_agent,
        )
        .await;
    }

    /// Logs a token refresh.
    pub async fn log_token_refresh(
        &self,
        user_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            None,
            "token_refresh",
            "/refresh-token",
            None,
            ip_address,
            user_agent,
        )
        .await;
    }

    /// Logs a password reset request.
    pub async fn log_password_reset_request(
        &self,
        email: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "email": email
        });

        self.log_event(
            None,
            None,
            "password_reset_request",
            "/forgot-password",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;
    }

    /// Logs a successful password reset.
    pub async fn log_password_reset_success(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "password_reset",
            "/reset-password",
            None,
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            user_id = %user_id,
            event_type = "audit_password_reset",
            "Password reset completed"
        );
    }

    /// Logs a password change (by user or admin).
    pub async fn log_password_change(
        &self,
        user_id: Uuid,
        username: &str,
        changed_by: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "changed_by_admin": changed_by.is_some(),
            "changed_by_user_id": changed_by
        });

        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "password_change",
            "/api/admin/users",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            user_id = %user_id,
            event_type = "audit_password_change",
            "Password changed"
        );
    }

    /// Logs user creation.
    pub async fn log_user_created(
        &self,
        new_user_id: Uuid,
        new_username: &str,
        created_by: Uuid,
        role: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "new_user_id": new_user_id,
            "new_username": new_username,
            "created_by": created_by,
            "role": role
        });

        self.log_event(
            Some(created_by),
            None,
            "user_created",
            "/api/admin/users",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            new_user_id = %new_user_id,
            created_by = %created_by,
            event_type = "audit_user_created",
            "User created"
        );
    }

    /// Logs user update.
    pub async fn log_user_updated(
        &self,
        user_id: Uuid,
        updated_by: Uuid,
        changes: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "user_id": user_id,
            "updated_by": updated_by,
            "changes": changes
        });

        self.log_event(
            Some(updated_by),
            None,
            "user_updated",
            &format!("/api/admin/users/{}", user_id),
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            user_id = %user_id,
            updated_by = %updated_by,
            event_type = "audit_user_updated",
            "User updated"
        );
    }

    /// Logs user deletion.
    pub async fn log_user_deleted(
        &self,
        user_id: Uuid,
        deleted_by: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "deleted_user_id": user_id,
            "deleted_by": deleted_by
        });

        self.log_event(
            Some(deleted_by),
            None,
            "user_deleted",
            &format!("/api/admin/users/{}", user_id),
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            user_id = %user_id,
            deleted_by = %deleted_by,
            event_type = "audit_user_deleted",
            "User deleted"
        );
    }

    /// Logs role change.
    pub async fn log_user_role_changed(
        &self,
        user_id: Uuid,
        old_role: &str,
        new_role: &str,
        changed_by: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "user_id": user_id,
            "old_role": old_role,
            "new_role": new_role,
            "changed_by": changed_by
        });

        self.log_event(
            Some(changed_by),
            None,
            "user_role_changed",
            &format!("/api/admin/users/{}", user_id),
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            user_id = %user_id,
            old_role = %old_role,
            new_role = %new_role,
            event_type = "audit_role_change",
            "User role changed"
        );
    }

    /// Logs policy addition.
    pub async fn log_policy_added(
        &self,
        subject: &str,
        object: &str,
        action: &str,
        added_by: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "subject": subject,
            "object": object,
            "action": action,
            "added_by": added_by
        });

        self.log_event(
            Some(added_by),
            None,
            "policy_added",
            "/api/admin/policies",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            subject = %subject,
            object = %object,
            action = %action,
            event_type = "audit_policy_added",
            "Policy added"
        );
    }

    /// Logs policy removal.
    pub async fn log_policy_removed(
        &self,
        subject: &str,
        object: &str,
        action: &str,
        removed_by: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "subject": subject,
            "object": object,
            "action": action,
            "removed_by": removed_by
        });

        self.log_event(
            Some(removed_by),
            None,
            "policy_removed",
            "/api/admin/policies",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            subject = %subject,
            object = %object,
            action = %action,
            event_type = "audit_policy_removed",
            "Policy removed"
        );
    }

    /// Logs MFA enabled.
    pub async fn log_mfa_enabled(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "mfa_enabled",
            "/auth/mfa/verify-setup",
            None,
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            user_id = %user_id,
            event_type = "audit_mfa_enabled",
            "MFA enabled for user"
        );
    }

    /// Logs MFA disabled.
    pub async fn log_mfa_disabled(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "mfa_disabled",
            "/auth/mfa/disable",
            None,
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            user_id = %user_id,
            event_type = "audit_mfa_disabled",
            "MFA disabled for user"
        );
    }

    /// Logs MFA verification success.
    pub async fn log_mfa_verified(
        &self,
        user_id: Uuid,
        backup_code_used: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "backup_code_used": backup_code_used
        });

        self.log_event(
            Some(user_id),
            None,
            "mfa_verified",
            "/auth/mfa/verify",
            Some(details),
            ip_address,
            user_agent,
        )
        .await;
    }

    /// Logs MFA verification failure.
    pub async fn log_mfa_failed(
        &self,
        user_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            None,
            "mfa_failed",
            "/auth/mfa/verify",
            None,
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            user_id = %user_id,
            event_type = "audit_mfa_failed",
            "MFA verification failed"
        );
    }

    /// Logs email verification.
    pub async fn log_email_verified(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        self.log_event(
            Some(user_id),
            Some(username.to_string()),
            "email_verified",
            "/verify-email",
            None,
            ip_address,
            user_agent,
        )
        .await;

        tracing::info!(
            user_id = %user_id,
            event_type = "audit_email_verified",
            "Email verified"
        );
    }

    /// Logs access denied (403).
    pub async fn log_access_denied(
        &self,
        user_id: Uuid,
        resource: &str,
        method: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let details = serde_json::json!({
            "method": method
        });

        self.log_event(
            Some(user_id),
            None,
            "admin_access_denied",
            resource,
            Some(details),
            ip_address,
            user_agent,
        )
        .await;

        tracing::warn!(
            user_id = %user_id,
            resource = %resource,
            event_type = "audit_access_denied",
            "Access denied to resource"
        );
    }
}

/// Helper trait for extracting client info from request
pub trait RequestInfoExtractor {
    fn get_ip_address(&self) -> Option<String>;
    fn get_user_agent(&self) -> Option<String>;
}

impl RequestInfoExtractor for axum::http::HeaderMap {
    fn get_ip_address(&self) -> Option<String> {
        // Try various headers in order of preference
        if let Some(forwarded) = self.get("x-forwarded-for") {
            if let Ok(s) = forwarded.to_str() {
                // Take first IP if multiple
                return Some(s.split(',').next().unwrap_or(s).trim().to_string());
            }
        }

        if let Some(real_ip) = self.get("x-real-ip") {
            if let Ok(s) = real_ip.to_str() {
                return Some(s.to_string());
            }
        }

        None
    }

    fn get_user_agent(&self) -> Option<String> {
        self.get(axum::http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}
