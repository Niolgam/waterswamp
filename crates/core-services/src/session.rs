//! Session and Cookie Security Service
//!
//! Provides secure cookie handling with:
//! - HMAC signing for integrity
//! - AES-256-GCM encryption for confidentiality
//! - Key rotation support
//! - HttpOnly, Secure, SameSite cookie flags

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use cookie::{Cookie, SameSite};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::time::Duration;
use thiserror::Error;

/// Cookie configuration constants
pub mod config {
    use std::time::Duration;

    /// Session cookie name
    pub const SESSION_COOKIE_NAME: &str = "__Host-session";

    /// CSRF cookie name (readable by JavaScript for header submission)
    pub const CSRF_COOKIE_NAME: &str = "csrf_token";

    /// Session duration (24 hours)
    pub const SESSION_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

    /// Short session for "remember me" unchecked (browser session)
    pub const SHORT_SESSION_DURATION: Duration = Duration::from_secs(2 * 60 * 60);

    /// Extended session for "remember me" checked (30 days)
    pub const EXTENDED_SESSION_DURATION: Duration = Duration::from_secs(30 * 24 * 60 * 60);

    /// Sliding window extension (30 minutes of activity extends session)
    pub const SLIDING_WINDOW_MINUTES: i32 = 30;

    /// Session token length in bytes (256 bits)
    pub const SESSION_TOKEN_BYTES: usize = 32;

    /// CSRF token length in bytes (256 bits)
    pub const CSRF_TOKEN_BYTES: usize = 32;

    /// Key ID length for rotation
    pub const KEY_ID_LENGTH: usize = 8;
}

/// Errors that can occur during session/cookie operations
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Invalid session token")]
    InvalidToken,

    #[error("Session expired")]
    Expired,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Missing encryption key")]
    MissingKey,

    #[error("CSRF token mismatch")]
    CsrfMismatch,

    #[error("Cookie not found: {0}")]
    CookieNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Generates a cryptographically secure random token
pub fn generate_token(length: usize) -> Vec<u8> {
    let mut token = vec![0u8; length];
    rand::thread_rng().fill_bytes(&mut token);
    token
}

/// Generates a session token and returns it as a hex string
pub fn generate_session_token() -> String {
    let token = generate_token(config::SESSION_TOKEN_BYTES);
    hex::encode(token)
}

/// Generates a CSRF token and returns it as a hex string
pub fn generate_csrf_token() -> String {
    let token = generate_token(config::CSRF_TOKEN_BYTES);
    hex::encode(token)
}

/// Generates a key ID for key rotation
pub fn generate_key_id() -> String {
    let id = generate_token(config::KEY_ID_LENGTH);
    hex::encode(id)
}

/// Hashes a token using SHA-256 for secure storage
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Session cookie builder with all security flags
pub struct SecureCookieBuilder {
    name: String,
    value: String,
    max_age: Option<Duration>,
    path: String,
    domain: Option<String>,
    secure: bool,
    http_only: bool,
    same_site: SameSite,
}

impl SecureCookieBuilder {
    /// Creates a new secure cookie builder with safe defaults
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            max_age: None,
            path: "/".to_string(),
            domain: None,
            secure: true, // Default: HTTPS only
            http_only: true, // Default: No JavaScript access
            same_site: SameSite::Strict, // Default: Strict CSRF protection
        }
    }

    /// Sets the max age (expiration)
    pub fn max_age(mut self, duration: Duration) -> Self {
        self.max_age = Some(duration);
        self
    }

    /// Sets the path
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Sets the domain
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the Secure flag (HTTPS only)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Sets the HttpOnly flag (JavaScript cannot access)
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Sets the SameSite policy
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }

    /// Builds the cookie
    pub fn build(self) -> Cookie<'static> {
        let mut cookie = Cookie::build((self.name, self.value))
            .path(self.path)
            .secure(self.secure)
            .http_only(self.http_only)
            .same_site(self.same_site);

        if let Some(max_age) = self.max_age {
            cookie = cookie.max_age(cookie::time::Duration::seconds(max_age.as_secs() as i64));
        }

        if let Some(domain) = self.domain {
            cookie = cookie.domain(domain);
        }

        cookie.build()
    }
}

/// Builds a session cookie with all security flags set
pub fn build_session_cookie(session_token: &str, duration: Duration) -> Cookie<'static> {
    SecureCookieBuilder::new(config::SESSION_COOKIE_NAME, session_token.to_string())
        .max_age(duration)
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .build()
}

/// Builds a CSRF cookie (readable by JavaScript for header submission)
pub fn build_csrf_cookie(csrf_token: &str, duration: Duration) -> Cookie<'static> {
    SecureCookieBuilder::new(config::CSRF_COOKIE_NAME, csrf_token.to_string())
        .max_age(duration)
        .path("/")
        .secure(true)
        .http_only(false) // JavaScript needs to read this
        .same_site(SameSite::Strict)
        .build()
}

/// Builds a cookie to delete/expire an existing cookie
pub fn build_removal_cookie(name: &str) -> Cookie<'static> {
    Cookie::build((name.to_string(), ""))
        .path("/")
        .max_age(cookie::time::Duration::ZERO)
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .build()
}

