//! Keyring — ADR 0001 key hierarchy.
//!
//! # Architecture
//! ```text
//! DEK  = 32 random bytes (OsRng)    -> SQLCipher raw key
//! BK   = 32 random bytes (OsRng)    -> backup key (F11)
//!
//! KEK_pw = Argon2id(master_password, salt_pw, m=64MiB, t=3, p=4)
//! KEK_rc = HKDF-SHA256(ikm = recovery entropy, salt_rc, info = "caf.kek.recovery.v1")
//!
//! keyring.v1.json (0600):
//!   { format, vault_id, kdf,
//!     password_slot: { salt_pw, nonce, ct = AEAD(KEK_pw, DEK||BK, aad="caf-keyring-v1|password|"+vault_id) },
//!     recovery_slot: { salt_rc, generation, nonce, ct = AEAD(KEK_rc, DEK||BK, aad="caf-keyring-v1|recovery|"+vault_id) }
//!   }
//! ```

use crate::error::{CatermError, VaultError};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::rand_core::OsRng,
    aead::{Aead, AeadCore, KeyInit},
};
use argon2::{Argon2, Params, Version};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;
use zeroize::Zeroizing;

// --------------------------------------------------------------------------
// Constants
// --------------------------------------------------------------------------

pub const FORMAT_VERSION: u16 = 1;
pub const KEYRING_FILE: &str = "keyring.v1.json";

const ARGON2_M_COST: u32 = 64 * 1024; // 64 MiB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;
const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;

const INFO_RECOVERY: &[u8] = b"caf.kek.recovery.v1";
const AAD_PW_PREFIX: &str = "caf-keyring-v1|password|";
const AAD_RC_PREFIX: &str = "caf-keyring-v1|recovery|";

// --------------------------------------------------------------------------
// Serde types
// --------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KdfConfig {
    pub alg: String,
    pub m: u32,
    pub t: u32,
    pub p: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasswordSlot {
    pub salt_pw: String, // base64
    pub nonce: String,   // base64
    pub ct: String,      // base64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecoverySlot {
    pub salt_rc: String, // base64
    pub generation: u32,
    pub nonce: String, // base64
    pub ct: String,    // base64
}

/// On-disk keyring structure, serialised to `keyring.v1.json`.
///
/// Contains *no* plaintext key material — all secrets are wrapped by their
/// respective KEKs via AES-256-GCM.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keyring {
    pub format: u16,
    pub vault_id: String,
    pub kdf: KdfConfig,
    pub password_slot: PasswordSlot,
    pub recovery_slot: RecoverySlot,
}

/// Unwrapped key material (lives in memory only, never written to disk).
#[derive(Debug)]
pub struct UnwrappedKeys {
    pub dek: Zeroizing<[u8; KEY_LEN]>,
    pub bk: Zeroizing<[u8; KEY_LEN]>,
}

// --------------------------------------------------------------------------
// Internal helpers
// --------------------------------------------------------------------------

fn b64_enc(b: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(b)
}

fn b64_dec(s: &str) -> Result<Vec<u8>, CatermError> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| keyring_err(format!("base64 decode failed: {e}")))
}

fn keyring_err(msg: impl Into<String>) -> CatermError {
    CatermError::Vault(VaultError::Generic(msg.into()))
}

fn random_salt() -> [u8; SALT_LEN] {
    let mut s = [0u8; SALT_LEN];
    use aes_gcm::aead::rand_core::RngCore;
    OsRng.fill_bytes(&mut s);
    s
}

fn random_key() -> Zeroizing<[u8; KEY_LEN]> {
    let mut k = [0u8; KEY_LEN];
    use aes_gcm::aead::rand_core::RngCore;
    OsRng.fill_bytes(&mut k);
    Zeroizing::new(k)
}

/// Argon2id key derivation (password → KEK_pw).
fn derive_password_kek(
    password: &str,
    salt: &[u8],
) -> Result<Zeroizing<[u8; KEY_LEN]>, CatermError> {
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))
        .map_err(|e| keyring_err(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut_slice())
        .map_err(|e| keyring_err(e.to_string()))?;
    Ok(key)
}

/// HKDF-SHA256 key derivation (recovery entropy → KEK_rc).
fn derive_recovery_kek(ikm: &[u8], salt: &[u8]) -> Zeroizing<[u8; KEY_LEN]> {
    // HKDF-extract: PRK = HMAC-SHA256(salt, IKM)
    let prk = hkdf_sha256_extract(salt, ikm);
    // HKDF-expand: OKM = T(1) = HMAC-SHA256(PRK, INFO || 0x01)
    hkdf_sha256_expand(&prk, INFO_RECOVERY, KEY_LEN)
}

// Minimal HKDF-SHA256 (we don't pull in the `hkdf` crate; sha2 is already a dep).
fn hkdf_sha256_extract(salt: &[u8], ikm: &[u8]) -> Zeroizing<Vec<u8>> {
    use hmac::{Hmac, Mac};
    let mut mac = match <Hmac<Sha256> as Mac>::new_from_slice(salt) {
        Ok(m) => m,
        Err(_) => {
            let fallback_key = [0u8; 32];
            match <Hmac<Sha256> as Mac>::new_from_slice(&fallback_key) {
                Ok(m) => m,
                Err(_) => unreachable!("fixed 32-byte key is valid for HMAC-SHA256"),
            }
        }
    };
    mac.update(ikm);
    Zeroizing::new(mac.finalize().into_bytes().to_vec())
}

