use domain::errors::RepositoryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

impl ServiceError {
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            ServiceError::UserAlreadyExists => http::StatusCode::CONFLICT,
            ServiceError::InvalidCredentials => http::StatusCode::UNAUTHORIZED,
            ServiceError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
            ServiceError::NotFound(_) => http::StatusCode::NOT_FOUND,
            ServiceError::Conflict(_) => http::StatusCode::CONFLICT,
            ServiceError::Repository(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::RepositoryError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::Internal(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<&ServiceError> for http::StatusCode {
    fn from(err: &ServiceError) -> Self {
        err.status_code()
    }
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => ServiceError::NotFound("Resource not found".to_string()),
            RepositoryError::Duplicate(msg) => ServiceError::Conflict(msg),
            RepositoryError::Database(msg) => ServiceError::Repository(msg),
            RepositoryError::ForeignKey(msg) => ServiceError::BadRequest(format!("Foreign key constraint: {}", msg)),
            RepositoryError::InvalidData(msg) => ServiceError::BadRequest(msg),
            RepositoryError::Transaction(msg) => ServiceError::Internal(format!("Transaction error: {}", msg)),
        }
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::Internal(err.to_string())
    }
}
