use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Erro de banco de dados: {0}")]
    Database(#[from] sqlx::Error),

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
        // 1. Log do erro real no servidor (para nós, desenvolvedores)
        // Erros 4xx não precisam ser logados como ERROR, podem ser INFO ou WARN
        match &self {
            AppError::Database(_) | AppError::Anyhow(_) | AppError::Internal(_) => {
                tracing::error!("Erro interno na requisição: {:?}", self)
            }
            _ => tracing::info!("Erro cliente na requisição: {:?}", self),
        }

        // 2. Determina o status code e a mensagem segura para o cliente
        let (status, error_message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro interno no servidor.".to_string(),
            ),
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
