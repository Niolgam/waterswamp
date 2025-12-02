//! User Self-Service Handlers
//!
//! Handlers para gerenciamento de perfil e senha do próprio usuário.

use axum::{extract::State, http::StatusCode, Json};
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};
use tracing::{error, info, instrument};
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

use super::contracts::{
    ChangePasswordRequest, ChangePasswordResponse, ProfileResponse, UpdateProfileRequest,
};
//
// ============================================================================
// Handlers
// ============================================================================

/// GET /api/v1/users/profile
///
/// Returns the authenticated user's profile information.
#[instrument(skip_all)]
pub async fn get_profile(
    State(state): State<AppState>,
    user_session: CurrentUser,
) -> Result<Json<ProfileResponse>, AppError> {
    info!(user_id = ?user_session.id, username = %user_session.username, "Fetching user profile");

    // 1. Instanciar o repositório usando o pool do state
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // 2. Usar o repositório
    let user = user_repo
        .find_extended_by_id(user_session.id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(ProfileResponse::from(user)))
}

/// PUT /api/v1/users/profile
///
/// Updates the authenticated user's profile (username and/or email).
///
/// **Note:** If email is changed, it will require re-verification.
#[instrument(skip_all)]
pub async fn update_profile(
    State(state): State<AppState>,
    user_session: CurrentUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    // 1. Validação básica (o formato de Email/Username já foi validado pelo Serde)
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    if payload.username.is_none() && payload.email.is_none() {
        return Err(AppError::BadRequest(
            "At least one field (username or email) must be provided".to_string(),
        ));
    }

    let user_id = user_session.id;
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // Buscar dados atuais
    let current_user_data = user_repo
        .find_extended_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    info!(?user_id, "Updating user profile");

    // 2. Atualizar Username
    // payload.username é Option<Username>. 'new_username' será &Username.
    if let Some(ref new_username) = payload.username {
        // Compara usando as_str() caso current_user_data.username ainda seja String
        if new_username.as_str() != current_user_data.username.as_str() {
            // O repositório espera &Username, passamos new_username direto
            if user_repo
                .exists_by_username_excluding(new_username, user_id)
                .await?
            {
                return Err(AppError::Conflict("Username already taken".to_string()));
            }

            user_repo.update_username(user_id, new_username).await?;
            info!(?user_id, new_username = %new_username.as_str(), "Username updated");
        }
    }

    // 3. Atualizar Email
    // payload.email é Option<Email>. 'new_email' será &Email.
    if let Some(ref new_email) = payload.email {
        // Compara usando as_str()
        if new_email.as_str() != current_user_data.email.as_str() {
            // O repositório espera &Email
            if user_repo
                .exists_by_email_excluding(new_email, user_id)
                .await?
            {
                return Err(AppError::Conflict("Email already taken".to_string()));
            }

            user_repo.update_email(user_id, new_email).await?;
            user_repo.mark_email_unverified(user_id).await?;

            info!(?user_id, new_email = %new_email.as_str(), "Email updated (requires re-verification)");
        }
    }

    // Retornar usuário atualizado
    let updated_user = user_repo
        .find_extended_by_id(user_id)
        .await?
        .ok_or_else(|| {
            error!(?user_id, "User not found after update");
            AppError::NotFound("User not found".to_string())
        })?;

    Ok(Json(ProfileResponse::from(updated_user)))
}

/// PUT /api/v1/users/password
///
/// Changes the authenticated user's password.
///
/// Requires the current password for security verification.
/// After changing password, all refresh tokens are revoked.
#[instrument(skip_all)]
pub async fn change_password(
    State(state): State<AppState>,
    user_session: CurrentUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<(StatusCode, Json<ChangePasswordResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let user_id = user_session.id;
    let user_repo = UserRepository::new(&state.db_pool_auth);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    let stored_hash = user_repo
        .get_password_hash(user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    let is_valid =
        core_services::security::verify_password(&payload.current_password, &stored_hash)
            .map_err(|_| AppError::Anyhow(anyhow::anyhow!("Error verifying password")))?;

    if !is_valid {
        return Err(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }

    let new_password_same_as_current =
        core_services::security::verify_password(&payload.new_password, &stored_hash)
            .unwrap_or(false);

    if new_password_same_as_current {
        return Err(AppError::BadRequest(
            "New password must be different".to_string(),
        ));
    }

    let new_hash = core_services::security::hash_password(&payload.new_password)
        .map_err(|_| AppError::Anyhow(anyhow::anyhow!("Error hashing password")))?;

    user_repo.update_password(user_id, &new_hash).await?;
    auth_repo.revoke_all_user_tokens(user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ChangePasswordResponse {
            message: "Password changed successfully.".to_string(),
        }),
    ))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use domain::{
        models::UserDtoExtended,
        value_objects::{Email, Username},
    };
    use uuid::Uuid;

    use crate::utils::ROLE_USER;

    use super::*;

    #[test]
    fn test_change_password_response_default() {
        let response = ChangePasswordResponse::default();
        assert!(response.message.contains("sucesso"));
    }

    #[test]
    fn test_profile_response_from_user_dto() {
        let user_dto = UserDtoExtended {
            id: Uuid::new_v4(),
            username: Username::try_from("testuser").unwrap(),
            email: Email::try_from("test@example.com").unwrap(),
            role: "user".to_string(),
            email_verified: true,
            email_verified_at: None,
            mfa_enabled: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let response = ProfileResponse::from(user_dto);

        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role, ROLE_USER);
        assert!(response.email_verified);
        assert!(!response.mfa_enabled);
    }
}
