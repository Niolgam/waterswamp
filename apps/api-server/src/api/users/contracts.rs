//! User API Contracts (DTOs)
//!
//! Data Transfer Objects para operações de self-service de usuários.

use chrono::NaiveDateTime;
use domain::models::UserDtoExtended;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::utils::{ROLE_ADMIN, ROLE_USER};

// Re-export do domain para tipos compartilhados

// =============================================================================
// PROFILE
// =============================================================================

/// Response com informações do perfil do usuário
#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserDtoExtended> for ProfileResponse {
    fn from(user: UserDtoExtended) -> Self {
        // Parse role string to Enum, defaulting to User if invalid
        let role = match user.role.as_str() {
            "admin" => ROLE_ADMIN,
            _ => ROLE_USER,
        };

        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: role.to_string(),
            email_verified: user.email_verified,
            mfa_enabled: user.mfa_enabled,
            created_at: user.created_at,
        }
    }
}

/// Request para atualizar perfil
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username deve ter entre 3 e 50 caracteres"
    ))]
    pub username: Option<String>,

    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,
}

/// Response após atualização do perfil
#[derive(Debug, Serialize)]
pub struct UpdateProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: NaiveDateTime,
    pub email_changed: bool,
}

// =============================================================================
// PASSWORD CHANGE
// =============================================================================

/// Request para alteração de senha (autenticado)
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Senha atual é obrigatória"))]
    pub current_password: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Nova senha deve ter entre 8 e 128 caracteres"
    ))]
    pub new_password: String,
}

/// Response após alteração de senha
#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
}

impl Default for ChangePasswordResponse {
    fn default() -> Self {
        Self {
            message: "Senha alterada com sucesso. Por favor, faça login novamente.".to_string(),
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_profile_request_validation() {
        // Valid with username
        let valid = UpdateProfileRequest {
            username: Some("newuser".to_string()),
            email: None,
        };
        assert!(valid.validate().is_ok());

        // Valid with email
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

        // Invalid email
        let invalid = UpdateProfileRequest {
            username: None,
            email: Some("not-an-email".to_string()),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_change_password_request_validation() {
        // Valid
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
