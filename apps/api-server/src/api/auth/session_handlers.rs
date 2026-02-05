//! Session-Based Authentication Handlers
//!
//! Cookie-based authentication with HttpOnly cookies and CSRF protection.
//! These handlers complement the existing JWT-based authentication.

use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use core_services::{
    security::verify_password,
    session::{
        build_csrf_cookie, build_removal_cookie, build_session_cookie, config, encryption,
        generate_csrf_token, generate_session_token, hash_token,
    },
};
use domain::models::{CreateSession, SessionRevocationReason};
use domain::ports::{SessionRepositoryPort, UserRepositoryPort};
use persistence::repositories::{
    session_repository::SessionRepository, user_repository::UserRepository,
};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::infra::{errors::AppError, state::AppState};

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

/// Login request for session-based authentication
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SessionLoginRequest {
    /// Username or email
    #[validate(length(min = 1, max = 100))]
    pub username: String,

    /// Password
    #[validate(length(min = 1, max = 100))]
    pub password: String,

    /// Remember me flag (extends session duration)
    #[serde(default)]
    pub remember_me: bool,
}

/// Response for successful session login
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionLoginResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Uuid,
    pub username: String,
    /// CSRF token (also set in cookie, but returned here for SPA convenience)
    pub csrf_token: String,
    /// Session expiration timestamp
    pub expires_at: i64,
}

/// Response for session info
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionInfoResponse {
    pub user_id: Uuid,
    pub username: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_activity_at: i64,
}

/// Response for logout
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionLogoutResponse {
    pub success: bool,
    pub message: String,
}

/// User's active sessions list
#[derive(Debug, Serialize, ToSchema)]
pub struct UserSessionsResponse {
    pub sessions: Vec<SessionSummaryDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionSummaryDto {
    pub id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: i64,
    pub last_activity_at: i64,
    pub is_current: bool,
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Extracts client IP from headers (X-Forwarded-For, X-Real-IP, or direct)
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (may have multiple IPs)
    if let Some(xff) = headers.get("X-Forwarded-For") {
        if let Ok(xff_str) = xff.to_str() {
            // Take the first IP (original client)
            if let Some(first_ip) = xff_str.split(',').next() {
                let ip_str = first_ip.trim();
                // Validate it looks like an IP
                if ip_str.parse::<std::net::IpAddr>().is_ok() {
                    return Some(ip_str.to_string());
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            // Validate it looks like an IP
            if ip_str.parse::<std::net::IpAddr>().is_ok() {
                return Some(ip_str.to_string());
            }
        }
    }

    None
}

/// Extracts User-Agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("User-Agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

/// Gets encryption key from state or generates one if not present
async fn get_encryption_key(state: &AppState) -> Result<Vec<u8>, AppError> {
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());

    // Try to get active encryption key
    if let Some(key) = session_repo.get_active_encryption_key().await? {
        return Ok(key.key_material);
    }

    // No key exists, create one
    let key_id = core_services::session::generate_key_id();
    let key_material = core_services::session::generate_token(32);

    let key = session_repo
        .create_session_key(&key_id, &key_material, "encryption")
        .await?;

    Ok(key.key_material)
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /api/v1/auth/session/login
///
/// Session-based login with HttpOnly cookies.
/// Sets session cookie (HttpOnly, Secure, SameSite=Strict) and CSRF cookie.
#[utoipa::path(
    post,
    path = "/api/v1/auth/session/login",
    tag = "Session Auth",
    request_body = SessionLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = SessionLoginResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn session_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SessionLoginRequest>,
) -> Result<Response, AppError> {
    // 1. Validate payload
    payload.validate().map_err(|e| {
        warn!(validation_errors = ?e, "Session login validation failed");
        AppError::Validation(e)
    })?;

    // 2. Find user
    let user_repo = UserRepository::new(state.db_pool_auth.clone());
    let user_info = user_repo
        .find_for_login(&payload.username)
        .await?
        .ok_or(AppError::InvalidPassword)?;

    let user_id = user_info.id;
    let username = user_info.username.clone();
    let password_hash = user_info.password_hash.clone();

    // 3. Verify password
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&payload.password, &password_hash))
            .await
            .map_err(|e| AppError::Internal(format!("Password verification task failed: {}", e)))?
            .map_err(|_| AppError::InvalidPassword)?;

    if !password_valid {
        warn!(username = %payload.username, "Invalid password attempt");
        return Err(AppError::InvalidPassword);
    }

    // 4. Check MFA (for now, return error if MFA is enabled - should use JWT flow)
    if user_info.mfa_enabled {
        return Err(AppError::BadRequest(
            "MFA is enabled. Please use the JWT login endpoint.".to_string(),
        ));
    }

