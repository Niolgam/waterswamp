use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

use crate::{state::AppState, web_models::CurrentUser};

/// Middleware that automatically logs audit events for all requests.
/// This captures request metadata and response status for comprehensive auditing.
pub async fn mw_audit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4();

    // Extract request info before consuming the request
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers = req.headers().clone();

    // Extract user info if authenticated
    let user_info = req
        .extensions()
        .get::<CurrentUser>()
        .map(|u| (u.id, u.username.clone()));

    let ip_address = extract_ip_address(&headers);
    let user_agent = extract_user_agent(&headers);

    // Execute the request
    let response = next.run(req).await;

    // Calculate duration
    let duration_ms = start_time.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i32;

    // Determine action based on path and method
    let action = determine_action(&method, &path, status_code);

    // Only log significant actions (not every GET request)
    if should_log_action(&action, &method, &path, status_code) {
        let pool = state.db_pool_logs.clone();

        let (user_id, username) = user_info.unzip();

        // Spawn task to avoid blocking response
        tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO audit_logs (
                    user_id, username, action, resource, method,
                    status_code, ip_address, user_agent, request_id, duration_ms
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7::INET, $8, $9, $10)
                "#,
            )
            .bind(user_id)
            .bind(username)
            .bind(&action)
            .bind(&path)
            .bind(&method)
            .bind(status_code)
            .bind(ip_address)
            .bind(user_agent)
            .bind(request_id)
            .bind(duration_ms)
            .execute(&pool)
            .await
            {
                tracing::error!(
                    error = ?e,
                    action = %action,
                    path = %path,
                    "Failed to write audit log from middleware"
                );
            }
        });
    }

    response
}

/// Determines the audit action based on request characteristics
fn determine_action(method: &str, path: &str, status_code: i32) -> String {
    // Handle failed requests
    if status_code == 401 {
        if path.contains("/login") {
            return "login_failed".to_string();
        }
        return "unauthorized_access".to_string();
    }

    if status_code == 403 {
        return "admin_access_denied".to_string();
    }

    // Handle specific paths
    match (method, path) {
        ("POST", "/login") => "login".to_string(),
        ("POST", "/register") => "user_registered".to_string(),
        ("POST", "/logout") => "logout".to_string(),
        ("POST", "/refresh-token") => "token_refresh".to_string(),
        ("POST", "/forgot-password") => "password_reset_request".to_string(),
        ("POST", "/reset-password") => "password_reset".to_string(),
        ("POST", "/verify-email") => "email_verified".to_string(),
        ("POST", "/resend-verification") => "email_verification_sent".to_string(),

        // MFA routes
        ("POST", p) if p.contains("/mfa/setup") => "mfa_setup_initiated".to_string(),
        ("POST", p) if p.contains("/mfa/verify-setup") => "mfa_enabled".to_string(),
        ("POST", p) if p.contains("/mfa/verify") => "mfa_verified".to_string(),
        ("POST", p) if p.contains("/mfa/disable") => "mfa_disabled".to_string(),
        ("POST", p) if p.contains("/mfa/regenerate") => "mfa_backup_codes_regenerated".to_string(),

        // Admin user management
        ("POST", "/api/admin/users") => "user_created".to_string(),
        ("PUT", p) if p.starts_with("/api/admin/users/") => "user_updated".to_string(),
        ("DELETE", p) if p.starts_with("/api/admin/users/") => "user_deleted".to_string(),

        // Admin policy management
        ("POST", "/api/admin/policies") => "policy_added".to_string(),
        ("DELETE", "/api/admin/policies") => "policy_removed".to_string(),

        // Default: resource access
        _ => "resource_access".to_string(),
    }
}

/// Determines if an action should be logged
fn should_log_action(action: &str, method: &str, path: &str, status_code: i32) -> bool {
    // Always log security-sensitive actions
    let security_actions = [
        "login",
        "login_failed",
        "logout",
        "token_refresh",
        "password_reset_request",
        "password_reset",
        "user_registered",
        "email_verified",
        "mfa_setup_initiated",
        "mfa_enabled",
        "mfa_disabled",
        "mfa_verified",
        "mfa_backup_codes_regenerated",
        "user_created",
        "user_updated",
        "user_deleted",
        "policy_added",
        "policy_removed",
        "unauthorized_access",
        "admin_access_denied",
    ];

    if security_actions.contains(&action) {
        return true;
    }

    // Log all errors (4xx and 5xx)
    if status_code >= 400 {
        return true;
    }

    // Log all admin routes
    if path.starts_with("/api/admin") {
        return true;
    }

    // Log all write operations (POST, PUT, DELETE, PATCH)
    if matches!(method, "POST" | "PUT" | "DELETE" | "PATCH") {
        return true;
    }

    // Don't log routine GET requests to avoid noise
    if method == "GET" {
        // Skip health checks, metrics, static assets
        if path == "/health"
            || path == "/health/live"
            || path == "/health/ready"
            || path == "/metrics"
            || path.starts_with("/static")
            || path.starts_with("/assets")
        {
            return false;
        }
    }

    // Default: log everything else
    true
}

/// Extracts client IP address from headers
fn extract_ip_address(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (common in reverse proxy setups)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(s) = forwarded.to_str() {
            // Take first IP if multiple are present
            return Some(s.split(',').next().unwrap_or(s).trim().to_string());
        }
    }

    // Try X-Real-IP (nginx)
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(s) = real_ip.to_str() {
            return Some(s.to_string());
        }
    }

    // Try CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("cf-connecting-ip") {
        if let Ok(s) = cf_ip.to_str() {
            return Some(s.to_string());
        }
    }

    None
}

/// Extracts user agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            // Truncate very long user agents
            if s.len() > 512 {
                format!("{}...", &s[..509])
            } else {
                s.to_string()
            }
        })
}

/// More selective middleware that only logs specific routes
/// Use this if the full middleware is too verbose
pub async fn selective_audit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();

    // Only audit specific paths
    let should_audit = path.contains("/login")
        || path.contains("/logout")
        || path.contains("/register")
        || path.contains("/reset-password")
        || path.contains("/forgot-password")
        || path.contains("/verify-email")
        || path.contains("/mfa")
        || path.starts_with("/api/admin");

    if should_audit {
        mw_audit(State(state), req, next).await
    } else {
        next.run(req).await
    }
}
