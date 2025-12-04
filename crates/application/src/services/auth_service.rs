use crate::errors::ServiceError;
use chrono::{Duration, Utc};
use core_services::jwt::JwtService;
use core_services::security::{hash_password, verify_password};
use domain::models::{RegisterPayload, TokenType, UserDto};
use domain::ports::{AuthRepositoryPort, EmailServicePort, UserRepositoryPort};
use domain::value_objects::{Email, Username};
use sha2::{Digest, Sha256};
use std::sync::Arc;

// DTOs de resposta do serviço
pub struct AuthResult {
    pub user: UserDto,
    pub access_token: String,
    pub refresh_token: String,
    pub mfa_required: bool,
    pub mfa_token: Option<String>,
}

pub struct RegisterResult {
    pub user: UserDto,
    pub access_token: String,
    pub refresh_token: String,
}

pub struct TokenRefreshResult {
    pub access_token: String,
    pub refresh_token: String,
}

// Helpers
const ACCESS_TOKEN_EXPIRY: i64 = 3600; // 1h
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 7;
const MFA_CHALLENGE_EXPIRY: i64 = 300; // 5min
const RESET_TOKEN_EXPIRY: i64 = 900; // 15min

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepositoryPort>,
    auth_repo: Arc<dyn AuthRepositoryPort>, // Novo
    email_service: Arc<dyn EmailServicePort>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepositoryPort>,
        auth_repo: Arc<dyn AuthRepositoryPort>, // Novo
        email_service: Arc<dyn EmailServicePort>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_repo,
            auth_repo,
            email_service,
            jwt_service,
        }
    }

    // --- REGISTRO ---
    pub async fn register_user(
        &self,
        req: RegisterPayload,
    ) -> Result<RegisterResult, ServiceError> {
        if self.user_repo.exists_by_email(&req.email).await? {
            return Err(ServiceError::UserAlreadyExists);
        }
        if self.user_repo.exists_by_username(&req.username).await? {
            return Err(ServiceError::UserAlreadyExists);
        }

        let password_hash =
            hash_password(&req.password).map_err(|e| ServiceError::Internal(anyhow::anyhow!(e)))?;

        let user = self
            .user_repo
            .create(&req.username, &req.email, &password_hash)
            .await?;

        // Gerar Tokens
        let (access_token, refresh_token) = self.generate_and_save_tokens(user.id).await?;

        // Enviar Emails
        let verification_token = "dummy-token"; // TODO: Gerar token real
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

    // --- LOGIN ---
    pub async fn login(
        &self,
        username_or_email: &str,
        password: &str,
    ) -> Result<AuthResult, ServiceError> {
        // 1. Buscar usuário (tenta username, se falhar, tenta email)
        // Isso assume que o repo tem um método genérico ou tentamos os dois.
        // Vamos tentar username primeiro.
        let user_opt = if let Ok(username) = Username::try_from(username_or_email) {
            self.user_repo.find_by_username(&username).await?
        } else if let Ok(email) = Email::try_from(username_or_email) {
            self.user_repo.find_by_email(&email).await?
        } else {
            // Formato inválido para ambos
            return Err(ServiceError::InvalidCredentials);
        };

        let user = user_opt.ok_or(ServiceError::InvalidCredentials)?;

        // 2. Buscar hash da senha (precisamos do hash para verificar)
        let hash_opt = self.user_repo.get_password_hash(user.id).await?;
        let hash = hash_opt.ok_or(ServiceError::InvalidCredentials)?;

        // 3. Verificar senha
        let valid =
            verify_password(password, &hash).map_err(|_| ServiceError::InvalidCredentials)?;
        if !valid {
            return Err(ServiceError::InvalidCredentials);
        }

        // 4. Buscar dados estendidos (para ver se tem MFA)
        let user_ext =
            self.user_repo
                .find_extended_by_id(user.id)
                .await?
                .ok_or(ServiceError::Internal(anyhow::anyhow!(
                    "User lost after login"
                )))?;

        if user_ext.mfa_enabled {
            // Gerar token MFA
            let mfa_token = self
                .jwt_service
                .generate_mfa_token(user.id, MFA_CHALLENGE_EXPIRY)
                .map_err(|e| ServiceError::Internal(e))?;

            return Ok(AuthResult {
                user,
                access_token: String::new(),
                refresh_token: String::new(),
                mfa_required: true,
                mfa_token: Some(mfa_token),
            });
        }

        // 5. Gerar tokens (Login normal)
        let (access_token, refresh_token) = self.generate_and_save_tokens(user.id).await?;

        Ok(AuthResult {
            user,
            access_token,
            refresh_token,
            mfa_required: false,
            mfa_token: None,
        })
    }

    // --- REFRESH TOKEN ---
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenRefreshResult, ServiceError> {
        let token_hash = hash_token(refresh_token);

        let token_info = self.auth_repo.find_token_by_hash(&token_hash).await?;

        // 1. Token existe?
        let token = match token_info {
            Some(t) => t,
            None => return Err(ServiceError::InvalidCredentials), // "Token inválido"
        };

        // 2. Detecção de roubo (Reuse Detection)
        if token.revoked {
            // Revoga toda a família!
            tracing::warn!(
                "Reuso de token detectado! Revogando família {}",
                token.family_id
            );
            self.auth_repo.revoke_token_family(token.family_id).await?;
            return Err(ServiceError::InvalidCredentials); // "Sessão invalidada"
        }

        // 3. Expirado?
        if token.expires_at <= Utc::now() {
            return Err(ServiceError::InvalidCredentials);
        }

        // 4. Rotação: Gerar novo e invalidar antigo
        let new_refresh_token_raw = uuid::Uuid::new_v4().to_string();
        let new_refresh_token_hash = hash_token(&new_refresh_token_raw);
        let new_expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);

        // Access Token
        let access_token = self
            .jwt_service
            .generate_token(token.user_id, TokenType::Access, ACCESS_TOKEN_EXPIRY)
            .map_err(|e| ServiceError::Internal(e))?;

        // Operação atômica no repo
        self.auth_repo
            .rotate_refresh_token(
                token.id,
                &new_refresh_token_hash,
                new_expires_at,
                token.family_id,
                &token_hash,
                token.user_id,
            )
            .await?;

        Ok(TokenRefreshResult {
            access_token,
            refresh_token: new_refresh_token_raw,
        })
    }

    // --- LOGOUT ---
    pub async fn logout(&self, refresh_token: &str) -> Result<(), ServiceError> {
        let hash = hash_token(refresh_token);
        let found = self.auth_repo.revoke_token(&hash).await?;
        if !found {
            return Err(ServiceError::InvalidCredentials); // Ou NotFound
        }
        Ok(())
    }

    // --- FORGOT PASSWORD ---
    pub async fn forgot_password(&self, email: &Email) -> Result<(), ServiceError> {
        // Sempre retorna Ok para não vazar info, mas loga internamente
        let user = self.user_repo.find_by_email(email).await?;

        if let Some(user) = user {
            let token = self
                .jwt_service
                .generate_token(user.id, TokenType::PasswordReset, RESET_TOKEN_EXPIRY)
                .map_err(|e| ServiceError::Internal(e))?;

            // Fire and forget
            let _ = self
                .email_service
                .send_password_reset_email(email, &user.username, &token)
                .await;
        }

        Ok(())
    }

    // --- RESET PASSWORD ---
    pub async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        // 1. Validar token
        let claims = self
            .jwt_service
            .verify_token(token, TokenType::PasswordReset)
            .map_err(|_| ServiceError::InvalidCredentials)?; // "Token inválido/expirado"

        let user_id = claims.sub;

        // 2. Hash senha
        let password_hash =
            hash_password(new_password).map_err(|e| ServiceError::Internal(anyhow::anyhow!(e)))?;

        // 3. Atualizar senha e revogar sessões
        self.user_repo
            .update_password(user_id, &password_hash)
            .await?;
        self.auth_repo.revoke_all_user_tokens(user_id).await?;

        Ok(())
    }

    // --- HELPER PRIVADO ---
    async fn generate_and_save_tokens(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<(String, String), ServiceError> {
        let access_token = self
            .jwt_service
            .generate_token(user_id, TokenType::Access, ACCESS_TOKEN_EXPIRY)
            .map_err(|e| ServiceError::Internal(e))?;

        let refresh_token_raw = uuid::Uuid::new_v4().to_string();
        let refresh_hash = hash_token(&refresh_token_raw);
        let family_id = uuid::Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);

        self.auth_repo
            .save_refresh_token(user_id, &refresh_hash, family_id, expires_at)
            .await?;

        Ok((access_token, refresh_token_raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use domain::models::{RefreshToken, UserDto, UserDtoExtended};
    use domain::value_objects::{Email, Username};
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    // 1. Mock User Repo
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
            async fn create(&self, username: &Username, email: &Email, password_hash: &str) -> Result<UserDto, domain::errors::RepositoryError>;
            async fn update_username(&self, id: Uuid, new_username: &Username) -> Result<(), domain::errors::RepositoryError>;
            async fn update_email(&self, id: Uuid, new_email: &Email) -> Result<(), domain::errors::RepositoryError>;
            async fn update_password(&self, id: Uuid, new_password_hash: &str) -> Result<(), domain::errors::RepositoryError>;
            async fn update_role(&self, id: Uuid, new_role: &str) -> Result<(), domain::errors::RepositoryError>;
            async fn mark_email_unverified(&self, id: Uuid) -> Result<(), domain::errors::RepositoryError>;
            async fn delete(&self, id: Uuid) -> Result<bool, domain::errors::RepositoryError>;
            async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<UserDto>, i64), domain::errors::RepositoryError>;
        }
    }

    // 2. Mock Auth Repo (NOVO)
    mock! {
        pub AuthRepo {}
        #[async_trait::async_trait]
        impl AuthRepositoryPort for AuthRepo {
            async fn save_refresh_token(&self, user_id: Uuid, token_hash: &str, family_id: Uuid, expires_at: DateTime<Utc>) -> Result<(), domain::errors::RepositoryError>;
            async fn find_token_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>, domain::errors::RepositoryError>;
            async fn rotate_refresh_token(&self, old_token_id: Uuid, new_token_hash: &str, new_expires_at: DateTime<Utc>, family_id: Uuid, parent_hash: &str, user_id: Uuid) -> Result<(), domain::errors::RepositoryError>;
            async fn revoke_token_family(&self, family_id: Uuid) -> Result<(), domain::errors::RepositoryError>;
            async fn revoke_token(&self, token_hash: &str) -> Result<bool, domain::errors::RepositoryError>;
            async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), domain::errors::RepositoryError>;
        }
    }

    // 3. Mock Email Repo
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

    fn create_test_jwt_service() -> Arc<JwtService> {
        let private_pem = b"-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIL/Ue/jT2+VqQ3+X2+VqQ3+X2+VqQ3+X2+VqQ3+X2+Vq\n-----END PRIVATE KEY-----";
        let public_pem = b"-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEA/9R7+NPb5WpDf5fb5WpDf5fb5WpDf5fb5WpDf5fb5Wo=\n-----END PUBLIC KEY-----";
        Arc::new(JwtService::new(private_pem, public_pem).unwrap())
    }

    #[tokio::test]
    async fn test_register_success() {
        let mut mock_user_repo = MockUserRepo::new();
        let mut mock_auth_repo = MockAuthRepo::new(); // Novo Mock
        let mut mock_email = MockEmailService::new();
        let jwt_service = create_test_jwt_service();

        let username = Username::try_from("newuser").unwrap();
        let email = Email::try_from("test@example.com").unwrap();
        let password = "SecurePass123!";

        mock_user_repo
            .expect_exists_by_email()
            .returning(|_| Ok(false));
        mock_user_repo
            .expect_exists_by_username()
            .returning(|_| Ok(false));

        let created_user = UserDto {
            id: Uuid::new_v4(),
            username: username.clone(),
            email: email.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let created_user_clone = created_user.clone();

        mock_user_repo
            .expect_create()
            .returning(move |_, _, _| Ok(created_user_clone.clone()));

        // Expectativa de salvar token
        mock_auth_repo
            .expect_save_refresh_token()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        mock_email
            .expect_send_verification_email()
            .returning(|_, _, _| Ok(()));
        mock_email
            .expect_send_welcome_email()
            .returning(|_, _| Ok(()));

        let service = AuthService::new(
            Arc::new(mock_user_repo),
            Arc::new(mock_auth_repo), // Injeção
            Arc::new(mock_email),
            jwt_service,
        );

        let payload = RegisterPayload {
            username,
            email,
            password: password.to_string(),
        };
        let result = service.register_user(payload).await;

        assert!(result.is_ok());
    }
}
