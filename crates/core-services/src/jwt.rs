use anyhow::{Context, Result};
use chrono::Duration;
use domain::models::{Claims, MfaChallengeClaims, TokenType}; // Importe MfaChallengeClaims
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: Arc<EncodingKey>,
    decoding_keys: Arc<Vec<DecodingKey>>,
}

impl JwtService {
    pub fn new(private_pem: &[u8], public_pem: &[u8]) -> Result<Self> {
        let encoding_key = EncodingKey::from_ed_pem(private_pem)
            .context("Falha ao carregar chave privada EdDSA")?;

        let decoding_key = DecodingKey::from_ed_pem(public_pem)
            .context("Falha ao carregar chave pública EdDSA")?;

        Ok(Self {
            encoding_key: Arc::new(encoding_key),
            decoding_keys: Arc::new(vec![decoding_key]),
        })
    }

    // ... (métodos existentes: add_rotation_key, generate_token, verify_token)

    pub fn generate_token(
        &self,
        user_id: Uuid,
        username: &str,
        token_type: TokenType,
        expires_in_seconds: i64,
    ) -> Result<String> {
        let now = chrono::Utc::now();
        let expiration = now + Duration::seconds(expires_in_seconds);

        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            token_type,
        };

        let header = Header::new(Algorithm::EdDSA);

        encode(&header, &claims, &self.encoding_key).context("Falha ao codificar e assinar JWT")
    }

    pub fn verify_token(&self, token: &str, expected_type: TokenType) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.leeway = 5;

        let mut last_error = anyhow::anyhow!("Nenhuma chave configurada");

        for key in self.decoding_keys.iter() {
            match decode::<Claims>(token, key, &validation) {
                Ok(token_data) => {
                    if token_data.claims.token_type != expected_type {
                        return Err(anyhow::anyhow!("Tipo de token inválido"));
                    }
                    return Ok(token_data.claims);
                }
                Err(e) => last_error = e.into(),
            }
        }

        Err(last_error).context("Token inválido ou expirado")
    }

    // --- NOVOS MÉTODOS PARA MFA ---

    /// Gera um token de desafio MFA
    pub fn generate_mfa_token(&self, user_id: Uuid, expires_in_seconds: i64) -> Result<String> {
        let now = chrono::Utc::now();
        let expiration = now + Duration::seconds(expires_in_seconds);

        let claims = MfaChallengeClaims {
            sub: user_id,
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            token_type: "mfa_challenge".to_string(),
        };

        let header = Header::new(Algorithm::EdDSA);
        encode(&header, &claims, &self.encoding_key).context("Falha ao gerar token MFA")
    }

    /// Verifica um token de desafio MFA
    pub fn verify_mfa_token(&self, token: &str) -> Result<MfaChallengeClaims> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.leeway = 5;

        let mut last_error = anyhow::anyhow!("Nenhuma chave configurada");

        for key in self.decoding_keys.iter() {
            match decode::<MfaChallengeClaims>(token, key, &validation) {
                Ok(token_data) => {
                    if token_data.claims.token_type != "mfa_challenge" {
                        return Err(anyhow::anyhow!(
                            "Tipo de token inválido (esperado mfa_challenge)"
                        ));
                    }
                    return Ok(token_data.claims);
                }
                Err(e) => last_error = e.into(),
            }
        }
        Err(last_error).context("Token MFA inválido ou expirado")
    }
}
