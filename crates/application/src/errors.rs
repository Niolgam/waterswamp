use domain::errors::RepositoryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Usuário já existe")]
    UserAlreadyExists,

    #[error("Credenciais inválidas")]
    InvalidCredentials,

    #[error("Erro de repositório: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Erro interno: {0}")]
    Internal(#[from] anyhow::Error),
}