    // 5. Generate session tokens
    let session_token = generate_session_token();
    let session_token_hash = hash_token(&session_token);
    let csrf_token = generate_csrf_token();
    let csrf_token_hash = hash_token(&csrf_token);

    // 6. Generate access token (JWT) for internal API calls
    let access_token = state
        .jwt_service
        .generate_token(user_id, &username, domain::models::TokenType::Access, 3600)
        .map_err(|e| {
            error!("Failed to generate access token: {:?}", e);
            AppError::Internal("Failed to generate access token".to_string())
        })?;

    // 7. Encrypt access token for storage
    let encryption_key = get_encryption_key(&state).await?;
    let access_token_encrypted = encryption::encrypt(&access_token, &encryption_key)
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

    // 8. Calculate session duration
    let session_duration = if payload.remember_me {
        config::EXTENDED_SESSION_DURATION
    } else {
        config::SESSION_DURATION
    };
    let expires_at = Utc::now() + Duration::from_std(session_duration).unwrap();

    // 9. Create session in database
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());
    let create_session = CreateSession {
        session_token_hash: session_token_hash.clone(),
        user_id,
        user_agent: extract_user_agent(&headers),
        ip_address: extract_client_ip(&headers),
        access_token_encrypted,
        refresh_token_id: None,
        expires_at,
        csrf_token_hash,
    };

    let session = session_repo.create_session(create_session).await?;

    // 10. Build cookies
    let session_cookie = build_session_cookie(&session_token, session_duration);
    let csrf_cookie = build_csrf_cookie(&csrf_token, session_duration);

    info!(user_id = %user_id, session_id = %session.id, "Session login successful");

    // 11. Build response with Set-Cookie headers
    let response_body = SessionLoginResponse {
        success: true,
        message: "Login successful".to_string(),
        user_id,
        username: username.clone(),
        csrf_token: csrf_token.clone(),
        expires_at: expires_at.timestamp(),
    };

    let mut response = Json(response_body).into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        session_cookie.to_string().parse().unwrap(),
    );
    response.headers_mut().append(
        SET_COOKIE,
        csrf_cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// POST /api/v1/auth/session/logout
///
/// Logs out the current session by revoking the session and clearing cookies.
#[utoipa::path(
    post,
    path = "/api/v1/auth/session/logout",
    tag = "Session Auth",
    responses(
        (status = 200, description = "Logout successful", body = SessionLogoutResponse),
        (status = 401, description = "No valid session")
    )
)]
pub async fn session_logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    // 1. Get session token from cookie
    let session_token = cookies
        .get(config::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("No session cookie".to_string()))?;

    let session_token_hash = hash_token(&session_token);

    // 2. Revoke session in database
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());
    let revoked = session_repo
        .revoke_session_by_token(&session_token_hash, SessionRevocationReason::UserLogout)
        .await?;

    if !revoked {
        warn!("Attempted logout with invalid/expired session");
    }

    // 3. Build removal cookies
    let session_removal = build_removal_cookie(config::SESSION_COOKIE_NAME);
    let csrf_removal = build_removal_cookie(config::CSRF_COOKIE_NAME);

    info!("Session logout successful");

    // 4. Build response
    let response_body = SessionLogoutResponse {
        success: true,
        message: "Logout successful".to_string(),
    };

    let mut response = Json(response_body).into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        session_removal.to_string().parse().unwrap(),
    );
    response.headers_mut().append(
        SET_COOKIE,
        csrf_removal.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// POST /api/v1/auth/session/logout-all
///
/// Logs out all sessions for the current user.
#[utoipa::path(
    post,
    path = "/api/v1/auth/session/logout-all",
    tag = "Session Auth",
    responses(
        (status = 200, description = "All sessions logged out"),
        (status = 401, description = "No valid session")
    )
)]
pub async fn session_logout_all(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    // 1. Get and validate current session
    let session_token = cookies
        .get(config::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("No session cookie".to_string()))?;

    let session_token_hash = hash_token(&session_token);
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());

    let session = session_repo
        .find_by_token_hash(&session_token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid session".to_string()))?;

    // 2. Revoke all user sessions
    let revoked_count = session_repo
        .revoke_all_user_sessions(session.user_id, SessionRevocationReason::UserLogoutAll)
        .await?;

    // 3. Clear cookies
    let session_removal = build_removal_cookie(config::SESSION_COOKIE_NAME);
    let csrf_removal = build_removal_cookie(config::CSRF_COOKIE_NAME);

    info!(user_id = %session.user_id, count = revoked_count, "All sessions logged out");

    let response_body = serde_json::json!({
        "success": true,
        "message": format!("Logged out of {} session(s)", revoked_count),
        "revoked_count": revoked_count
    });

    let mut response = Json(response_body).into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        session_removal.to_string().parse().unwrap(),
    );
    response.headers_mut().append(
        SET_COOKIE,
        csrf_removal.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// GET /api/v1/auth/session/me
///
/// Returns information about the current session.
#[utoipa::path(
    get,
    path = "/api/v1/auth/session/me",
    tag = "Session Auth",
    responses(
        (status = 200, description = "Session info", body = SessionInfoResponse),
        (status = 401, description = "No valid session")
    )
)]
pub async fn session_info(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Json<SessionInfoResponse>, AppError> {
    // 1. Get session from cookie
    let session_token = cookies
        .get(config::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("No session cookie".to_string()))?;

    let session_token_hash = hash_token(&session_token);
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());

    let session = session_repo
        .find_by_token_hash(&session_token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".to_string()))?;

    // 2. Get username
    let user_repo = UserRepository::new(state.db_pool_auth.clone());
    let user = user_repo
        .find_by_id(session.user_id)
        .await?
        .ok_or_else(|| AppError::Internal("User not found".to_string()))?;

    // 3. Touch session (extend expiry)
    session_repo
        .touch_session(session.id, Some(config::SLIDING_WINDOW_MINUTES))
        .await?;

    Ok(Json(SessionInfoResponse {
        user_id: session.user_id,
        username: user.username.to_string(),
        created_at: session.created_at.timestamp(),
        expires_at: session.expires_at.timestamp(),
        last_activity_at: session.last_activity_at.timestamp(),
    }))
}

/// GET /api/v1/auth/session/list
///
/// Lists all active sessions for the current user.
#[utoipa::path(
    get,
    path = "/api/v1/auth/session/list",
    tag = "Session Auth",
    responses(
        (status = 200, description = "List of sessions", body = UserSessionsResponse),
        (status = 401, description = "No valid session")
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Json<UserSessionsResponse>, AppError> {
    // 1. Get current session
    let session_token = cookies
        .get(config::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("No session cookie".to_string()))?;

    let session_token_hash = hash_token(&session_token);
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());

    let current_session = session_repo
        .find_by_token_hash(&session_token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid session".to_string()))?;

    // 2. List all sessions
    let sessions = session_repo
        .list_user_sessions(current_session.user_id, Some(current_session.id))
        .await?;

    let session_dtos: Vec<SessionSummaryDto> = sessions
        .into_iter()
        .map(|s| SessionSummaryDto {
            id: s.id,
            user_agent: s.user_agent,
            ip_address: s.ip_address,
            created_at: s.created_at.timestamp(),
            last_activity_at: s.last_activity_at.timestamp(),
            is_current: s.is_current,
        })
        .collect();

    Ok(Json(UserSessionsResponse {
        sessions: session_dtos,
    }))
}

/// DELETE /api/v1/auth/session/{session_id}
///
/// Revokes a specific session (must be owned by current user).
#[utoipa::path(
    delete,
    path = "/api/v1/auth/session/{session_id}",
    tag = "Session Auth",
    params(
        ("session_id" = Uuid, Path, description = "Session ID to revoke")
    ),
    responses(
        (status = 204, description = "Session revoked"),
        (status = 401, description = "No valid session"),
        (status = 404, description = "Session not found")
    )
)]
pub async fn revoke_session(
    State(state): State<AppState>,
    cookies: Cookies,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // 1. Verify current session
    let session_token = cookies
        .get(config::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("No session cookie".to_string()))?;

    let session_token_hash = hash_token(&session_token);
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());

    let current_session = session_repo
        .find_by_token_hash(&session_token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid session".to_string()))?;

    // 2. Check if target session belongs to the same user
    // For security, we can only revoke our own sessions
    // (Admin route would be separate)
    let sessions = session_repo
        .list_user_sessions(current_session.user_id, None)
        .await?;

    let target_exists = sessions.iter().any(|s| s.id == session_id);
    if !target_exists {
        return Err(AppError::NotFound("Session not found".to_string()));
    }

    // 3. Revoke the session
    session_repo
        .revoke_session(session_id, SessionRevocationReason::UserLogout)
        .await?;

    info!(user_id = %current_session.user_id, session_id = %session_id, "Session revoked");

    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_client_ip_xff() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", "10.20.30.40".parse().unwrap());

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("10.20.30.40".to_string()));
    }

    #[test]
    fn test_extract_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", "Mozilla/5.0 Test".parse().unwrap());

        let ua = extract_user_agent(&headers);
        assert_eq!(ua, Some("Mozilla/5.0 Test".to_string()));
    }
}