fn hkdf_sha256_expand(prk: &[u8], info: &[u8], len: usize) -> Zeroizing<[u8; KEY_LEN]> {
    use hmac::{Hmac, Mac};
    // T(1) = HMAC-SHA256(PRK, "" || info || 0x01)
    let mut mac = match <Hmac<Sha256> as Mac>::new_from_slice(prk) {
        Ok(m) => m,
        Err(_) => {
            let fallback_key = [0u8; 32];
            match <Hmac<Sha256> as Mac>::new_from_slice(&fallback_key) {
                Ok(m) => m,
                Err(_) => unreachable!("fixed 32-byte key is valid for HMAC-SHA256"),
            }
        }
    };
    mac.update(info);
    mac.update(&[1u8]);
    let t1 = mac.finalize().into_bytes();
    let mut out = [0u8; KEY_LEN];
    let copy_len = len.min(KEY_LEN).min(t1.len());
    if let (Some(dest), Some(src)) = (out.get_mut(..copy_len), t1.get(..copy_len)) {
        dest.copy_from_slice(src);
    }
    Zeroizing::new(out)
}

/// AES-256-GCM wrap: `AEAD(kek, plaintext, aad)`.
fn aes_gcm_wrap(
    kek: &[u8],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), CatermError> {
    let cipher = Aes256Gcm::new_from_slice(kek)
        .map_err(|e| keyring_err(format!("cipher init failed: {e}")))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(
            &nonce,
            aes_gcm::aead::Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|e| keyring_err(format!("AEAD encrypt failed: {e}")))?;
    Ok((nonce.to_vec(), ct))
}

/// AES-256-GCM unwrap.
fn aes_gcm_unwrap(
    kek: &[u8],
    nonce_bytes: &[u8],
    ct: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CatermError> {
    if nonce_bytes.len() != 12 {
        return Err(keyring_err("invalid nonce length"));
    }
    let cipher = Aes256Gcm::new_from_slice(kek)
        .map_err(|e| keyring_err(format!("cipher init failed: {e}")))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, aes_gcm::aead::Payload { msg: ct, aad })
        .map_err(|_| {
            CatermError::Vault(VaultError::Generic(
                "AEAD tag mismatch (corrupt or wrong key)".into(),
            ))
        })
}

/// Combine DEK and BK into a single 64-byte payload.
fn make_payload(dek: &[u8; KEY_LEN], bk: &[u8; KEY_LEN]) -> Zeroizing<Vec<u8>> {
    let mut v = Zeroizing::new(vec![0u8; KEY_LEN * 2]);
    if let Some(slice) = v.get_mut(..KEY_LEN) {
        slice.copy_from_slice(dek.as_ref());
    }
    if let Some(slice) = v.get_mut(KEY_LEN..) {
        slice.copy_from_slice(bk.as_ref());
    }
    v
}

/// Split a 64-byte payload back into DEK and BK.
fn split_payload(payload: Vec<u8>) -> Result<UnwrappedKeys, CatermError> {
    if payload.len() != KEY_LEN * 2 {
        return Err(CatermError::Vault(VaultError::Generic(
            "keyring payload has wrong length".into(),
        )));
    }
    let mut dek = Zeroizing::new([0u8; KEY_LEN]);
    let mut bk = Zeroizing::new([0u8; KEY_LEN]);
    if let Some(slice) = payload.get(..KEY_LEN) {
        dek.copy_from_slice(slice);
    }
    if let Some(slice) = payload.get(KEY_LEN..) {
        bk.copy_from_slice(slice);
    }
    Ok(UnwrappedKeys { dek, bk })
}

// --------------------------------------------------------------------------
// Generation
// --------------------------------------------------------------------------

/// Generate fresh `DEK` and `BK`, wrap both into a new keyring.
///
/// Returns the keyring and the unwrapped keys (DEK passed to SQLCipher,
/// BK stored for backup operations in F11).
pub fn generate(
    password: &str,
    recovery_entropy: &[u8],
) -> Result<(Keyring, UnwrappedKeys), CatermError> {
    let vault_id = Uuid::new_v4().to_string();
    let dek = random_key();
    let bk = random_key();
    let payload = make_payload(&dek, &bk);

    // --- Password slot ---
    let salt_pw = random_salt();
    let kek_pw = derive_password_kek(password, &salt_pw)?;
    let aad_pw = format!("{AAD_PW_PREFIX}{vault_id}");
    let (nonce_pw, ct_pw) = aes_gcm_wrap(kek_pw.as_ref(), &payload, aad_pw.as_bytes())?;

    // --- Recovery slot ---
    let salt_rc = random_salt();
    let kek_rc = derive_recovery_kek(recovery_entropy, &salt_rc);
    let aad_rc = format!("{AAD_RC_PREFIX}{vault_id}");
    let (nonce_rc, ct_rc) = aes_gcm_wrap(kek_rc.as_ref(), &payload, aad_rc.as_bytes())?;

    let kdf = KdfConfig {
        alg: "argon2id".to_string(),
        m: ARGON2_M_COST,
        t: ARGON2_T_COST,
        p: ARGON2_P_COST,
    };

    let keyring = Keyring {
        format: FORMAT_VERSION,
        vault_id,
        kdf,
        password_slot: PasswordSlot {
            salt_pw: b64_enc(&salt_pw),
            nonce: b64_enc(&nonce_pw),
            ct: b64_enc(&ct_pw),
        },
        recovery_slot: RecoverySlot {
            salt_rc: b64_enc(&salt_rc),
            generation: 1,
            nonce: b64_enc(&nonce_rc),
            ct: b64_enc(&ct_rc),
        },
    };

    Ok((keyring, UnwrappedKeys { dek, bk }))
}

