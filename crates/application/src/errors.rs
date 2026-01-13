use domain::errors::RepositoryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Usuário já existe")]
    UserAlreadyExists,

    #[error("Credenciais inválidas")]
    InvalidCredentials,

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Erro de repositório: {0}")]
    Repository(String),

    #[error("Erro interno: {0}")]
    Internal(String),
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
        }
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::Internal(err.to_string())
    }
}
