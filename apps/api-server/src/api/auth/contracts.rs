//! Auth API Contracts (DTOs)
//!
//! Data Transfer Objects específicos para a feature de autenticação.
//! Estes DTOs são usados para request/response na camada de API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// LOGIN
// =============================================================================

/// Request para login de usuário
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, message = "Username deve ter no mínimo 3 caracteres"))]
    pub username: String,

    #[validate(length(min = 6, message = "Senha deve ter no mínimo 6 caracteres"))]
    pub password: String,
}

/// Response de login bem-sucedido (sem MFA)
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl LoginResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

/// Response quando MFA é necessário
#[derive(Debug, Serialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub mfa_token: String,
    pub message: String,
}

impl MfaRequiredResponse {
    pub fn new(mfa_token: String) -> Self {
        Self {
            mfa_required: true,
            mfa_token,
            message: "Autenticação de dois fatores necessária".to_string(),
        }
    }
}

// =============================================================================
// REGISTER
// =============================================================================

/// Request para registro de novo usuário
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 8, message = "Senha deve ter no mínimo 8 caracteres"))]
    pub password: String,
}

/// Response de registro bem-sucedido
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub message: String,
    pub user_id: Uuid,
}

impl RegisterResponse {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user_id: Uuid,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            message: "Registro realizado com sucesso. Verifique seu email para ativar a conta."
                .to_string(),
            user_id,
        }
    }
}

// =============================================================================
// REFRESH TOKEN
// =============================================================================

/// Request para renovar access token
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token não pode estar vazio"))]
    pub refresh_token: String,
}

/// Response de renovação de token
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl RefreshTokenResponse {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

// =============================================================================
// LOGOUT
// =============================================================================

/// Request para logout (revogar refresh token)
#[derive(Debug, Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1, message = "Refresh token não pode estar vazio"))]
    pub refresh_token: String,
}

/// Response de logout
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

impl Default for LogoutResponse {
    fn default() -> Self {
        Self {
            message: "Logout realizado com sucesso".to_string(),
        }
    }
}

// =============================================================================
// PASSWORD RESET
// =============================================================================

/// Request para solicitar reset de senha
#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Email inválido"))]
    pub email: String,
}

/// Response de solicitação de reset (sempre igual para evitar enumeração)
#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

impl Default for ForgotPasswordResponse {
    fn default() -> Self {
        Self {
            message:
                "Se este email estiver registado, um link de redefinição de senha foi enviado."
                    .to_string(),
        }
    }
}

/// Request para redefinir senha com token
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "Token não pode estar vazio"))]
    pub token: String,

    #[validate(length(min = 8, message = "Nova senha deve ter no mínimo 8 caracteres"))]
    pub new_password: String,
}

/// Response de reset de senha
#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

impl Default for ResetPasswordResponse {
    fn default() -> Self {
        Self {
            message: "Senha atualizada com sucesso".to_string(),
        }
    }
}

// =============================================================================
// GENERIC ERROR RESPONSE
// =============================================================================

/// Response genérica de erro (para documentação OpenAPI)
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

// =============================================================================
// USER INFO (para claims extraídos do JWT)
// =============================================================================

/// Informações do usuário autenticado extraídas do JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_validation() {
        // Valid request
        let valid = LoginRequest {
            username: "alice".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Invalid - username too short
        let invalid_username = LoginRequest {
            username: "ab".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_username.validate().is_err());

        // Invalid - password too short
        let invalid_password = LoginRequest {
            username: "alice".to_string(),
            password: "12345".to_string(),
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_register_request_validation() {
        // Valid request
        let valid = RegisterRequest {
            username: "newuser".to_string(),
            email: "user@example.com".to_string(),
            password: "SecureP@ss123".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Invalid email
        let invalid_email = RegisterRequest {
            username: "newuser".to_string(),
            email: "not-an-email".to_string(),
            password: "SecureP@ss123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // Invalid - password too short
        let invalid_password = RegisterRequest {
            username: "newuser".to_string(),
            email: "user@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_login_response_creation() {
        let response = LoginResponse::new(
            "access_token_value".to_string(),
            "refresh_token_value".to_string(),
            3600,
        );

        assert_eq!(response.access_token, "access_token_value");
        assert_eq!(response.refresh_token, "refresh_token_value");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
    }

    #[test]
    fn test_mfa_required_response() {
        let response = MfaRequiredResponse::new("mfa_token_123".to_string());

        assert!(response.mfa_required);
        assert_eq!(response.mfa_token, "mfa_token_123");
        assert!(!response.message.is_empty());
    }

    #[test]
    fn test_refresh_token_request_validation() {
        let valid = RefreshTokenRequest {
            refresh_token: "valid-token-uuid".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = RefreshTokenRequest {
            refresh_token: "".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_forgot_password_request_validation() {
        let valid = ForgotPasswordRequest {
            email: "user@example.com".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = ForgotPasswordRequest {
            email: "invalid-email".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_reset_password_request_validation() {
        let valid = ResetPasswordRequest {
            token: "reset-token-123".to_string(),
            new_password: "NewSecureP@ss123".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Invalid - empty token
        let invalid_token = ResetPasswordRequest {
            token: "".to_string(),
            new_password: "NewSecureP@ss123".to_string(),
        };
        assert!(invalid_token.validate().is_err());

        // Invalid - password too short
        let invalid_password = ResetPasswordRequest {
            token: "reset-token-123".to_string(),
            new_password: "short".to_string(),
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_default_responses() {
        let logout = LogoutResponse::default();
        assert!(!logout.message.is_empty());

        let forgot = ForgotPasswordResponse::default();
        assert!(forgot.message.contains("email"));

        let reset = ResetPasswordResponse::default();
        assert!(reset.message.contains("sucesso"));
    }
}
