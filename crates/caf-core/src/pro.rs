//! Generic Pro Licensing & Account client.

use crate::error::CafError;
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProStatus {
    pub is_pro: bool,
    pub plan: String,
    pub email: Option<String>,
    pub expires_at: Option<String>,
}

/// Compiled-in GCC Ed25519 public key for verifying license tokens.
/// Aligned with production GCC signing key.
pub fn public_key(version: u32) -> Option<VerifyingKey> {
    let compiled = match version {
        1 => option_env!("CATERM_PRO_LICENSE_PUBKEY_V1").or(Some(
            "4aed277ac7b92ee58778d3c5233ebda741c652c753e3dae7127f367dbb58d92f",
        )),
        2 => option_env!("CATERM_PRO_LICENSE_PUBKEY_V2"),
        _ => None,
    };
    #[cfg(debug_assertions)]
    let runtime = std::env::var("CATERM_PRO_LICENSE_PUBKEY").ok();
    #[cfg(not(debug_assertions))]
    let runtime: Option<String> = None;

    let hex_key = compiled.map(str::to_string).or(runtime)?;
    let bytes: [u8; 32] = hex::decode(hex_key.trim()).ok()?.try_into().ok()?;
    VerifyingKey::from_bytes(&bytes).ok()
}

pub fn key_configured() -> bool {
    public_key(1).is_some()
}

pub fn get_pro_status() -> Result<ProStatus, CafError> {
    Ok(ProStatus {
        is_pro: false,
        plan: "free".to_string(),
        email: None,
        expires_at: None,
    })
}
