use crate::errors::RepositoryError;
use crate::models::{UserDto, UserDtoExtended, UserLoginInfo};
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

    /// Buscar informações de login por username ou email
    /// (usado apenas para autenticação - retorna password_hash e mfa_enabled)
    async fn find_for_login(&self, identifier: &str) -> Result<Option<UserLoginInfo>, RepositoryError>;

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
