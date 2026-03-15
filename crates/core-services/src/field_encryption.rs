//! Field-level encryption for sensitive data at rest.
//!
//! - AES-256-GCM with random nonce for confidentiality
//! - HMAC-SHA256 blind index for deterministic lookups (emails, etc.)
//! - Key parsing from hex-encoded environment variable

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

const NONCE_SIZE: usize = 12;

/// Encrypts `plaintext` using AES-256-GCM with a random 96-bit nonce.
/// Returns a base64url-encoded string of the form `nonce || ciphertext`.
pub fn encrypt_field(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Failed to initialise cipher: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(combined))
}

/// Decrypts a value produced by [`encrypt_field`].
/// Returns an error if the key is wrong or the data is corrupted.
pub fn decrypt_field(ciphertext_b64: &str, key: &[u8; 32]) -> Result<String> {
    let combined = URL_SAFE_NO_PAD
        .decode(ciphertext_b64)
        .map_err(|_| anyhow::anyhow!("Failed to base64-decode ciphertext"))?;

    if combined.len() < NONCE_SIZE {
        bail!("Ciphertext is too short to be valid");
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Failed to initialise cipher: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong key or corrupted data"))?;

    String::from_utf8(plaintext)
        .map_err(|_| anyhow::anyhow!("Decrypted value is not valid UTF-8"))
}

/// Produces a deterministic HMAC-SHA256 blind index for searchable encrypted
/// fields (e.g. email). The value is lowercased before hashing so lookups are
/// case-insensitive.
pub fn blind_index(value: &str, key: &[u8; 32]) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(value.to_lowercase().as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Parses a 64-character hex string into a 32-byte AES key.
/// The expected env var is `WS_FIELD_ENCRYPTION_KEY`.
pub fn parse_key(hex_key: &str) -> Result<[u8; 32]> {
    let bytes =
        hex::decode(hex_key.trim()).map_err(|_| {
            anyhow::anyhow!("WS_FIELD_ENCRYPTION_KEY must be a 64-character hex string")
        })?;
    if bytes.len() != 32 {
        bail!(
            "WS_FIELD_ENCRYPTION_KEY must decode to exactly 32 bytes (got {})",
            bytes.len()
        );
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut k);
        k
    }

    #[test]
    fn roundtrip() {
        let key = random_key();
        let plaintext = "test@example.com";
        let enc = encrypt_field(plaintext, &key).unwrap();
        let dec = decrypt_field(&enc, &key).unwrap();
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn different_nonces_per_call() {
        let key = random_key();
        let enc1 = encrypt_field("same", &key).unwrap();
        let enc2 = encrypt_field("same", &key).unwrap();
        assert_ne!(enc1, enc2); // random nonce
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = random_key();
        let key2 = random_key();
        let enc = encrypt_field("secret", &key1).unwrap();
        assert!(decrypt_field(&enc, &key2).is_err());
    }

    #[test]
    fn blind_index_is_case_insensitive() {
        let key = random_key();
        assert_eq!(
            blind_index("User@Example.COM", &key),
            blind_index("user@example.com", &key)
        );
    }
}