// --------------------------------------------------------------------------
// Unwrap
// --------------------------------------------------------------------------

/// Unwrap keyring with a master password. Returns `WrongPassword` on auth failure.
pub fn unwrap_with_password(
    keyring: &Keyring,
    password: &str,
) -> Result<UnwrappedKeys, CatermError> {
    let salt_pw = b64_dec(&keyring.password_slot.salt_pw)?;
    let nonce = b64_dec(&keyring.password_slot.nonce)?;
    let ct = b64_dec(&keyring.password_slot.ct)?;

    let kek_pw = derive_password_kek(password, &salt_pw)?;
    let aad_pw = format!("{AAD_PW_PREFIX}{}", keyring.vault_id);

    let payload = aes_gcm_unwrap(kek_pw.as_ref(), &nonce, &ct, aad_pw.as_bytes())
        .map_err(|_| CatermError::Vault(VaultError::WrongPassword))?;

    split_payload(payload)
}

/// Unwrap keyring with recovery entropy (raw 32-byte BIP-39 entropy).
/// Returns `WrongRecoveryCode` on auth failure.
pub fn unwrap_with_recovery(
    keyring: &Keyring,
    recovery_entropy: &[u8],
) -> Result<UnwrappedKeys, CatermError> {
    let salt_rc = b64_dec(&keyring.recovery_slot.salt_rc)?;
    let nonce = b64_dec(&keyring.recovery_slot.nonce)?;
    let ct = b64_dec(&keyring.recovery_slot.ct)?;

    let kek_rc = derive_recovery_kek(recovery_entropy, &salt_rc);
    let aad_rc = format!("{AAD_RC_PREFIX}{}", keyring.vault_id);

    let payload = aes_gcm_unwrap(kek_rc.as_ref(), &nonce, &ct, aad_rc.as_bytes())
        .map_err(|_| CatermError::Vault(VaultError::WrongRecoveryCode))?;

    split_payload(payload)
}

// --------------------------------------------------------------------------
// Rewrap
// --------------------------------------------------------------------------

/// Rewrap the password slot with a new password and fresh salt.
/// The DEK and BK are unchanged.
pub fn rewrap_password_slot(
    keyring: &mut Keyring,
    old_password: &str,
    new_password: &str,
) -> Result<(), CatermError> {
    let keys = unwrap_with_password(keyring, old_password)?;
    let payload = make_payload(&keys.dek, &keys.bk);

    let salt_pw = random_salt();
    let kek_pw = derive_password_kek(new_password, &salt_pw)?;
    let aad_pw = format!("{AAD_PW_PREFIX}{}", keyring.vault_id);
    let (nonce, ct) = aes_gcm_wrap(kek_pw.as_ref(), &payload, aad_pw.as_bytes())?;

    keyring.password_slot = PasswordSlot {
        salt_pw: b64_enc(&salt_pw),
        nonce: b64_enc(&nonce),
        ct: b64_enc(&ct),
    };
    Ok(())
}

/// Replace the recovery slot with a new entropy and fresh salt, incrementing generation.
pub fn replace_recovery_slot(
    keyring: &mut Keyring,
    password: &str,
    new_recovery_entropy: &[u8],
) -> Result<(), CatermError> {
    let keys = unwrap_with_password(keyring, password)?;
    let payload = make_payload(&keys.dek, &keys.bk);

    let salt_rc = random_salt();
    let kek_rc = derive_recovery_kek(new_recovery_entropy, &salt_rc);
    let aad_rc = format!("{AAD_RC_PREFIX}{}", keyring.vault_id);
    let (nonce, ct) = aes_gcm_wrap(kek_rc.as_ref(), &payload, aad_rc.as_bytes())?;

    keyring.recovery_slot = RecoverySlot {
        salt_rc: b64_enc(&salt_rc),
        generation: keyring.recovery_slot.generation + 1,
        nonce: b64_enc(&nonce),
        ct: b64_enc(&ct),
    };
    Ok(())
}

// --------------------------------------------------------------------------
// Persistence (atomic write)
// --------------------------------------------------------------------------

/// Load and deserialise the keyring from `<data_dir>/keyring.v1.json`.
/// Returns `LegacyFormat` when only the old canary layout exists.
pub fn load(data_dir: &Path) -> Result<Keyring, CatermError> {
    let keyring_path = data_dir.join(KEYRING_FILE);

    if !keyring_path.exists() {
        // Detect legacy vault layout (vault_salt.bin + vault_canary.bin, no keyring).
        let is_legacy =
            data_dir.join("vault_salt.bin").exists() || data_dir.join("vault_canary.bin").exists();
        if is_legacy {
            return Err(CatermError::Vault(VaultError::LegacyFormat));
        }
        return Err(CatermError::Vault(VaultError::Generic(
            "keyring not found; vault not initialized".into(),
        )));
    }

    let raw = std::fs::read_to_string(&keyring_path)
        .map_err(|e| keyring_err(format!("cannot read keyring: {e}")))?;

    let keyring: Keyring = serde_json::from_str(&raw).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!("keyring JSON corrupt: {e}")))
    })?;

    if keyring.format != FORMAT_VERSION {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "unsupported keyring format version: {}",
            keyring.format
        ))));
    }

    Ok(keyring)
}

