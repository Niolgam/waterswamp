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

    #[error("Erro de validação: {0}")]
    ValidationError(String),

    #[error("Não encontrado: {0}")]
    NotFound(String),

    #[error("Erro de repositório: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Erro interno: {0}")]
    Internal(#[from] anyhow::Error),
}
