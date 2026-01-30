use application::errors::ServiceError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::errors::RepositoryError;
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Internal error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
}

// Diz ao Axum como converter nosso erro em uma resposta HTTP
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 1. Log do erro real no servidor
        match &self {
            AppError::Database(_) | AppError::Anyhow(_) | AppError::Internal(_) => {
                tracing::error!("Erro interno na requisição: {:?}", self)
            }
            // Logar erros de repositório que sejam de infraestrutura (Database) como erro
            AppError::Repository(RepositoryError::Database(_)) => {
                tracing::error!("Erro de banco no repositório: {:?}", self)
            }
            _ => tracing::info!("Erro cliente na requisição: {:?}", self),
        }

        // 2. Determine status code and safe message for client
        let (status, error_message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error.".to_string(),
            ),
            // Smart mapping of Repository errors
            AppError::Repository(repo_err) => match repo_err {
                RepositoryError::NotFound => {
                    (StatusCode::NOT_FOUND, "Resource not found.".to_string())
                }
                RepositoryError::Duplicate(msg) => (StatusCode::CONFLICT, msg),
                RepositoryError::Database(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal persistence error.".to_string(),
                ),
                RepositoryError::ForeignKey(msg) => (
                    StatusCode::BAD_REQUEST,
                    format!("Foreign key constraint: {}", msg),
                ),
                RepositoryError::InvalidData(msg) => (StatusCode::BAD_REQUEST, msg),
                RepositoryError::Transaction(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Transaction error.".to_string(),
                ),
            },
            // Mapping of Service errors
            AppError::Service(service_err) => match service_err {
                ServiceError::UserAlreadyExists => (
                    StatusCode::CONFLICT,
                    "User already exists.".to_string(),
                ),
                ServiceError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid credentials.".to_string(),
                ),
                ServiceError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
                ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                ServiceError::Conflict(msg) => (StatusCode::CONFLICT, msg),
                ServiceError::Repository(msg) | ServiceError::RepositoryError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Repository error: {}", msg),
                ),
                ServiceError::Internal(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal service error.".to_string(),
                ),
            },
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::InvalidPassword => (
                StatusCode::UNAUTHORIZED,
                "Invalid username or password.".to_string(),
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Anyhow(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected internal error.".to_string(),
            ),
        };

        // 3. Cria o corpo da resposta JSON
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