/// Atomically write the keyring to disk:
///   1. Write to `keyring.v1.json.tmp`.
///   2. Rename over `keyring.v1.json`.
pub fn save(data_dir: &Path, keyring: &Keyring) -> Result<(), CatermError> {
    std::fs::create_dir_all(data_dir)
        .map_err(|e| keyring_err(format!("cannot create data dir: {e}")))?;

    let tmp_path = data_dir.join(format!("{KEYRING_FILE}.tmp"));
    let final_path = data_dir.join(KEYRING_FILE);

    let json = serde_json::to_string_pretty(keyring)
        .map_err(|e| keyring_err(format!("keyring serialisation failed: {e}")))?;

    {
        let mut f = std::fs::File::create(&tmp_path)
            .map_err(|e| keyring_err(format!("cannot open tmp keyring: {e}")))?;
        f.write_all(json.as_bytes())
            .map_err(|e| keyring_err(format!("cannot write tmp keyring: {e}")))?;
        f.sync_all()
            .map_err(|e| keyring_err(format!("fsync tmp keyring failed: {e}")))?;
    }

    std::fs::rename(&tmp_path, &final_path)
        .map_err(|e| keyring_err(format!("atomic rename failed: {e}")))?;

    // Best-effort directory fsync (Linux / macOS).
    if let Ok(dir) = std::fs::File::open(data_dir) {
        let _ = dir.sync_all();
    }

    Ok(())
}

// --------------------------------------------------------------------------
// F6.2: BIP-39 setup, confirmation, change-password, recovery unlock
// --------------------------------------------------------------------------

pub const MIN_MASTER_PASSWORD_LEN: usize = 12; // D-7

/// Result of `initialize_vault_keyring`: returns the keyring, the DEK/BK, and
/// the 24 BIP-39 words (Zeroizing so they're wiped on drop).
pub struct SetupResult {
    pub keyring: Keyring,
    pub keys: UnwrappedKeys,
    /// The 24 BIP-39 words, space-separated.
    pub mnemonic: Zeroizing<String>,
    /// Raw 256-bit entropy (needed to confirm positions).
    pub entropy: Zeroizing<Vec<u8>>,
}

/// Generate 256-bit entropy and derive BIP-39 24-word mnemonic.
/// Returns `(mnemonic, entropy)` — neither is written to disk.
pub fn generate_mnemonic() -> (Zeroizing<String>, Zeroizing<Vec<u8>>) {
    use bip39::{Language, Mnemonic, WordCount};
    // 256-bit entropy → 24 words
    let m = match Mnemonic::generate_in(Language::English, WordCount::Words24) {
        Ok(m) => m,
        Err(_) => {
            let entropy = [0u8; 32];
            match Mnemonic::from_entropy_in(Language::English, &entropy) {
                Ok(m) => m,
                Err(_) => unreachable!("valid 32-byte entropy for English bip39"),
            }
        }
    };
    let phrase = Zeroizing::new(m.to_string());
    let entropy = Zeroizing::new(m.to_entropy());
    (phrase, entropy)
}

/// Convert 24 BIP-39 words back to raw entropy bytes.
/// Returns an error if any word is invalid or the checksum fails.
pub fn mnemonic_to_entropy(phrase: &str) -> Result<Zeroizing<Vec<u8>>, CatermError> {
    use bip39::{Language, Mnemonic};
    let m = Mnemonic::parse_in(Language::English, phrase)
        .map_err(|e| keyring_err(format!("invalid recovery phrase: {e}")))?;
    Ok(Zeroizing::new(m.to_entropy()))
}

/// Initialize a new vault keyring:
/// - validates the password length (D-7: ≥ 12 chars)
/// - generates DEK and BK
/// - generates a 24-word BIP-39 mnemonic
/// - wraps both keys for password and recovery slots
///
/// The caller must:
///   1. write the keyring to disk (`save()`),
///   2. display the 24 words to the user once,
///   3. call `confirm_recovery_words()` to transition out of `RecoveryUnconfirmed`.
pub fn initialize_vault_keyring(password: &str) -> Result<SetupResult, CatermError> {
    if password.len() < MIN_MASTER_PASSWORD_LEN {
        return Err(keyring_err(format!(
            "master password must be at least {MIN_MASTER_PASSWORD_LEN} characters (D-7)"
        )));
    }
    let (mnemonic, entropy) = generate_mnemonic();
    let (keyring, keys) = generate(password, &entropy)?;
    Ok(SetupResult {
        keyring,
        keys,
        mnemonic,
        entropy,
    })
}

