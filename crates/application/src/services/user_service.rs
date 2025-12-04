use crate::errors::ServiceError;
use core_services::security::{hash_password, verify_password};
use domain::models::{UpdateUserPayload, UserDtoExtended};
use domain::ports::{AuthRepositoryPort, EmailServicePort, UserRepositoryPort};
use std::sync::Arc;
use uuid::Uuid;

pub struct UserService {
    user_repo: Arc<dyn UserRepositoryPort>,
    auth_repo: Arc<dyn AuthRepositoryPort>,
    email_service: Arc<dyn EmailServicePort>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepositoryPort>,
        auth_repo: Arc<dyn AuthRepositoryPort>,
        email_service: Arc<dyn EmailServicePort>,
    ) -> Self {
        Self {
            user_repo,
            auth_repo,
            email_service,
        }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserDtoExtended, ServiceError> {
        self.user_repo
            .find_extended_by_id(user_id)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or(ServiceError::Internal(anyhow::anyhow!(
                "Usuário não encontrado"
            )))
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        payload: UpdateUserPayload,
    ) -> Result<UserDtoExtended, ServiceError> {
        let current_user = self.get_profile(user_id).await?;

        if let Some(new_username) = &payload.username {
            if new_username.as_str() != current_user.username.as_str() {
                if self
                    .user_repo
                    .exists_by_username_excluding(new_username, user_id)
                    .await?
                {
                    return Err(ServiceError::UserAlreadyExists);
                }

                self.user_repo
                    .update_username(user_id, new_username)
                    .await?;
            }
        }

        if let Some(new_email) = &payload.email {
            if new_email.as_str() != current_user.email.as_str() {
                if self
                    .user_repo
                    .exists_by_email_excluding(new_email, user_id)
                    .await?
                {
                    return Err(ServiceError::UserAlreadyExists);
                }

                self.user_repo.update_email(user_id, new_email).await?;

                self.user_repo.mark_email_unverified(user_id).await?;

                // TODO: Gerar token real via serviço de JWT ou similar
                let verification_token = "dummy-token-update-profile";

                let _ = self
                    .email_service
                    .send_verification_email(
                        new_email,
                        payload.username.as_ref().unwrap_or(&current_user.username),
                        verification_token,
                    )
                    .await;
            }
        }

        self.get_profile(user_id).await
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        let stored_hash =
            self.user_repo
                .get_password_hash(user_id)
                .await?
                .ok_or(ServiceError::Internal(anyhow::anyhow!(
                    "Usuário sem senha ou não encontrado"
                )))?;

        let is_valid = verify_password(current_password, &stored_hash)
            .map_err(|_| ServiceError::InvalidCredentials)?;

        if !is_valid {
            return Err(ServiceError::InvalidCredentials);
        }

        // Verificar se a nova é igual à antiga
        // Isso evita trabalho desnecessário, mas verify é custoso.
        // Se a regra de negócio exigir, descomente:
        // if verify_password(new_password, &stored_hash).unwrap_or(false) {
        //     return Err(ServiceError::Internal(anyhow::anyhow!("Nova senha igual à atual")));
        // }

        let new_hash =
            hash_password(new_password).map_err(|e| ServiceError::Internal(anyhow::anyhow!(e)))?;

        self.user_repo.update_password(user_id, &new_hash).await?;

        self.auth_repo.revoke_all_user_tokens(user_id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use domain::errors::RepositoryError;
    use domain::models::{RefreshToken, UserDto, UserDtoExtended};
    use domain::value_objects::{Email, Username};
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    // --- MOCKS ---
    // (Repetimos a definição aqui para isolamento, em um projeto real pode ir para um test_utils.rs compartilhado)

    mock! {
        pub UserRepo {}
        #[async_trait::async_trait]
        impl UserRepositoryPort for UserRepo {
            async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, RepositoryError>;
            async fn find_extended_by_id(&self, id: Uuid) -> Result<Option<UserDtoExtended>, RepositoryError>;
            async fn find_by_username(&self, username: &Username) -> Result<Option<UserDto>, RepositoryError>;
            async fn find_by_email(&self, email: &Email) -> Result<Option<UserDto>, RepositoryError>;

            async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError>;
            async fn exists_by_username(&self, username: &Username) -> Result<bool, RepositoryError>;
            async fn exists_by_email_excluding(&self, email: &Email, exclude_id: Uuid) -> Result<bool, RepositoryError>;
            async fn exists_by_username_excluding(&self, username: &Username, exclude_id: Uuid) -> Result<bool, RepositoryError>;

            async fn get_password_hash(&self, id: Uuid) -> Result<Option<String>, RepositoryError>;

            async fn create(&self, username: &Username, email: &Email, password_hash: &str) -> Result<UserDto, RepositoryError>;
            async fn update_username(&self, id: Uuid, new_username: &Username) -> Result<(), RepositoryError>;
            async fn update_email(&self, id: Uuid, new_email: &Email) -> Result<(), RepositoryError>;
            async fn update_password(&self, id: Uuid, new_password_hash: &str) -> Result<(), RepositoryError>;
            async fn update_role(&self, id: Uuid, new_role: &str) -> Result<(), RepositoryError>;
            async fn mark_email_unverified(&self, id: Uuid) -> Result<(), RepositoryError>;
            async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
            async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<UserDto>, i64), RepositoryError>;
        }
    }

    mock! {
        pub AuthRepo {}
        #[async_trait::async_trait]
        impl AuthRepositoryPort for AuthRepo {
            async fn save_refresh_token(&self, user_id: Uuid, token_hash: &str, family_id: Uuid, expires_at: DateTime<Utc>) -> Result<(), RepositoryError>;
            async fn find_token_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>, RepositoryError>;
            async fn rotate_refresh_token(&self, old_token_id: Uuid, new_token_hash: &str, new_expires_at: DateTime<Utc>, family_id: Uuid, parent_hash: &str, user_id: Uuid) -> Result<(), RepositoryError>;
            async fn revoke_token_family(&self, family_id: Uuid) -> Result<(), RepositoryError>;
            async fn revoke_token(&self, token_hash: &str) -> Result<bool, RepositoryError>;
            async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), RepositoryError>;
        }
    }

    mock! {
        pub EmailService {}
        #[async_trait::async_trait]
        impl EmailServicePort for EmailService {
            async fn send_verification_email(&self, to: &Email, username: &Username, token: &str) -> Result<(), String>;
            async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), String>;
            async fn send_password_reset_email(&self, to: &Email, username: &Username, token: &str) -> Result<(), String>;
            async fn send_mfa_enabled_email(&self, to: &Email, username: &Username) -> Result<(), String>;
        }
    }

    // --- HELPERS ---

    fn create_dummy_user(id: Uuid) -> UserDtoExtended {
        UserDtoExtended {
            id,
            username: Username::try_from("testuser").unwrap(),
            email: Email::try_from("test@example.com").unwrap(),
            role: "user".to_string(),
            email_verified: true,
            email_verified_at: Some(Utc::now()),
            mfa_enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // --- TESTES ---

    #[tokio::test]
    async fn test_get_profile_success() {
        let mut mock_user_repo = MockUserRepo::new();
        let mock_auth_repo = MockAuthRepo::new();
        let mock_email = MockEmailService::new();

        let user_id = Uuid::new_v4();
        let user = create_dummy_user(user_id);
        let user_clone = user.clone();

        mock_user_repo
            .expect_find_extended_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(user_clone.clone())));

        let service = UserService::new(
            Arc::new(mock_user_repo),
            Arc::new(mock_auth_repo),
            Arc::new(mock_email),
        );

        let result = service.get_profile(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username.as_str(), "testuser");
    }

    #[tokio::test]
    async fn test_update_profile_username_success() {
        let mut mock_user_repo = MockUserRepo::new();
        let mock_auth_repo = MockAuthRepo::new();
        let mock_email = MockEmailService::new();

        let user_id = Uuid::new_v4();
        let user = create_dummy_user(user_id);
        let new_username = Username::try_from("newname").unwrap();

        // Setup para find (retorna user atual)
        let user_clone = user.clone();
        mock_user_repo
            .expect_find_extended_by_id()
            .with(eq(user_id))
            .times(2) // 1x no início, 1x no retorno final
            .returning(move |_| Ok(Some(user_clone.clone())));

        // Setup para check de duplicidade
        mock_user_repo
            .expect_exists_by_username_excluding()
            .with(eq(new_username.clone()), eq(user_id))
            .times(1)
            .returning(|_, _| Ok(false)); // Não existe

        // Setup para update
        mock_user_repo
            .expect_update_username()
            .with(eq(user_id), eq(new_username.clone()))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = UserService::new(
            Arc::new(mock_user_repo),
            Arc::new(mock_auth_repo),
            Arc::new(mock_email),
        );

        let payload = UpdateUserPayload {
            username: Some(new_username),
            email: None,
            password: None,
            role: None,
        };

        let result = service.update_profile(user_id, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_profile_email_triggers_verification() {
        let mut mock_user_repo = MockUserRepo::new();
        let mock_auth_repo = MockAuthRepo::new();
        let mut mock_email = MockEmailService::new();

        let user_id = Uuid::new_v4();
        let user = create_dummy_user(user_id);
        let new_email = Email::try_from("new@example.com").unwrap();

        // 1. Busca perfil inicial
        let user_clone = user.clone();
        mock_user_repo
            .expect_find_extended_by_id()
            .times(2) // Início e Fim
            .returning(move |_| Ok(Some(user_clone.clone())));

        // 2. Check unicidade email
        mock_user_repo
            .expect_exists_by_email_excluding()
            .returning(|_, _| Ok(false));

        // 3. Update email
        mock_user_repo
            .expect_update_email()
            .with(eq(user_id), eq(new_email.clone()))
            .times(1)
            .returning(|_, _| Ok(()));

        // 4. Mark unverified (Regra crítica!)
        mock_user_repo
            .expect_mark_email_unverified()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(()));

        // 5. Envia email
        mock_email
            .expect_send_verification_email()
            .with(eq(new_email.clone()), eq(user.username.clone()), always())
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserService::new(
            Arc::new(mock_user_repo),
            Arc::new(mock_auth_repo),
            Arc::new(mock_email),
        );

        let payload = UpdateUserPayload {
            username: None,
            email: Some(new_email),
            password: None,
            role: None,
        };

        let result = service.update_profile(user_id, payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_change_password_success() {
        let mut mock_user_repo = MockUserRepo::new();
        let mut mock_auth_repo = MockAuthRepo::new();
        let mock_email = MockEmailService::new();

        let user_id = Uuid::new_v4();
        let old_pass = "OldPass123!";
        let new_pass = "NewPass456!";

        // Hash do "OldPass123!" (Argon2 simulado ou real, aqui usamos real se possível, ou mockamos o verify)
        // Para simplificar, vamos assumir que o hash_password e verify_password do core_services funcionam.
        let hashed_old = hash_password(old_pass).unwrap();

        mock_user_repo
            .expect_get_password_hash()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(hashed_old.clone())));

        mock_user_repo
            .expect_update_password()
            .with(eq(user_id), always()) // Não validamos o hash exato da nova senha pois é aleatório
            .times(1)
            .returning(|_, _| Ok(()));

        // Regra crítica: Revogar tokens!
        mock_auth_repo
            .expect_revoke_all_user_tokens()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(()));

        let service = UserService::new(
            Arc::new(mock_user_repo),
            Arc::new(mock_auth_repo),
            Arc::new(mock_email),
        );

        let result = service.change_password(user_id, old_pass, new_pass).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_change_password_wrong_current() {
        let mut mock_user_repo = MockUserRepo::new();
        let mut mock_auth_repo = MockAuthRepo::new();
        let mock_email = MockEmailService::new();

        let user_id = Uuid::new_v4();
        let real_pass = "RealPass123!";
        let wrong_pass = "WrongPass!!!";
        let hashed_real = hash_password(real_pass).unwrap();

        mock_user_repo
            .expect_get_password_hash()
            .returning(move |_| Ok(Some(hashed_real.clone())));

        // Não deve chamar update nem revoke
        mock_user_repo.expect_update_password().times(0);
        mock_auth_repo.expect_revoke_all_user_tokens().times(0);

        let service = UserService::new(
            Arc::new(mock_user_repo),
            Arc::new(mock_auth_repo),
            Arc::new(mock_email),
        );

        let result = service
            .change_password(user_id, wrong_pass, "AnyNewPass")
            .await;
        assert!(matches!(result, Err(ServiceError::InvalidCredentials)));
    }
}
