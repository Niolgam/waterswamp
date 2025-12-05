//! Funções auxiliares reutilizáveis.
//!
//! Este módulo contém funções helper usadas em toda a aplicação,
//! como hashing de tokens, geração de códigos de backup, etc.

use crate::infra::{errors::AppError, state::AppState};
use crate::utils::constants::*;
use anyhow::Context;
// Importa constantes
use chrono::{Duration, Utc};
use domain::models::TokenType;
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

// =============================================================================
// TOKEN HASHING
// =============================================================================

/// Gera um hash SHA-256 de um token.
///
/// Usado para armazenar refresh tokens de forma segura no banco de dados.
/// Nunca armazenamos tokens em texto plano por segurança.
///
/// # Exemplo
///
/// ```rust,ignore
/// let token = "some-refresh-token-uuid";
/// let hash = hash_token(token);
/// // hash = "a1b2c3d4e5f6..."
/// ```
///
/// # Notas
///
/// - O hash é uma string hexadecimal de 64 caracteres
/// - É unidirecional (não pode ser revertido)
/// - Dois tokens iguais sempre geram o mesmo hash
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

// =============================================================================
// BACKUP CODES
// =============================================================================

/// Gera códigos de backup para MFA.
///
/// Retorna uma tupla com:
/// - `Vec<String>` - Códigos em texto plano (para mostrar ao usuário)
/// - `Vec<String>` - Códigos hasheados (para armazenar no banco)
///
/// # Parâmetros
///
/// - `count` - Número de códigos a gerar (padrão: 10)
/// - `length` - Comprimento de cada código (padrão: 12)
///
/// # Formato dos Códigos
///
/// - Apenas caracteres maiúsculos e números
/// - Sem caracteres ambíguos (0, O, I, l, 1)
/// - Exemplo: `ABCD1234EFGH`
///
/// # Exemplo
///
/// ```rust,ignore
/// let (plain, hashed) = generate_backup_codes(10, 12);
/// // plain[0] = "ABCD1234EFGH"
/// // hashed[0] = "a1b2c3d4e5f6..." (SHA-256)
/// ```
///
/// # Segurança
///
/// - Usa `rand::thread_rng()` para geração criptograficamente segura
/// - Códigos hasheados com SHA-256 antes de armazenar
/// - Códigos em texto plano são mostrados **apenas uma vez** ao usuário
pub fn generate_backup_codes(count: usize, length: usize) -> (Vec<String>, Vec<String>) {
    let mut rng = rand::thread_rng();
    let mut plain_codes = Vec::with_capacity(count);
    let mut hashed_codes = Vec::with_capacity(count);

    for _ in 0..count {
        // Gera código aleatório
        let code: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..BACKUP_CODE_CHARSET.len());
                BACKUP_CODE_CHARSET[idx] as char
            })
            .collect();

        // Armazena código em texto plano (para o usuário)
        plain_codes.push(code.clone());

        // Armazena código hasheado (para o banco)
        hashed_codes.push(hash_backup_code(&code));
    }

    (plain_codes, hashed_codes)
}

/// Wrapper para gerar códigos de backup com valores padrão.
///
/// Usa as constantes BACKUP_CODES_COUNT e BACKUP_CODE_LENGTH.
pub fn generate_backup_codes_default() -> (Vec<String>, Vec<String>) {
    generate_backup_codes(BACKUP_CODES_COUNT, BACKUP_CODE_LENGTH)
}

/// Gera um hash SHA-256 de um backup code.
///
/// Converte o código para maiúsculas antes de hashear para
/// garantir comparação case-insensitive.
///
/// # Exemplo
///
/// ```rust,ignore
/// let hash = hash_backup_code("ABCD1234EFGH");
/// // hash = "a1b2c3d4e5f6..."
/// ```
pub fn hash_backup_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.to_uppercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

// =============================================================================
// TOKEN GENERATION
// =============================================================================

