use crate::errors::RepositoryError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait MfaRepositoryPort: Send + Sync {
    // Gestão de Setup (Token temporário durante a configuração)
    async fn save_setup_token(
        &self,
        user_id: Uuid,
        secret: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    async fn find_setup_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<(Uuid, String)>, RepositoryError>; // Retorna (user_id, secret)

    async fn delete_setup_token(&self, token_hash: &str) -> Result<(), RepositoryError>;

    // Gestão de MFA Ativo (Segredo confirmado)
    async fn enable_mfa(&self, user_id: Uuid, secret: &str) -> Result<(), RepositoryError>;
    async fn disable_mfa(&self, user_id: Uuid) -> Result<(), RepositoryError>;

    // Backup Codes
    async fn save_backup_codes(
        &self,
        user_id: Uuid,
        codes: &[String],
    ) -> Result<(), RepositoryError>;
    async fn get_backup_codes(&self, user_id: Uuid) -> Result<Vec<String>, RepositoryError>;

    /// Verifica e consome um código de backup. Retorna true se válido.
    async fn verify_and_consume_backup_code(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<bool, RepositoryError>;

    // Leitura
    async fn get_mfa_secret(&self, user_id: Uuid) -> Result<Option<String>, RepositoryError>;
    async fn is_mfa_enabled(&self, user_id: Uuid) -> Result<bool, RepositoryError>;
}
