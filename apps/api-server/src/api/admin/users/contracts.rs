use domain::models::UserDtoExtended;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub use domain::models::CreateUserPayload as AdminCreateUserRequest;
pub use domain::models::UpdateUserPayload as AdminUpdateUserRequest;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserListResponse {
    /// Lista de usuários
    pub users: Vec<UserDtoExtended>,
    /// Total de usuários
    pub total: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BanUserRequest {
    /// Razão do banimento (opcional)
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserActionResponse {
    /// ID do usuário afetado
    pub user_id: Uuid,
    /// Ação executada
    pub action: String,
    /// Indica se a ação foi bem-sucedida
    pub success: bool,
    /// Mensagem descritiva
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationParams {
    /// Limite de resultados por página
    pub limit: Option<i64>,
    /// Offset para paginação
    pub offset: Option<i64>,
    /// Termo de busca
    pub search: Option<String>,
}
