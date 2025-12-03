use crate::errors::ServiceError;
use core_services::jwt::JwtService;
use core_services::security::hash_password;
use domain::models::{RegisterPayload, TokenType, UserDto};
use domain::ports::{EmailServicePort, UserRepositoryPort};
use domain::value_objects::{Email, Username};
use std::sync::Arc;

pub struct RegisterResult {
    pub user: UserDto,
    pub access_token: String,
    pub refresh_token: String,
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepositoryPort>,
    email_service: Arc<dyn EmailServicePort>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepositoryPort>,
        email_service: Arc<dyn EmailServicePort>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_repo,
            email_service,
            jwt_service,
        }
    }

    pub async fn register_user(
        &self,
        req: RegisterPayload,
    ) -> Result<RegisterResult, ServiceError> {
        // 1. Validar unicidade (Regra de Negócio)
        if self.user_repo.exists_by_email(&req.email).await? {
            return Err(ServiceError::UserAlreadyExists);
        }
        if self.user_repo.exists_by_username(&req.username).await? {
            return Err(ServiceError::UserAlreadyExists);
        }

        // 2. Hash da senha (Lógica de Segurança)
        // Note: Em produção real, idealmente isso rodaria em spawn_blocking aqui ou no core-service
        let password_hash =
            hash_password(&req.password).map_err(|e| ServiceError::Internal(anyhow::anyhow!(e)))?;

        // 3. Criar Usuário (Persistência)
        let user = self
            .user_repo
            .create(&req.username, &req.email, &password_hash)
            .await?;

        // 4. Gerar Tokens (Segurança)
        let access_token = self
            .jwt_service
            .generate_token(user.id, TokenType::Access, 3600)
            .map_err(|e| ServiceError::Internal(e))?;

        let refresh_token = uuid::Uuid::new_v4().to_string(); // Simplificação, idealmente via service também

        // TODO: Persistir refresh token (precisaríamos expor AuthRepositoryPort também)

        // 5. Enviar Email (Notificação)
        // Aqui usaríamos um token real gerado
        let verification_token = "dummy-token";

        // Dispara envio de email (fire and forget ou await dependendo da regra)
        let _ = self
            .email_service
            .send_verification_email(&req.email, &req.username, verification_token)
            .await;
        let _ = self
            .email_service
            .send_welcome_email(&req.email, &req.username)
            .await;

        Ok(RegisterResult {
            user,
            access_token,
            refresh_token,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::{UserDto, UserDtoExtended};
    use domain::value_objects::{Email, Username};
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    mock! {
        pub UserRepo {}
        #[async_trait::async_trait]
        impl UserRepositoryPort for UserRepo {
            async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, domain::errors::RepositoryError>;
            async fn find_extended_by_id(&self, id: Uuid) -> Result<Option<UserDtoExtended>, domain::errors::RepositoryError>;
            async fn find_by_username(&self, username: &Username) -> Result<Option<UserDto>, domain::errors::RepositoryError>;
            async fn find_by_email(&self, email: &Email) -> Result<Option<UserDto>, domain::errors::RepositoryError>;

            async fn exists_by_email(&self, email: &Email) -> Result<bool, domain::errors::RepositoryError>;
            async fn exists_by_username(&self, username: &Username) -> Result<bool, domain::errors::RepositoryError>;
            async fn exists_by_email_excluding(&self, email: &Email, exclude_id: Uuid) -> Result<bool, domain::errors::RepositoryError>;
            async fn exists_by_username_excluding(&self, username: &Username, exclude_id: Uuid) -> Result<bool, domain::errors::RepositoryError>;

            async fn get_password_hash(&self, id: Uuid) -> Result<Option<String>, domain::errors::RepositoryError>;

            async fn create(
                &self,
                username: &Username,
                email: &Email,
                password_hash: &str
            ) -> Result<UserDto, domain::errors::RepositoryError>;

            async fn update_username(&self, id: Uuid, new_username: &Username) -> Result<(), domain::errors::RepositoryError>;
            async fn update_email(&self, id: Uuid, new_email: &Email) -> Result<(), domain::errors::RepositoryError>;
            async fn update_password(&self, id: Uuid, new_password_hash: &str) -> Result<(), domain::errors::RepositoryError>;
            async fn update_role(&self, id: Uuid, new_role: &str) -> Result<(), domain::errors::RepositoryError>;
            async fn mark_email_unverified(&self, id: Uuid) -> Result<(), domain::errors::RepositoryError>;

            async fn delete(&self, id: Uuid) -> Result<bool, domain::errors::RepositoryError>;

            async fn list(
                &self,
                limit: i64,
                offset: i64,
                search: Option<&String>
            ) -> Result<(Vec<UserDto>, i64), domain::errors::RepositoryError>;
        }
    }

    // 2. Mock do EmailServicePort
    mock! {
        pub EmailService {}
        #[async_trait::async_trait]
        impl EmailServicePort for EmailService {
            async fn send_verification_email(&self, to: &Email, username: &Username, token: &str) -> Result<(), String>;
            async fn send_welcome_email(&self, to: &Email, username: &Username) -> Result<(), String>;
            async fn send_password_reset_email(&self, to: &Email, username: &Username, token: &str) -> Result<(), String>;
        }
    }

    // Helper para criar JwtService válido nos testes
    fn create_test_jwt_service() -> Arc<JwtService> {
        // Chaves EdDSA dummy para teste (não use em produção!)
        let private_pem = b"-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIL/Ue/jT2+VqQ3+X2+VqQ3+X2+VqQ3+X2+VqQ3+X2+Vq\n-----END PRIVATE KEY-----";
        let public_pem = b"-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEA/9R7+NPb5WpDf5fb5WpDf5fb5WpDf5fb5WpDf5fb5Wo=\n-----END PUBLIC KEY-----";

        Arc::new(JwtService::new(private_pem, public_pem).unwrap())
    }

    #[tokio::test]
    async fn test_register_success() {
        // Arrange
        let mut mock_repo = MockUserRepo::new();
        let mut mock_email = MockEmailService::new();
        let jwt_service = create_test_jwt_service(); // <--- CORREÇÃO

        let username = Username::try_from("newuser").unwrap();
        let email = Email::try_from("test@example.com").unwrap();
        let password = "SecurePass123!";

        // Expectativas do Repositório
        mock_repo
            .expect_exists_by_email()
            .with(eq(email.clone()))
            .times(1)
            .returning(|_| Ok(false)); // Não existe

        mock_repo
            .expect_exists_by_username()
            .with(eq(username.clone()))
            .times(1)
            .returning(|_| Ok(false)); // Não existe

        let created_user = UserDto {
            id: Uuid::new_v4(),
            username: username.clone(),
            email: email.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_user_clone = created_user.clone(); // Agora funciona (derive Clone)

        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_, _, _| Ok(created_user_clone.clone()));

        // Expectativas do Email
        mock_email
            .expect_send_verification_email()
            .times(1)
            .returning(|_, _, _| Ok(()));

        mock_email
            .expect_send_welcome_email()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = AuthService::new(Arc::new(mock_repo), Arc::new(mock_email), jwt_service);

        // Act
        let payload = RegisterPayload {
            username,
            email,
            password: password.to_string(),
        };

        let result = service.register_user(payload).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.user.id, created_user.id);
        assert!(!response.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        // Arrange
        let mut mock_repo = MockUserRepo::new();
        let mock_email = MockEmailService::new();
        let jwt_service = create_test_jwt_service(); // <--- CORREÇÃO

        let email = Email::try_from("exists@example.com").unwrap();

        // Simula que email já existe
        mock_repo
            .expect_exists_by_email()
            .with(eq(email.clone()))
            .returning(|_| Ok(true));

        let service = AuthService::new(Arc::new(mock_repo), Arc::new(mock_email), jwt_service);

        // Act
        let payload = RegisterPayload {
            username: Username::try_from("user").unwrap(),
            email,
            password: "Pass".to_string(),
        };

        let result = service.register_user(payload).await;

        // Assert
        assert!(matches!(result, Err(ServiceError::UserAlreadyExists)));
    }
}
