//! At-rest encryption for host credentials (passwords, SSH key passphrases). This is an
//! interim step ahead of the full Argon2id-derived vault key hierarchy planned for Fase 1
//! (`prd-v2.md` §Vault): the AES-256-GCM key here is derived from the same local key that
//! already keys SQLCipher (`vault::load_or_create_local_key`), domain-separated via SHA-256
//! so the two layers never share raw key material. When the real master-password-derived
//! vault lands, this module's key derivation is the one line that needs to change — callers
//! (`store::save_host_in`, `store::load_host_for_connect`) stay the same.

use crate::error::{CatermError, VaultError};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::Engine;
use sha2::{Digest, Sha256};

const DOMAIN: &[u8] = b"caframework-host-secret-v1";
const PREFIX: &str = "gcm1:";

fn derive_key(local_key_hex: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(local_key_hex.as_bytes());
    hasher.finalize().into()
}

/// Encrypts `plaintext` with a key derived from the local vault key. Returns
/// `"gcm1:" + base64(nonce || ciphertext)`, safe to store as a `TEXT` column.
pub fn encrypt(local_key_hex: &str, plaintext: &str) -> Result<String, CatermError> {
    let key = derive_key(local_key_hex);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!(
            "gagal inisialisasi cipher: {e}"
        )))
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!("gagal enkripsi secret: {e}")))
    })?;

    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

/// Decrypts a blob produced by [`encrypt`].
pub fn encrypt_bytes(key: &[u8; 32], plaintext: &[u8]) -> Result<String, CatermError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!(
            "gagal inisialisasi cipher: {e}"
        )))
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!("gagal enkripsi secret: {e}")))
    })?;

    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

pub fn decrypt_bytes(key: &[u8; 32], encoded: &str) -> Result<Vec<u8>, CatermError> {
    let payload = encoded
        .strip_prefix(PREFIX)
        .ok_or_else(|| CatermError::Vault(VaultError::Generic("unknown secret format".into())))?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| {
            CatermError::Vault(VaultError::Generic(format!("corrupt secret (base64): {e}")))
        })?;
    if raw.len() < 12 {
        return Err(CatermError::Vault(VaultError::Generic(
            "corrupt secret (payload too short)".into(),
        )));
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!(
            "failed to initialize cipher: {e}"
        )))
    })?;
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| {
            CatermError::Vault(VaultError::Generic(format!(
                "failed to decrypt secret: {e}"
            )))
        })
}
pub fn decrypt(local_key_hex: &str, encoded: &str) -> Result<String, CatermError> {
    let payload = encoded
        .strip_prefix(PREFIX)
        .ok_or_else(|| CatermError::Vault(VaultError::Generic("unknown secret format".into())))?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| {
            CatermError::Vault(VaultError::Generic(format!("secret rusak (base64): {e}")))
        })?;
    if raw.len() < 12 {
        return Err(CatermError::Vault(VaultError::Generic(
            "secret rusak (terlalu pendek)".into(),
        )));
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let key = derive_key(local_key_hex);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!(
            "gagal inisialisasi cipher: {e}"
        )))
    })?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|e| {
            CatermError::Vault(VaultError::Generic(format!("gagal dekripsi secret: {e}")))
        })?;
    String::from_utf8(plaintext)
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("secret bukan utf-8: {e}"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const KEY_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn roundtrips() {
        let enc = encrypt(KEY_A, "hunter2").expect("encrypt gagal");
        assert_ne!(enc, "hunter2");
        assert!(enc.starts_with(PREFIX));
        assert_eq!(decrypt(KEY_A, &enc).expect("decrypt gagal"), "hunter2");
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let enc = encrypt(KEY_A, "hunter2").expect("encrypt gagal");
        assert!(decrypt(KEY_B, &enc).is_err());
    }

    #[test]
    fn garbage_input_is_rejected_not_panicking() {
        assert!(decrypt(KEY_A, "not-a-real-blob").is_err());
        assert!(decrypt(KEY_A, "gcm1:not-base64!!").is_err());
    }

    #[test]
    fn two_encryptions_of_same_plaintext_differ() {
        // Nonce must be random per call — same plaintext must not produce identical ciphertext.
        let a = encrypt(KEY_A, "hunter2").expect("encrypt gagal");
        let b = encrypt(KEY_A, "hunter2").expect("encrypt gagal");
        assert_ne!(a, b);
    }
}