/// HMAC-based signing for cookie values
pub mod signing {
    use super::*;
    use sha2::Sha256;

    /// Signs a value using HMAC-SHA256
    pub fn sign(value: &str, key: &[u8]) -> String {
        use sha2::digest::Mac;
        type HmacSha256 = hmac::Hmac<Sha256>;

        let mut mac =
            HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(value.as_bytes());
        let result = mac.finalize();
        let signature = result.into_bytes();

        // Format: value.signature
        format!("{}.{}", value, URL_SAFE_NO_PAD.encode(signature))
    }

    /// Verifies and extracts the original value from a signed cookie
    pub fn verify(signed_value: &str, key: &[u8]) -> Result<String, SessionError> {
        use sha2::digest::Mac;
        type HmacSha256 = hmac::Hmac<Sha256>;

        let parts: Vec<&str> = signed_value.rsplitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(SessionError::InvalidToken);
        }

        let signature_b64 = parts[0];
        let value = parts[1];

        let signature = URL_SAFE_NO_PAD
            .decode(signature_b64)
            .map_err(|_| SessionError::InvalidSignature)?;

        let mut mac =
            HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(value.as_bytes());

        mac.verify_slice(&signature)
            .map_err(|_| SessionError::InvalidSignature)?;

        Ok(value.to_string())
    }
}

/// AES-256-GCM encryption for sensitive cookie data
pub mod encryption {
    use super::*;
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    const NONCE_SIZE: usize = 12;

    /// Encrypts data using AES-256-GCM
    pub fn encrypt(plaintext: &str, key: &[u8]) -> Result<String, SessionError> {
        if key.len() != 32 {
            return Err(SessionError::MissingKey);
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| SessionError::Internal(format!("Failed to create cipher: {}", e)))?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| SessionError::Internal(format!("Encryption failed: {}", e)))?;

        // Format: nonce + ciphertext, base64 encoded
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(URL_SAFE_NO_PAD.encode(combined))
    }

    /// Decrypts data using AES-256-GCM
    pub fn decrypt(ciphertext_b64: &str, key: &[u8]) -> Result<String, SessionError> {
        if key.len() != 32 {
            return Err(SessionError::MissingKey);
        }

        let combined = URL_SAFE_NO_PAD
            .decode(ciphertext_b64)
            .map_err(|_| SessionError::DecryptionFailed)?;

        if combined.len() < NONCE_SIZE {
            return Err(SessionError::DecryptionFailed);
        }

        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| SessionError::Internal(format!("Failed to create cipher: {}", e)))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| SessionError::DecryptionFailed)?;

        String::from_utf8(plaintext).map_err(|_| SessionError::DecryptionFailed)
    }
}

/// Validates a CSRF token from request header against session
pub fn validate_csrf(header_token: &str, session_csrf_hash: &str) -> Result<(), SessionError> {
    let header_hash = hash_token(header_token);
    if header_hash != session_csrf_hash {
        return Err(SessionError::CsrfMismatch);
    }
    Ok(())
}

/// Generates new session keys (signing + encryption)
pub fn generate_session_keys() -> (String, Vec<u8>, Vec<u8>) {
    let key_id = generate_key_id();
    let signing_key = generate_token(32);
    let encryption_key = generate_token(32);
    (key_id, signing_key, encryption_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();

        assert_eq!(token1.len(), 64); // 32 bytes = 64 hex chars
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token_12345";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 = 64 hex chars
    }

    #[test]
    fn test_csrf_validation() {
        let csrf_token = generate_csrf_token();
        let csrf_hash = hash_token(&csrf_token);

        assert!(validate_csrf(&csrf_token, &csrf_hash).is_ok());
        assert!(validate_csrf("wrong_token", &csrf_hash).is_err());
    }

    #[test]
    fn test_signing() {
        let key = generate_token(32);
        let value = "session_token_123";

        let signed = signing::sign(value, &key);
        let verified = signing::verify(&signed, &key).unwrap();

        assert_eq!(verified, value);
    }

    #[test]
    fn test_signing_tamper_detection() {
        let key = generate_token(32);
        let value = "session_token_123";

        let signed = signing::sign(value, &key);
        let tampered = signed.replace("session", "hacked");

        assert!(signing::verify(&tampered, &key).is_err());
    }

    #[test]
    fn test_encryption() {
        let key = generate_token(32);
        let plaintext = "sensitive_access_token_jwt";

        let encrypted = encryption::encrypt(plaintext, &key).unwrap();
        let decrypted = encryption::decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
        assert_ne!(encrypted, plaintext);
    }

    #[test]
    fn test_encryption_different_nonces() {
        let key = generate_token(32);
        let plaintext = "same_content";

        let encrypted1 = encryption::encrypt(plaintext, &key).unwrap();
        let encrypted2 = encryption::encrypt(plaintext, &key).unwrap();

        // Same plaintext should produce different ciphertext (random nonce)
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same value
        assert_eq!(encryption::decrypt(&encrypted1, &key).unwrap(), plaintext);
        assert_eq!(encryption::decrypt(&encrypted2, &key).unwrap(), plaintext);
    }
}
