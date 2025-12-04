use crate::errors::ServiceError;
use chrono::{Duration, Utc};
use core_services::jwt::JwtService;
use domain::models::{MfaSetupCompleteResponse, MfaSetupResponse, MfaVerifyResponse, TokenType};
use domain::ports::{AuthRepositoryPort, EmailServicePort, MfaRepositoryPort, UserRepositoryPort};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::info;
use uuid::Uuid;

// Helper local se não estiver exposto no core
fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub struct MfaService {
    mfa_repo: Arc<dyn MfaRepositoryPort>,
    user_repo: Arc<dyn UserRepositoryPort>,
    auth_repo: Arc<dyn AuthRepositoryPort>,
    email_service: Arc<dyn EmailServicePort>,
    jwt_service: Arc<JwtService>,
}

impl MfaService {
    pub fn new(
        mfa_repo: Arc<dyn MfaRepositoryPort>,
        user_repo: Arc<dyn UserRepositoryPort>,
        auth_repo: Arc<dyn AuthRepositoryPort>,
        email_service: Arc<dyn EmailServicePort>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            mfa_repo,
            user_repo,
            auth_repo,
            email_service,
            jwt_service,
        }
    }

    /// Inicia o setup: Gera segredo e retorna dados para QR Code
    pub async fn initiate_setup(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<MfaSetupResponse, ServiceError> {
        // Gerar segredo
        let secret = Secret::generate_secret().to_encoded().to_string();

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.as_bytes().to_vec(),
            Some("WaterSwamp".to_string()),
            username.to_string(),
        )
        .map_err(|e| ServiceError::Internal(anyhow::anyhow!(e)))?;

        let qr_code = totp
            .get_qr_base64()
            .map_err(|e| ServiceError::Internal(anyhow::anyhow!(e)))?;

        // Gerar token de setup temporário
        let setup_token_raw = Uuid::new_v4().to_string();
        let setup_token_hash = hash_string(&setup_token_raw);
        let expires_at = Utc::now() + Duration::minutes(10); // 10 min para escanear

        // Salvar estado temporário
        self.mfa_repo
            .save_setup_token(user_id, &secret, &setup_token_hash, expires_at)
            .await
            .map_err(ServiceError::Repository)?;

        Ok(MfaSetupResponse {
            secret,
            qr_code_url: format!("data:image/png;base64,{}", qr_code),
            setup_token: setup_token_raw,
        })
    }

    /// Finaliza o setup: Valida código e ativa MFA
    pub async fn complete_setup(
        &self,
        setup_token: &str,
        code: &str,
    ) -> Result<MfaSetupCompleteResponse, ServiceError> {
        let setup_hash = hash_string(setup_token);

        // 1. Buscar token de setup
        let (user_id, secret) = self
            .mfa_repo
            .find_setup_token(&setup_hash)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or(ServiceError::InvalidCredentials)?; // "Token expirado ou inválido"

        // 2. Validar TOTP
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.as_bytes().to_vec(),
            None,
            "".to_string(),
        )
        .unwrap();

        if !totp.check_current(code).unwrap_or(false) {
            return Err(ServiceError::InvalidCredentials); // "Código inválido"
        }

        // 3. Ativar MFA
        self.mfa_repo
            .enable_mfa(user_id, &secret)
            .await
            .map_err(ServiceError::Repository)?;

        // 4. Gerar códigos de backup
        let backup_codes: Vec<String> = (0..10)
            .map(|_| Uuid::new_v4().to_string().replace("-", "")[0..8].to_string())
            .collect();
        let hashed_codes: Vec<String> = backup_codes.iter().map(|c| hash_string(c)).collect();

        self.mfa_repo
            .save_backup_codes(user_id, &hashed_codes)
            .await
            .map_err(ServiceError::Repository)?;

        // 5. Notificar
        let user = self
            .user_repo
            .find_extended_by_id(user_id)
            .await
            .map_err(ServiceError::Repository)?
            .unwrap();
        let _ = self
            .email_service
            .send_mfa_enabled_email(&user.email, &user.username)
            .await;

        Ok(MfaSetupCompleteResponse {
            enabled: true,
            backup_codes,
            message: "MFA ativado com sucesso".to_string(),
        })
    }

    /// Verifica MFA no Login (TOTP ou Backup Code)
    pub async fn verify_login(
        &self,
        mfa_token: &str,
        code: &str,
    ) -> Result<MfaVerifyResponse, ServiceError> {
        // 1. Validar token MFA (JWT temporário)
        let claims = self
            .jwt_service
            .verify_mfa_token(mfa_token)
            .map_err(|_| ServiceError::InvalidCredentials)?;
        let user_id = claims.sub;

        // 2. Tentar como TOTP
        let secret_opt = self
            .mfa_repo
            .get_mfa_secret(user_id)
            .await
            .map_err(ServiceError::Repository)?;

        let mut valid = false;
        let mut backup_used = false;

        if let Some(secret) = secret_opt.clone() {
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret.as_bytes().to_vec(),
                None,
                "".to_string(),
            )
            .unwrap();

            if totp.check_current(code).unwrap_or(false) {
                valid = true;
            }
        }

        // 3. Se falhou TOTP, tentar Backup Code
        if !valid {
            let code_hash = hash_string(code);
            if self
                .mfa_repo
                .verify_and_consume_backup_code(user_id, &code_hash)
                .await
                .map_err(ServiceError::Repository)?
            {
                valid = true;
                backup_used = true;
            }
        }

        if !valid {
            return Err(ServiceError::InvalidCredentials);
        }

        // 4. Sucesso! Gerar tokens finais
        let access_token = self
            .jwt_service
            .generate_token(user_id, TokenType::Access, 3600)
            .map_err(|e| ServiceError::Internal(e))?;
        // Nota: Refresh token logic deveria ser reusada do AuthService,
        // ou duplicada aqui, ou AuthService expor um método publico 'generate_tokens'.
        // Simplificando com geração direta:
        let refresh_token = Uuid::new_v4().to_string();
        let refresh_hash = hash_string(&refresh_token);
        // Precisaria chamar auth_repo.save_refresh_token...
        // Para evitar duplicação, o ideal seria ter um TokenService separado.
        // Aqui vou assumir que você injeta a lógica ou copia por brevidade:
        let family_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(7);
        self.auth_repo
            .save_refresh_token(user_id, &refresh_hash, family_id, expires_at)
            .await
            .map_err(ServiceError::Repository)?;

        info!(
            "Validando TOTP para user: '{}' com secret: '{:?}'",
            user_id, secret_opt
        );

        Ok(MfaVerifyResponse::new(
            access_token,
            refresh_token,
            3600,
            backup_used,
        ))
    }
}