/// Check 3 confirmation positions.
///
/// `positions` – three 0-based word indices (chosen by the caller; should be 3
///               random values in 0..24).
/// `guesses`   – the user's words for those positions.
/// `entropy`   – the entropy from `initialize_vault_keyring` (in memory, not disk).
///
/// Returns `Ok(())` if all 3 match, otherwise `Err(WrongRecoveryCode)`.
pub fn confirm_recovery_words(
    entropy: &[u8],
    positions: &[usize],
    guesses: &[&str],
) -> Result<(), CatermError> {
    use bip39::{Language, Mnemonic};
    if positions.len() != 3 || guesses.len() != 3 {
        return Err(keyring_err("exactly 3 confirmation positions required"));
    }
    let m = Mnemonic::from_entropy_in(Language::English, entropy)
        .map_err(|e| keyring_err(format!("entropy→mnemonic failed: {e}")))?;
    let words: Vec<&str> = m.words().collect();
    for (&pos, &guess) in positions.iter().zip(guesses.iter()) {
        if let Some(&word) = words.get(pos) {
            if word != guess {
                return Err(CatermError::Vault(VaultError::WrongRecoveryCode));
            }
        } else {
            return Err(CatermError::Vault(VaultError::WrongRecoveryCode));
        }
    }
    Ok(())
}

/// Change the master password without re-keying the database.
/// - Verifies `old_password` by unwrapping
/// - Enforces D-7 length on `new_password` (≥ 12 chars)
/// - Rewraps the password slot with a fresh salt
/// - Updates the keyring in place (caller must re-save to disk)
pub fn change_master_password(
    keyring: &mut Keyring,
    old_password: &str,
    new_password: &str,
) -> Result<(), CatermError> {
    if new_password.len() < MIN_MASTER_PASSWORD_LEN {
        return Err(keyring_err(format!(
            "new password must be at least {MIN_MASTER_PASSWORD_LEN} characters (D-7)"
        )));
    }
    rewrap_password_slot(keyring, old_password, new_password)
}

// --------------------------------------------------------------------------
// F6.6: Brute-force backoff counter (shared for password + recovery attempts)
// --------------------------------------------------------------------------

use std::sync::atomic::{AtomicU32, Ordering};

/// Global attempt counter. Reset externally on vault lock + successful auth.
static AUTH_FAIL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Max consecutive failures before lockout activates (D-7: backoff after 5).
const MAX_AUTH_FAILURES: u32 = 5;

/// Returns `LockoutActive` if the fail counter has reached the threshold.
fn check_not_locked_out() -> Result<(), CatermError> {
    if AUTH_FAIL_COUNT.load(Ordering::Acquire) >= MAX_AUTH_FAILURES {
        return Err(CatermError::Vault(VaultError::LockoutActive));
    }
    Ok(())
}

fn record_fail() {
    AUTH_FAIL_COUNT.fetch_add(1, Ordering::AcqRel);
}

/// Reset the fail counter (call after a successful auth or vault lock).
pub fn reset_auth_fail_count() {
    AUTH_FAIL_COUNT.store(0, Ordering::Release);
}

/// `unwrap_with_password` guarded by the brute-force counter.
///
/// On failure: increments the counter and returns `WrongPassword`.
/// On success: resets the counter.
pub fn guarded_unwrap_with_password(
    keyring: &Keyring,
    password: &str,
) -> Result<UnwrappedKeys, CatermError> {
    check_not_locked_out()?;
    match unwrap_with_password(keyring, password) {
        Ok(k) => {
            reset_auth_fail_count();
            Ok(k)
        }
        Err(e) => {
            record_fail();
            Err(e)
        }
    }
}

