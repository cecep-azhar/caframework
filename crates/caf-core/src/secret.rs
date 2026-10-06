//! At-rest encryption for host credentials (passwords, SSH key passphrases).
//! This module provides AES‑256‑GCM encryption/decryption using a key derived from
//! the local vault key. The plaintext is wrapped in `secrecy::SecretString` to ensure
//! memory is zero‑ed when dropped.

use crate::error::{CafError, VaultError};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::Engine;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

// Argon2 utilities for password hashing (used elsewhere in this module).
use argon2::{
    Argon2, Params,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng as ArgonOsRng,
    },
};

const DOMAIN: &[u8] = b"caframework-host-secret-v1";
const PREFIX: &str = "gcm1:";

/// Derive a 32‑byte AES key from the local vault key (hex string).
fn derive_key(local_key_hex: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(local_key_hex.as_bytes());
    // The result is a fresh array that we will zero‑ize after use.
    hasher.finalize().into()
}

/// Encrypt a secret string using the derived key.
/// Returns a base64‑encoded `gcm1:<nonce||ciphertext>`.
pub fn encrypt(local_key_hex: &str, plaintext: &SecretString) -> Result<String, CafError> {
    // Derive and immediately protect the key.
    let mut key = derive_key(local_key_hex);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
        key.zeroize();
        CafError::Vault(VaultError::Generic(format!(
            "gagal inisialisasi cipher: {e}"
        )))
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.expose_secret().as_bytes())
        .map_err(|e| {
            key.zeroize();
            CafError::Vault(VaultError::Generic(format!("gagal enkripsi secret: {e}")))
        })?;
    // Zero the key now that encryption is done.
    key.zeroize();
    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

/// Encrypt raw bytes with a pre‑derived key (used for backup encryption).
pub fn encrypt_bytes(key: &[u8; 32], plaintext: &[u8]) -> Result<String, CafError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        CafError::Vault(VaultError::Generic(format!(
            "gagal inisialisasi cipher: {e}"
        )))
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|e| {
        CafError::Vault(VaultError::Generic(format!("gagal enkripsi secret: {e}")))
    })?;
    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

/// Decrypt a blob produced by `encrypt_bytes`.
pub fn decrypt_bytes(key: &[u8; 32], encoded: &str) -> Result<Vec<u8>, CafError> {
    let payload = encoded
        .strip_prefix(PREFIX)
        .ok_or_else(|| CafError::Vault(VaultError::Generic("unknown secret format".into())))?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| {
            CafError::Vault(VaultError::Generic(format!("corrupt secret (base64): {e}")))
        })?;
    if raw.len() < 12 {
        return Err(CafError::Vault(VaultError::Generic(
            "corrupt secret (payload too short)".into(),
        )));
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        CafError::Vault(VaultError::Generic(format!(
            "failed to initialize cipher: {e}"
        )))
    })?;
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| {
            CafError::Vault(VaultError::Generic(format!(
                "failed to decrypt secret: {e}"
            )))
        })
}

/// Decrypt a blob produced by `encrypt` and return a `SecretString`.
pub fn decrypt(local_key_hex: &str, encoded: &str) -> Result<SecretString, CafError> {
    let payload = encoded
        .strip_prefix(PREFIX)
        .ok_or_else(|| CafError::Vault(VaultError::Generic("unknown secret format".into())))?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| {
            CafError::Vault(VaultError::Generic(format!("secret rusak (base64): {e}")))
        })?;
    if raw.len() < 12 {
        return Err(CafError::Vault(VaultError::Generic(
            "secret rusak (terlalu pendek)".into(),
        )));
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let mut key = derive_key(local_key_hex);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
        key.zeroize();
        CafError::Vault(VaultError::Generic(format!(
            "gagal inisialisasi cipher: {e}"
        )))
    })?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| {
            key.zeroize();
            CafError::Vault(VaultError::Generic(format!("gagal dekripsi secret: {e}")))
        })?;
    key.zeroize();
    let s = String::from_utf8(plaintext)
        .map_err(|e| CafError::Vault(VaultError::Generic(format!("secret bukan utf-8: {e}"))))?;
    Ok(SecretString::from(s))
}

// ---------------------------------------------------------------------------
// Password hashing utilities (unchanged semantics, but now with proper imports).
// ---------------------------------------------------------------------------

/// Hash a password or passphrase using Argon2id with strong parameters.
pub fn hash_password(password: &str) -> Result<String, CafError> {
    let params = Params::new(65536, 3, 4, Some(32)).map_err(|e| {
        CafError::Vault(VaultError::Generic(format!("invalid argon2 params: {e}")))
    })?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = SaltString::generate(&mut ArgonOsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            CafError::Vault(VaultError::Generic(format!("failed to hash password: {e}")))
        })?
        .to_string();
    Ok(password_hash)
}

/// Verify a password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CafError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        CafError::Vault(VaultError::Generic(format!("invalid hash format: {e}")))
    })?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    const KEY_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const KEY_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn roundtrips() {
        let ss = SecretString::from("hunter2".to_string());
        let enc = encrypt(KEY_A, &ss).expect("encrypt gagal");
        assert_ne!(enc, "hunter2");
        assert!(enc.starts_with(PREFIX));
        let dec = decrypt(KEY_A, &enc).expect("decrypt gagal");
        assert_eq!(dec.expose_secret(), "hunter2");
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let ss = SecretString::from("hunter2".to_string());
        let enc = encrypt(KEY_A, &ss).expect("encrypt gagal");
        assert!(decrypt(KEY_B, &enc).is_err());
    }

    #[test]
    fn garbage_input_is_rejected_not_panicking() {
        assert!(decrypt(KEY_A, "not-a-real-blob").is_err());
        assert!(decrypt(KEY_A, "gcm1:not-base64!!").is_err());
    }

    #[test]
    fn two_encryptions_of_same_plaintext_differ() {
        let ss = SecretString::from("hunter2".to_string());
        let a = encrypt(KEY_A, &ss).expect("encrypt gagal");
        let b = encrypt(KEY_A, &ss).expect("encrypt gagal");
        assert_ne!(a, b);
    }
}
