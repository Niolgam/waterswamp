use crate::errors::RepositoryError;
use crate::models::RefreshToken;
use crate::models::{UserDto, UserDtoExtended};
use crate::value_objects::{Email, Username};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepositoryPort: Send + Sync {
    // Leitura
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, RepositoryError>;
    async fn find_extended_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<UserDtoExtended>, RepositoryError>;
    async fn find_by_username(
        &self,
        username: &Username,
    ) -> Result<Option<UserDto>, RepositoryError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<UserDto>, RepositoryError>;

    // Verificações
    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError>;
    async fn exists_by_username(&self, username: &Username) -> Result<bool, RepositoryError>;
    async fn exists_by_email_excluding(
        &self,
        email: &Email,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn exists_by_username_excluding(
        &self,
        username: &Username,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError>;

    // Auth helpers
    async fn get_password_hash(&self, id: Uuid) -> Result<Option<String>, RepositoryError>;

    // Escrita
    async fn create(
        &self,
        username: &Username,
        email: &Email,
        password_hash: &str,
    ) -> Result<UserDto, RepositoryError>;

    async fn update_username(
        &self,
        id: Uuid,
        new_username: &Username,
    ) -> Result<(), RepositoryError>;
    async fn update_email(&self, id: Uuid, new_email: &Email) -> Result<(), RepositoryError>;
    async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: &str,
    ) -> Result<(), RepositoryError>;
    async fn update_role(&self, id: Uuid, new_role: &str) -> Result<(), RepositoryError>;
    async fn mark_email_unverified(&self, id: Uuid) -> Result<(), RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    // Listagem (Admin)
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<UserDto>, i64), RepositoryError>;
}

#[async_trait]
pub trait EmailServicePort: Send + Sync {
    async fn send_verification_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String>;
    async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), String>;
    async fn send_password_reset_email(
        &self,
        to: &Email,
        username: &Username,
        token: &str,
    ) -> Result<(), String>;
    async fn send_mfa_enabled_email(&self, to: &Email, username: &Username) -> Result<(), String>;
}

#[async_trait]
pub trait AuthRepositoryPort: Send + Sync {
    /// Salva um novo refresh token (usado no login)
    async fn save_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        family_id: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
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
        new_expires_at: chrono::DateTime<chrono::Utc>,
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

#[async_trait]
pub trait MfaRepositoryPort: Send + Sync {
    async fn save_setup_token(
        &self,
        user_id: Uuid,
        secret: &str,
        token_hash: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
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