/// Gera um par de tokens (access + refresh) para um usuário.
///
/// Esta é uma função auxiliar usada após login ou verificação MFA bem-sucedida.
///
/// # Retorno
///
/// - `Ok((access_token, refresh_token))` - Par de tokens JWT
/// - `Err(AppError)` - Erro ao gerar tokens ou salvar no banco
///
/// # Fluxo
///
/// 1. Gera access token (JWT) com expiração de 1 hora
/// 2. Gera refresh token (UUID) com expiração de 7 dias
/// 3. Faz hash do refresh token
/// 4. Salva refresh token hasheado no banco
/// 5. Retorna tokens em texto plano
///
/// # Exemplo
///
/// ```rust,ignore
/// let user_id = Uuid::new_v4();
/// let (access, refresh) = generate_tokens_helper(&state, user_id).await?;
///
/// // access = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9..."
/// // refresh = "123e4567-e89b-12d3-a456-426614174000"
/// ```
///
/// # Segurança
///
/// - Access token é JWT assinado (EdDSA)
/// - Refresh token é UUID aleatório
/// - Refresh token é hasheado (SHA-256) antes de armazenar
/// - Cada refresh token pertence a uma "família" (para detecção de roubo)
pub async fn generate_tokens_helper(
    state: &AppState,
    user_id: Uuid,
    username: &str,
) -> Result<(String, String), AppError> {
    let access_token = state
        .jwt_service
        .generate_token(user_id, username, TokenType::Access, ACCESS_TOKEN_EXPIRY_SECONDS)
        .map_err(|e| {
            tracing::error!("Erro ao gerar access token: {:?}", e);
            AppError::Anyhow(e)
        })?;

    // 2. Generate Refresh Token (Opaque)
    let refresh_token_raw = Uuid::new_v4().to_string();
    let refresh_token_hash = hash_token(&refresh_token_raw);
    let family_id = Uuid::new_v4();

    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY_SECONDS);

    // 3. Save Refresh Token
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family_id, parent_token_hash)
        VALUES ($1, $2, $3, $4, NULL)
        "#,
    )
    .bind(user_id)
    .bind(&refresh_token_hash)
    .bind(expires_at)
    .bind(family_id)
    .execute(&state.db_pool_auth)
    .await
    .context("Falha ao salvar refresh token inicial")?;

    Ok((access_token, refresh_token_raw))
}

// =============================================================================
// TESTES
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token() {
        let token = "test-token-123";
        let hash = hash_token(token);

        // Hash SHA-256 deve ter 64 caracteres hexadecimais
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Mesmo token sempre gera o mesmo hash
        let hash2 = hash_token(token);
        assert_eq!(hash, hash2);

        // Tokens diferentes geram hashes diferentes
        let hash3 = hash_token("different-token");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_backup_code() {
        let code = "ABCD1234EFGH";
        let hash = hash_backup_code(code);

        // Hash SHA-256 deve ter 64 caracteres hexadecimais
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Case-insensitive (mesma hash para maiúsculas e minúsculas)
        let hash_lower = hash_backup_code("abcd1234efgh");
        assert_eq!(hash, hash_lower);
    }

    #[test]
    fn test_generate_backup_codes() {
        let (plain, hashed) = generate_backup_codes(BACKUP_CODES_COUNT, BACKUP_CODE_LENGTH);

        // Deve gerar exatamente 10 códigos
        assert_eq!(plain.len(), BACKUP_CODES_COUNT);
        assert_eq!(hashed.len(), BACKUP_CODES_COUNT);

        // Cada código deve ter 12 caracteres
        for code in &plain {
            assert_eq!(code.len(), BACKUP_CODE_LENGTH);
            // Apenas caracteres válidos
            assert!(code
                .chars()
                .all(|c| BACKUP_CODE_CHARSET.contains(&(c as u8))));
        }

        // Códigos devem ser únicos
        let unique_plain: std::collections::HashSet<_> = plain.iter().collect();
        assert_eq!(unique_plain.len(), BACKUP_CODES_COUNT);

        // Hashes devem ser únicos
        let unique_hashed: std::collections::HashSet<_> = hashed.iter().collect();
        assert_eq!(unique_hashed.len(), BACKUP_CODES_COUNT);

        // Cada hash deve ter 64 caracteres
        for hash in &hashed {
            assert_eq!(hash.len(), 64);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_generate_backup_codes_default() {
        let (plain, hashed) = generate_backup_codes_default();

        assert_eq!(plain.len(), BACKUP_CODES_COUNT);
        assert_eq!(hashed.len(), BACKUP_CODES_COUNT);

        for code in &plain {
            assert_eq!(code.len(), BACKUP_CODE_LENGTH);
        }
    }

    #[test]
    fn test_backup_codes_no_ambiguous_chars() {
        let (plain, _) = generate_backup_codes(100, 12);

        // Caracteres ambíguos que NÃO devem aparecer
        let ambiguous = ['0', 'O', 'I', 'l', '1'];

        for code in &plain {
            for ambiguous_char in &ambiguous {
                assert!(
                    !code.contains(*ambiguous_char),
                    "Código '{}' contém caractere ambíguo '{}'",
                    code,
                    ambiguous_char
                );
            }
        }
    }

    #[test]
    fn test_backup_code_format() {
        let (plain, _) = generate_backup_codes(10, 12);

        for code in &plain {
            // Apenas maiúsculas e números
            assert!(code
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));

            // Sem espaços ou caracteres especiais
            assert!(!code.contains(' '));
            assert!(!code.contains('-'));
            assert!(!code.contains('_'));
        }
    }
}
