//! User Self-Service Handlers
use axum::{extract::State, http::StatusCode, Json};
use tracing::{info, instrument};
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

use super::contracts::{
    ChangePasswordRequest, ChangePasswordResponse, ProfileResponse, UpdateProfileRequest,
};

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/v1/users/profile
#[instrument(skip_all)]
pub async fn get_profile(
    State(state): State<AppState>,
    user_session: CurrentUser,
) -> Result<Json<ProfileResponse>, AppError> {
    info!(user_id = ?user_session.id, username = %user_session.username, "Fetching user profile");

    let user = state
        .user_service
        .get_profile(user_session.id)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?; // Melhore o map_err se desejar

    Ok(Json(ProfileResponse::from(user)))
}

/// PUT /api/v1/users/profile
#[instrument(skip_all)]
pub async fn update_profile(
    State(state): State<AppState>,
    user_session: CurrentUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    // Validação HTTP
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    if payload.username.is_none() && payload.email.is_none() {
        return Err(AppError::BadRequest(
            "At least one field (username or email) must be provided".to_string(),
        ));
    }

    let user_id = user_session.id;
    info!(?user_id, "Updating user profile");

    // Mapear DTO API -> DTO Domínio
    let update_payload = domain::models::UpdateUserPayload {
        username: payload.username,
        email: payload.email,
        password: None, // UpdateProfileRequest não muda senha
        role: None,     // Usuário não pode mudar seu próprio role
    };

    // Delegar para o Serviço
    let updated_user = state
        .user_service
        .update_profile(user_id, update_payload)
        .await
        .map_err(|e| {
            use application::errors::ServiceError;
            match e {
                ServiceError::UserAlreadyExists => {
                    AppError::Conflict("Username ou Email já está em uso".to_string())
                }
                ServiceError::Repository(r) => AppError::Repository(r),
                _ => AppError::Anyhow(anyhow::anyhow!(e)),
            }
        })?;

    Ok(Json(ProfileResponse::from(updated_user)))
}

/// PUT /api/v1/users/password
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
    info!(?user_id, "Attempting to change password");

    // Delegar para o Serviço
    state
        .user_service
        .change_password(user_id, &payload.current_password, &payload.new_password)
        .await
        .map_err(|e| {
            use application::errors::ServiceError;
            match e {
                ServiceError::InvalidCredentials => {
                    AppError::Unauthorized("Senha atual incorreta".to_string())
                }
                _ => AppError::Anyhow(anyhow::anyhow!(e)),
            }
        })?;

    info!(?user_id, "Password changed successfully");

    Ok((
        StatusCode::OK,
        Json(ChangePasswordResponse {
            message: "Password changed successfully.".to_string(),
        }),
    ))
}

// Mantenha os testes existentes abaixo (adapte se necessário)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ROLE_USER;
    use domain::models::UserDtoExtended;
    use domain::value_objects::{Email, Username};
    use uuid::Uuid;

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