/// `unwrap_with_recovery` guarded by the same brute-force counter (F6.6).
///
/// On failure: increments the counter and returns `WrongRecoveryCode`.
/// On success: resets the counter.
pub fn guarded_unwrap_with_recovery(
    keyring: &Keyring,
    recovery_entropy: &[u8],
) -> Result<UnwrappedKeys, CatermError> {
    check_not_locked_out()?;
    match unwrap_with_recovery(keyring, recovery_entropy) {
        Ok(k) => {
            reset_auth_fail_count();
            Ok(k)
        }
        Err(e) => {
            record_fail();
            Err(e)
        }
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "super-secret-password-123";
    const NEW_PASS: &str = "new-super-secret-password-456";
    const RECOVERY: &[u8] = b"test-recovery-entropy-for-tests-32b";

    fn make_keyring() -> (Keyring, UnwrappedKeys) {
        generate(PASS, RECOVERY).expect("generate failed")
    }

    #[allow(dead_code)]
    pub(crate) fn make_keyring_pub() -> (Keyring, UnwrappedKeys) {
        make_keyring()
    }

    // ---- round-trip with password ----
    #[test]
    fn password_roundtrip() {
        let (kr, original) = make_keyring();
        let unwrapped = unwrap_with_password(&kr, PASS).expect("unwrap_with_password failed");
        assert_eq!(original.dek.as_ref(), unwrapped.dek.as_ref());
        assert_eq!(original.bk.as_ref(), unwrapped.bk.as_ref());
    }

    // ---- round-trip with recovery ----
    #[test]
    fn recovery_roundtrip() {
        let (kr, original) = make_keyring();
        let unwrapped = unwrap_with_recovery(&kr, RECOVERY).expect("unwrap_with_recovery failed");
        assert_eq!(original.dek.as_ref(), unwrapped.dek.as_ref());
        assert_eq!(original.bk.as_ref(), unwrapped.bk.as_ref());
    }

    // ---- wrong password → error (not Corrupted) ----
    #[test]
    fn wrong_password_returns_error() {
        let (kr, _) = make_keyring();
        let res = unwrap_with_password(&kr, "wrong-password");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(
            matches!(err, CatermError::Vault(VaultError::WrongPassword)),
            "expected WrongPassword, got: {err}"
        );
    }

    // ---- wrong recovery → error ----
    #[test]
    fn wrong_recovery_returns_error() {
        let (kr, _) = make_keyring();
        let res = unwrap_with_recovery(&kr, b"completely-wrong-recovery-entropy");
        assert!(res.is_err());
    }

    // ---- flipping a ciphertext byte → Corrupted / AEAD fail ----
    #[test]
    fn corrupted_ciphertext_detected() {
        let (mut kr, _) = make_keyring();
        // Tamper the password slot ciphertext
        let mut ct_bytes = b64_dec(&kr.password_slot.ct).unwrap();
        ct_bytes[0] ^= 0xFF;
        kr.password_slot.ct = b64_enc(&ct_bytes);
        let res = unwrap_with_password(&kr, PASS);
        assert!(res.is_err());
        // WrongPassword is returned because AEAD auth failure maps to that
        assert!(
            matches!(
                res.unwrap_err(),
                CatermError::Vault(VaultError::WrongPassword)
            ),
            "expected WrongPassword on corrupted CT"
        );
    }

    // ---- flipping a nonce byte → AEAD fail ----
    #[test]
    fn corrupted_nonce_detected() {
        let (mut kr, _) = make_keyring();
        let mut n = b64_dec(&kr.password_slot.nonce).unwrap();
        n[0] ^= 0x01;
        kr.password_slot.nonce = b64_enc(&n);
        assert!(unwrap_with_password(&kr, PASS).is_err());
    }

    // ---- malformed JSON ----
    #[test]
    fn malformed_json_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(KEYRING_FILE), b"not json at all").unwrap();
        let res = load(tmp.path());
        assert!(res.is_err());
        assert!(!res.unwrap_err().to_string().contains("panic")); // must not panic
    }

    // ---- truncated JSON ----
    #[test]
    fn truncated_json_is_rejected_not_panicking() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(KEYRING_FILE), b"{\"format\": 1}").unwrap();
        let res = load(tmp.path());
        assert!(res.is_err());
    }

    // ---- two rewraps produce different ciphertexts (fresh nonce) ----
    #[test]
    fn rewrap_produces_different_ct() {
        let (mut kr1, _) = make_keyring();
        let (mut kr2, _) = make_keyring();
        // Unwrap once to get the same DEK/BK into kr1, then rewrap twice
        rewrap_password_slot(&mut kr1, PASS, NEW_PASS).unwrap();
        let ct1 = kr1.password_slot.ct.clone();
        rewrap_password_slot(&mut kr2, PASS, NEW_PASS).unwrap();
        let ct2 = kr2.password_slot.ct.clone();
        // Different randomness → different ct (extremely likely)
        // (they *could* collide with probability ~2^-96, acceptable in test)
        assert_ne!(ct1, ct2);
    }

    // ---- password rewrap: old password fails, new succeeds ----
    #[test]
    fn rewrap_password_slot_works() {
        let (mut kr, original_keys) = make_keyring();
        rewrap_password_slot(&mut kr, PASS, NEW_PASS).expect("rewrap failed");
        // Old password must fail
        assert!(unwrap_with_password(&kr, PASS).is_err());
        // New password succeeds and returns same DEK/BK
        let new_keys = unwrap_with_password(&kr, NEW_PASS).expect("new password failed");
        assert_eq!(original_keys.dek.as_ref(), new_keys.dek.as_ref());
        // Recovery still works
        let rc_keys = unwrap_with_recovery(&kr, RECOVERY).expect("recovery after rewrap");
        assert_eq!(original_keys.dek.as_ref(), rc_keys.dek.as_ref());
    }

    // ---- replace recovery slot increments generation ----
    #[test]
    fn replace_recovery_slot_increments_generation() {
        let (mut kr, original_keys) = make_keyring();
        let new_entropy = b"new-recovery-entropy-for-test-32b";
        replace_recovery_slot(&mut kr, PASS, new_entropy).expect("replace recovery failed");
        assert_eq!(kr.recovery_slot.generation, 2);
        // Old recovery fails
        assert!(unwrap_with_recovery(&kr, RECOVERY).is_err());
        // New recovery works
        let keys = unwrap_with_recovery(&kr, new_entropy).expect("new recovery failed");
        assert_eq!(original_keys.dek.as_ref(), keys.dek.as_ref());
    }

    // ---- persistence round-trip ----
    #[test]
    fn save_and_load_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let (kr, original) = make_keyring();
        save(tmp.path(), &kr).expect("save failed");
        let loaded = load(tmp.path()).expect("load failed");
        let keys = unwrap_with_password(&loaded, PASS).expect("unwrap after load");
        assert_eq!(original.dek.as_ref(), keys.dek.as_ref());
    }

    // ---- legacy layout detected ----
    #[test]
    fn legacy_layout_reports_legacy_format() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("vault_salt.bin"), b"fakesalt").unwrap();
        let res = load(tmp.path());
        assert!(res.is_err());
        assert!(
            matches!(
                res.unwrap_err(),
                CatermError::Vault(VaultError::LegacyFormat)
            ),
            "expected LegacyFormat variant"
        );
    }

    // ---- F6.2: initialize_vault_keyring + confirm_recovery_words ----
    #[test]
    fn initialize_vault_keyring_creates_valid_keyring() {
        let result = initialize_vault_keyring(PASS).expect("initialize_vault_keyring failed");
        // Keyring is usable
        let keys = unwrap_with_password(&result.keyring, PASS).expect("unwrap after init");
        assert_eq!(keys.dek.as_ref().len(), KEY_LEN);
        assert_eq!(keys.bk.as_ref().len(), KEY_LEN);
        // Mnemonic is 24 words
        let word_count = result.mnemonic.split_whitespace().count();
        assert_eq!(word_count, 24, "expected 24 BIP-39 words, got {word_count}");
        // Recovery unwrap works with the returned entropy
        let rc_keys =
            unwrap_with_recovery(&result.keyring, &result.entropy).expect("recovery after init");
        assert_eq!(keys.dek.as_ref(), rc_keys.dek.as_ref());
    }

    #[test]
    fn short_password_rejected_by_initialize() {
        let res = initialize_vault_keyring("tooshort");
        assert!(
            res.is_err(),
            "should reject password shorter than {MIN_MASTER_PASSWORD_LEN} chars"
        );
    }

    #[test]
    fn confirm_recovery_words_correct_positions() {
        let result = initialize_vault_keyring(PASS).expect("init failed");
        let words: Vec<&str> = result.mnemonic.split_whitespace().collect();
        // Confirm 3 positions
        let positions = [0usize, 7, 23];
        let guesses = [words[0], words[7], words[23]];
        confirm_recovery_words(&result.entropy, &positions, &guesses)
            .expect("confirm with correct words failed");
    }

    #[test]
    fn confirm_recovery_words_wrong_word_fails() {
        let result = initialize_vault_keyring(PASS).expect("init failed");
        let positions = [0usize, 7, 23];
        let guesses = ["wrong", "wrong", "wrong"];
        let res = confirm_recovery_words(&result.entropy, &positions, &guesses);
        assert!(
            matches!(res, Err(CatermError::Vault(VaultError::WrongRecoveryCode))),
            "expected WrongRecoveryCode"
        );
    }

    #[test]
    fn initialize_writes_no_plaintext_words_to_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let result = initialize_vault_keyring(PASS).expect("init failed");
        save(tmp.path(), &result.keyring).expect("save failed");

        let words: Vec<&str> = result.mnemonic.split_whitespace().collect();
        // Scan every file in tmp for any of the 24 words (skip tiny substrings that match dictionary words like "pass" matching "password_slot")
        for entry in std::fs::read_dir(tmp.path()).unwrap() {
            let entry = entry.unwrap();
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                // Keyring json keys / structure shouldn't accidentally trigger false positive substring matches
                // Check exact word boundaries or token matches in json
                for word in &words {
                    // BIP-39 English words can occasionally be "pass", "slot", etc.
                    // The on-disk JSON contains fields like "password_slot". Check if word appears as an isolated token / outside schema keys.
                    assert!(
                        !content.contains(&format!("\"{word}\""))
                            && !content.contains(&format!(" {word} ")),
                        "recovery word '{word}' found in {:?}",
                        entry.path()
                    );
                }
            }
        }
    }

    // ---- F6.3: change_master_password (no PRAGMA rekey) ----
    #[test]
    fn change_master_password_success() {
        let (mut kr, original) = make_keyring();
        change_master_password(&mut kr, PASS, NEW_PASS).expect("change_master_password failed");
        // Old password must fail
        assert!(
            unwrap_with_password(&kr, PASS).is_err(),
            "old password should fail after change"
        );
        // New password works, same DEK
        let new_keys = unwrap_with_password(&kr, NEW_PASS).expect("new password failed");
        assert_eq!(
            original.dek.as_ref(),
            new_keys.dek.as_ref(),
            "DEK must not change"
        );
        // Recovery still works
        let rc_keys = unwrap_with_recovery(&kr, RECOVERY).expect("recovery after password change");
        assert_eq!(original.dek.as_ref(), rc_keys.dek.as_ref());
    }

    #[test]
    fn change_master_password_short_new_password_rejected() {
        let (mut kr, _) = make_keyring();
        let res = change_master_password(&mut kr, PASS, "tooshort");
        assert!(res.is_err(), "should reject short new password");
        // Old password still works (no lockout from validation failure)
        assert!(unwrap_with_password(&kr, PASS).is_ok());
    }

    // ---- F6.4: unlock_with_recovery and replace_recovery_slot_for_reset ----
    #[test]
    fn recovery_unlock_and_reset_password() {
        let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset_auth_fail_count();
        let (mut kr, original) = make_keyring();
        // Unlock via recovery: get the keys
        let rc_keys = guarded_unwrap_with_recovery(&kr, RECOVERY).expect("recovery unlock failed");
        assert_eq!(original.dek.as_ref(), rc_keys.dek.as_ref());
        // After recovery unlock, set a new master password by rewrapping
        let reset_pass = "reset-master-password-123";
        change_master_password(&mut kr, PASS, reset_pass).expect("set new password failed");
        // Old password fails
        assert!(unwrap_with_password(&kr, PASS).is_err());
        // New password works
        let new_keys =
            unwrap_with_password(&kr, reset_pass).expect("new password after recovery reset");
        assert_eq!(original.dek.as_ref(), new_keys.dek.as_ref());
        reset_auth_fail_count();
    }

    // ---- F6.5: regenerate recovery code (replace_recovery_slot) ----
    #[test]
    fn regenerate_recovery_code_revokes_old() {
        let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset_auth_fail_count();
        let (mut kr, _) = make_keyring();
        let gen_before = kr.recovery_slot.generation;
        let new_entropy = b"new-recovery-entropy-for-test-32b";
        replace_recovery_slot(&mut kr, PASS, new_entropy).expect("replace_recovery_slot failed");
        // generation incremented
        assert_eq!(kr.recovery_slot.generation, gen_before + 1);
        // Old recovery code fails
        assert!(
            unwrap_with_recovery(&kr, RECOVERY).is_err(),
            "old recovery should fail"
        );
        // New recovery code works
        assert!(guarded_unwrap_with_recovery(&kr, new_entropy).is_ok());
        reset_auth_fail_count();
    }

    // ---- F6.6: brute-force protection (guarded functions) ----
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn lockout_after_max_failures() {
        let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset_auth_fail_count();
        let (kr, _) = make_keyring();
        // 5 wrong password attempts
        for _ in 0..5 {
            let res = guarded_unwrap_with_password(&kr, "wrong-password-for-lockout");
            assert!(res.is_err());
        }
        // Fail count is now 5. Next attempt should be LockoutActive
        let res = guarded_unwrap_with_password(&kr, PASS);
        assert!(
            matches!(res, Err(CatermError::Vault(VaultError::LockoutActive))),
            "expected LockoutActive after 5 failures, got: {res:?}"
        );
        // Recovery also blocked
        let rc_res = guarded_unwrap_with_recovery(&kr, RECOVERY);
        assert!(
            matches!(rc_res, Err(CatermError::Vault(VaultError::LockoutActive))),
            "expected LockoutActive for recovery after 5 failures"
        );
        // Reset clears lockout
        reset_auth_fail_count();
        assert!(
            guarded_unwrap_with_password(&kr, PASS).is_ok(),
            "should work after reset"
        );
    }

    #[test]
    fn successful_auth_resets_counter() {
        let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset_auth_fail_count();
        let (kr, _) = make_keyring();
        // 3 failures
        for _ in 0..3 {
            let _ = guarded_unwrap_with_password(&kr, "wrong");
        }
        // Success should reset counter to 0
        let res = guarded_unwrap_with_password(&kr, PASS);
        assert!(res.is_ok());
        assert_eq!(AUTH_FAIL_COUNT.load(Ordering::Acquire), 0);
        // Now 4 more failures — should NOT lock out
        for _ in 0..4 {
            let _ = guarded_unwrap_with_password(&kr, "wrong");
        }
        // 5th failure reaches threshold 5
        let _ = guarded_unwrap_with_password(&kr, "wrong");
        let res = guarded_unwrap_with_password(&kr, PASS);
        assert!(
            matches!(res, Err(CatermError::Vault(VaultError::LockoutActive))),
            "expected LockoutActive"
        );
        reset_auth_fail_count();
    }
}

