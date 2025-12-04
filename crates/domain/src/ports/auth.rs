use crate::errors::RepositoryError;
use crate::models::RefreshToken;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait AuthRepositoryPort: Send + Sync {
    /// Salva um novo refresh token (usado no login)
    async fn save_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        family_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    /// Busca um refresh token pelo hash (incluindo revogados/expirados para verificação)
    async fn find_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError>;

    /// Rotaciona um token: revoga o antigo e salva o novo numa transação
    async fn rotate_refresh_token(
        &self,
        old_token_id: Uuid,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
        family_id: Uuid,
        parent_hash: &str,
        user_id: Uuid,
    ) -> Result<(), RepositoryError>;

    /// Revoga uma família inteira de tokens (usado em caso de roubo detetado)
    async fn revoke_token_family(&self, family_id: Uuid) -> Result<(), RepositoryError>;

    /// Revoga um token específico
    async fn revoke_token(&self, token_hash: &str) -> Result<bool, RepositoryError>;

    /// Revoga todos os tokens de um usuário (logout global / reset de senha)
    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), RepositoryError>;
}
