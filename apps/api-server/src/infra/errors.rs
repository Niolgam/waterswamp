use application::errors::ServiceError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::errors::RepositoryError; // <--- Importante!
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Erro de banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    // Novo: Erro vindo da camada de domínio/repositório
    #[error("Erro de repositório: {0}")]
    Repository(#[from] RepositoryError),

    // Erro vindo da camada de serviço/aplicação
    #[error("Erro de serviço: {0}")]
    Service(#[from] ServiceError),

    #[error("Não autorizado: {0}")]
    Unauthorized(String),

    #[error("Acesso negado: {0}")]
    Forbidden(String),

    #[error("Não encontrado: {0}")]
    NotFound(String),

    #[error("Conflito: {0}")]
    Conflict(String),

    #[error("Requisição inválida: {0}")]
    BadRequest(String),

    #[error("Erro interno: {0}")]
    Internal(String),

    #[error("Erro interno: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Senha inválida")]
    InvalidPassword,

    #[error("Erro de validação: {0}")]
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

        // 2. Determina o status code e a mensagem segura para o cliente
        let (status, error_message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro interno no servidor.".to_string(),
            ),
            // Mapeamento inteligente dos erros do Repositório
            AppError::Repository(repo_err) => match repo_err {
                RepositoryError::NotFound => {
                    (StatusCode::NOT_FOUND, "Recurso não encontrado.".to_string())
                }
                RepositoryError::Duplicate(msg) => (
                    StatusCode::CONFLICT,
                    msg, // "Email já existe", etc.
                ),
                RepositoryError::Database(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erro interno de persistência.".to_string(),
                ),
            },
            // Mapeamento dos erros de serviço
            AppError::Service(service_err) => match service_err {
                ServiceError::UserAlreadyExists => (
                    StatusCode::CONFLICT,
                    "Usuário já existe.".to_string(),
                ),
                ServiceError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "Credenciais inválidas.".to_string(),
                ),
                ServiceError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
                ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                ServiceError::Conflict(msg) => (StatusCode::CONFLICT, msg),
                ServiceError::Repository(repo_err) => match repo_err {
                    RepositoryError::NotFound => {
                        (StatusCode::NOT_FOUND, "Recurso não encontrado.".to_string())
                    }
                    RepositoryError::Duplicate(msg) => (StatusCode::CONFLICT, msg),
                    RepositoryError::Database(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Erro interno de persistência.".to_string(),
                    ),
                },
                ServiceError::Internal(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erro interno no serviço.".to_string(),
                ),
            },
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::InvalidPassword => (
                StatusCode::UNAUTHORIZED,
                "Usuário ou senha inválidos.".to_string(),
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Anyhow(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro interno inesperado.".to_string(),
            ),
        };

        // 3. Cria o corpo da resposta JSON
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