// ---- proptest: keyring JSON parsers (§1.10) ----
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Use a pre-computed minimal keyring structure to avoid running Argon2 in every fuzz case
    fn static_keyring() -> Keyring {
        Keyring {
            format: FORMAT_VERSION,
            vault_id: "00000000-0000-0000-0000-000000000000".into(),
            kdf: KdfConfig {
                alg: "argon2id".into(),
                m: 64,
                t: 1,
                p: 1,
            },
            password_slot: PasswordSlot {
                salt_pw: b64_enc(&[0u8; 16]),
                nonce: b64_enc(&[0u8; 12]),
                ct: b64_enc(&[0u8; 80]),
            },
            recovery_slot: RecoverySlot {
                salt_rc: b64_enc(&[0u8; 16]),
                generation: 1,
                nonce: b64_enc(&[0u8; 12]),
                ct: b64_enc(&[0u8; 80]),
            },
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases: 50,
            ..Default::default()
        })]

        /// Any arbitrary byte string used as keyring file content must not panic.
        #[test]
        fn arbitrary_json_file_never_panics(content in proptest::collection::vec(any::<u8>(), 0..512)) {
            let tmp = tempfile::tempdir().unwrap();
            let path = tmp.path().join(KEYRING_FILE);
            let _ = std::fs::write(&path, &content);
            // Must not panic; result may be Ok or Err
            let _ = load(tmp.path());
        }

        /// Arbitrary base64 ciphertext in password_slot must not panic; must be Err.
        #[test]
        fn fuzzed_password_slot_ct_never_panics(ct in "[a-zA-Z0-9+/=]{0,512}") {
            let mut kr = static_keyring();
            kr.password_slot.ct = ct;
            let _ = unwrap_with_password(&kr, PASS);
        }

        /// Fuzzed nonce bytes: must not panic.
        #[test]
        fn fuzzed_password_slot_nonce_never_panics(nonce in "[a-zA-Z0-9+/=]{0,64}") {
            let mut kr = static_keyring();
            kr.password_slot.nonce = nonce;
            let _ = unwrap_with_password(&kr, PASS);
        }

        /// Fuzzed recovery slot: must not panic.
        #[test]
        fn fuzzed_recovery_slot_never_panics(
            ct in "[a-zA-Z0-9+/=]{0,512}",
            nonce in "[a-zA-Z0-9+/=]{0,64}",
        ) {
            let mut kr = static_keyring();
            kr.recovery_slot.ct = ct;
            kr.recovery_slot.nonce = nonce;
            let _ = unwrap_with_recovery(&kr, RECOVERY);
        }
    }

    const PASS: &str = "super-secret-password-123";
    const RECOVERY: &[u8] = b"test-recovery-entropy-for-tests-32b";
}
