//! User self-service handlers
//!
//! Handlers for user profile management and password changes.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use validator::Validate;

use domain::{
    models::{User, UserRole},
    repositories::{AuthRepository, UserRepository},
};

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

// ============================================================================
// DTOs (Data Transfer Objects)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: chrono::NaiveDateTime,
}

impl From<User> for ProfileResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            email_verified: user.email_verified,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "New password must be between 8 and 128 characters"
    ))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/v1/users/profile
///
/// Returns the authenticated user's profile information.
#[instrument(skip(current_user))]
pub async fn get_profile(
    CurrentUser(user): CurrentUser,
) -> Result<Json<ProfileResponse>, AppError> {
    info!(user_id = user.id, username = %user.username, "Fetching user profile");

    Ok(Json(ProfileResponse::from(user)))
}

/// PUT /api/v1/users/profile
///
/// Updates the authenticated user's profile (username and/or email).
///
/// **Note:** If email is changed, it will require re-verification.
#[instrument(skip(state, current_user, payload))]
pub async fn update_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    // Validate payload
    payload.validate().map_err(|e| {
        error!(user_id = user.id, validation_errors = ?e, "Profile update validation failed");
        AppError::ValidationError(e.to_string())
    })?;

    // Check if there's anything to update
    if payload.username.is_none() && payload.email.is_none() {
        return Err(AppError::ValidationError(
            "At least one field (username or email) must be provided".to_string(),
        ));
    }

    let user_id = user.id;
    info!(
        user_id,
        new_username = ?payload.username,
        new_email = ?payload.email,
        "Updating user profile"
    );

    // Check if username is taken (if being changed)
    if let Some(ref new_username) = payload.username {
        if new_username != &user.username {
            if state
                .user_repo
                .find_by_username(new_username)
                .await?
                .is_some()
            {
                error!(user_id, new_username, "Username already taken");
                return Err(AppError::Conflict("Username already taken".to_string()));
            }
        }
    }

    // Check if email is taken (if being changed)
    let email_changed = if let Some(ref new_email) = payload.email {
        if new_email != &user.email {
            if state.user_repo.find_by_email(new_email).await?.is_some() {
                error!(user_id, new_email, "Email already taken");
                return Err(AppError::Conflict("Email already taken".to_string()));
            }
            true
        } else {
            false
        }
    } else {
        false
    };

    // Update username if provided
    if let Some(new_username) = payload.username {
        if new_username != user.username {
            state
                .user_repo
                .update_username(user_id, &new_username)
                .await?;
            info!(user_id, new_username, "Username updated successfully");
        }
    }

    // Update email if provided
    if let Some(new_email) = payload.email {
        if new_email != user.email {
            state.user_repo.update_email(user_id, &new_email).await?;

            // Mark email as unverified since it changed
            state.user_repo.mark_email_unverified(user_id).await?;

            info!(
                user_id,
                new_email, "Email updated successfully (requires re-verification)"
            );
        }
    }

    // Fetch updated user
    let updated_user = state.user_repo.find_by_id(user_id).await?.ok_or_else(|| {
        error!(user_id, "User not found after update");
        AppError::NotFound("User not found".to_string())
    })?;

    info!(user_id, email_changed, "Profile updated successfully");

    Ok(Json(ProfileResponse::from(updated_user)))
}

/// PUT /api/v1/users/password
///
/// Changes the authenticated user's password.
///
/// Requires the current password for security verification.
/// After changing password, all refresh tokens are revoked.
#[instrument(skip(state, current_user, payload))]
pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<(StatusCode, Json<ChangePasswordResponse>), AppError> {
    // Validate payload
    payload.validate().map_err(|e| {
        error!(user_id = user.id, validation_errors = ?e, "Password change validation failed");
        AppError::ValidationError(e.to_string())
    })?;

    let user_id = user.id;
    info!(user_id, "Attempting to change password");

    // Verify current password
    let password_valid = state
        .user_repo
        .verify_password(user_id, &payload.current_password)
        .await?;

    if !password_valid {
        error!(user_id, "Current password verification failed");
        return Err(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }

    // Check if new password is different from current
    let new_password_same_as_current = state
        .user_repo
        .verify_password(user_id, &payload.new_password)
        .await?;

    if new_password_same_as_current {
        error!(user_id, "New password is the same as current password");
        return Err(AppError::ValidationError(
            "New password must be different from current password".to_string(),
        ));
    }

    // Update password
    state
        .user_repo
        .update_password(user_id, &payload.new_password)
        .await?;

    // Revoke all refresh tokens for security
    state.auth_repo.revoke_all_user_tokens(user_id).await?;

    info!(
        user_id,
        "Password changed successfully, all refresh tokens revoked"
    );

    Ok((
        StatusCode::OK,
        Json(ChangePasswordResponse {
            message: "Password changed successfully. Please log in again with your new password."
                .to_string(),
        }),
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_response_from_user() {
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: UserRole::User,
            email_verified: true,
            mfa_enabled: false,
            mfa_secret: None,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        };

        let response = ProfileResponse::from(user);

        assert_eq!(response.id, 1);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role, UserRole::User);
        assert!(response.email_verified);
        assert!(!response.mfa_enabled);
    }

    #[test]
    fn test_update_profile_request_validation() {
        // Valid request with username
        let valid = UpdateProfileRequest {
            username: Some("newuser".to_string()),
            email: None,
        };
        assert!(valid.validate().is_ok());

        // Valid request with email
        let valid = UpdateProfileRequest {
            username: None,
            email: Some("new@example.com".to_string()),
        };
        assert!(valid.validate().is_ok());

        // Invalid username (too short)
        let invalid = UpdateProfileRequest {
            username: Some("ab".to_string()),
            email: None,
        };
        assert!(invalid.validate().is_err());

        // Invalid email format
        let invalid = UpdateProfileRequest {
            username: None,
            email: Some("not-an-email".to_string()),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_change_password_request_validation() {
        // Valid request
        let valid = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "newpass123".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Invalid - new password too short
        let invalid = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "short".to_string(),
        };
        assert!(invalid.validate().is_err());

        // Invalid - empty current password
        let invalid = ChangePasswordRequest {
            current_password: "".to_string(),
            new_password: "newpass123".to_string(),
        };
        assert!(invalid.validate().is_err());
    }
}
