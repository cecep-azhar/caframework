//! Token validation and parsing for GCC Billing Hub entitlement tokens.

use crate::error::{CatermError, VaultError};
use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitlementPayload {
    pub v: u32,
    pub kid: String,
    pub account_id: String,
    pub product_code: String,
    pub device_id: String,
    pub tier: String,         // "free" | "pro"
    pub billing_type: String, // "free" | "monthly" | "yearly" | "lifetime"
    pub status: String,       // "active" | "grace" | "expired" | "revoked"
    pub expires_at: Option<String>,
    pub grace_until: Option<String>,
    pub issued_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwsHeader {
    pub alg: String,
    pub typ: String,
    pub kid: String,
}

/// Verifies a compact JWS token (header.payload.signature) using the provided public keys map.
pub fn verify_and_parse_token(
    token: &str,
    public_keys: &[(String, String)], // (kid, base64_ed25519_pubkey)
    expected_product_code: &str,
    expected_device_id: &str,
) -> Result<EntitlementPayload, CatermError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(CatermError::Vault(VaultError::Generic(
            "invalid token format: must be compact JWS with 3 parts".into(),
        )));
    }

    let header_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid header base64: {e}"))))?;
    let header: JwsHeader = serde_json::from_slice(&header_bytes)
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid header JSON: {e}"))))?;

    if header.alg != "EdDSA" {
        return Err(CatermError::Vault(VaultError::Generic(
            "unsupported token algorithm: expected EdDSA".into(),
        )));
    }

    // Find matching public key by kid
    let pub_key_entry = public_keys
        .iter()
        .find(|(kid, _)| kid == &header.kid)
        .ok_or_else(|| {
            CatermError::Vault(VaultError::Generic(format!(
                "unknown key ID (kid): {}",
                header.kid
            )))
        })?;

    let pub_key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&pub_key_entry.1)
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid public key base64: {e}"))))?;

    if pub_key_bytes.len() != 32 {
        return Err(CatermError::Vault(VaultError::Generic(
            "invalid public key length: expected 32 bytes".into(),
        )));
    }

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&pub_key_bytes);
    let verifying_key = VerifyingKey::from_bytes(&key_arr)
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid verifying key: {e}"))))?;

    let sig_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid signature base64: {e}"))))?;

    if sig_bytes.len() != 64 {
        return Err(CatermError::Vault(VaultError::Generic(
            "invalid signature length: expected 64 bytes".into(),
        )));
    }

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    verifying_key
        .verify_strict(signing_input.as_bytes(), &signature)
        .map_err(|_| {
            CatermError::Vault(VaultError::Generic("token signature verification failed".into()))
        })?;

    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid payload base64: {e}"))))?;
    let payload: EntitlementPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid payload JSON: {e}"))))?;

    if payload.product_code != expected_product_code {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "product code mismatch: expected {}, got {}",
            expected_product_code, payload.product_code
        ))));
    }

    if payload.device_id != expected_device_id {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "device ID mismatch: expected {}, got {}",
            expected_device_id, payload.device_id
        ))));
    }

    Ok(payload)
}
