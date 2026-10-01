//! Zero-Knowledge Argon2id Vault Manager.

use crate::error::{CatermError, DbError, VaultError};
use argon2::{
    Argon2, Params, Version,
    password_hash::rand_core::{OsRng, RngCore},
};
use parking_lot::RwLock;
use std::sync::LazyLock;
use zeroize::Zeroize;

pub const MIN_PASSWORD_LEN: usize = 8;

const CANARY_FILE: &str = "vault_canary.bin";
const CANARY_PLAINTEXT: &[u8] = b"CAFRAMEWORK_VAULT_CANARY_V2";
const ARGON2_M_COST: u32 = 64 * 1024; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

const SALT_FILE: &str = "vault_salt.bin";
const SALT_LEN: usize = 16;

static ACTIVE_VAULT_KEY: LazyLock<RwLock<Option<[u8; 32]>>> = LazyLock::new(|| RwLock::new(None));

fn vault_err(msg: impl Into<String>) -> CatermError {
    CatermError::Vault(VaultError::Generic(msg.into()))
}

pub fn is_unlocked() -> Result<bool, CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    Ok(guard.is_some())
}

pub fn ensure_unlocked_key() -> Result<String, CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    if let Some(key) = *guard {
        Ok(hex::encode(key))
    } else {
        let data_dir = crate::paths::resolve_data_dir()?.path;
        load_or_create_local_key(&data_dir)
    }
}

pub fn is_vault_initialized() -> Result<bool, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    Ok(data_dir.join(CANARY_FILE).exists())
}

pub fn lock() {
    let mut guard = ACTIVE_VAULT_KEY.write();
    if let Some(mut key) = guard.take() {
        key.zeroize();
    }
}

pub fn reset_vault() -> Result<(), CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut key) = guard.take() {
            key.zeroize();
        }
    }

    let files = [
        CANARY_FILE,
        SALT_FILE,
        "vault.key",
        "local.key",
        "caframework.db",
        "caframework.db-wal",
        "caframework.db-shm",
        "caframework.db-journal",
    ];
    for f in &files {
        let p = data_dir.join(f);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }

    Ok(())
}

fn check_password_len(password: &str) -> Result<(), CatermError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(vault_err(format!(
            "Master password minimal {MIN_PASSWORD_LEN} karakter."
        )));
    }
    Ok(())
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], CatermError> {
    check_password_len(master_password)?;

    let mut derived_key = [0u8; 32];
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| vault_err(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| vault_err(e.to_string()))?;
    Ok(derived_key)
}

fn load_or_create_salt(data_dir: &std::path::Path) -> Result<[u8; SALT_LEN], CatermError> {
    let path = data_dir.join(SALT_FILE);
    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(salt) = <[u8; SALT_LEN]>::try_from(bytes.as_slice())
    {
        return Ok(salt);
    }

    std::fs::create_dir_all(data_dir).map_err(|e| vault_err(e.to_string()))?;
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    std::fs::write(&path, salt).map_err(|e| vault_err(e.to_string()))?;
    Ok(salt)
}

fn verify_canary(key: &[u8; 32], canary_path: &std::path::Path) -> Result<(), CatermError> {
    let wrong_password = || vault_err("Master password salah. Silakan coba lagi.");
    let encrypted_canary =
        std::fs::read_to_string(canary_path).map_err(|e| vault_err(e.to_string()))?;
    let decrypted =
        crate::secret::decrypt_bytes(key, &encrypted_canary).map_err(|_| wrong_password())?;
    if decrypted != CANARY_PLAINTEXT {
        return Err(wrong_password());
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<bool, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);

    if !canary_path.exists() {
        // First run — initialize vault
        let salt = load_or_create_salt(&data_dir)?;
        let key = derive_key(password, &salt)?;
        let encrypted = crate::secret::encrypt_bytes(&key, CANARY_PLAINTEXT)?;
        std::fs::write(&canary_path, encrypted).map_err(|e| vault_err(e.to_string()))?;

        {
            let mut guard = ACTIVE_VAULT_KEY.write();
            *guard = Some(key);
        }

        let _ = crate::db::open()?;
        let _ = crate::audit::log_event("VAULT_INIT", None, "Vault initialized");
        return Ok(true);
    }

    let salt = load_or_create_salt(&data_dir)?;
    let key = derive_key(password, &salt)?;
    if verify_canary(&key, &canary_path).is_ok() {
        {
            let mut guard = ACTIVE_VAULT_KEY.write();
            *guard = Some(key);
        }
        let _ = crate::db::open()?;
        let _ = crate::audit::log_event("VAULT_UNLOCK", None, "Vault unlocked");
        Ok(true)
    } else {
        Err(vault_err("Master password salah."))
    }
}

pub fn change_master_password(old_pass: &str, new_pass: &str) -> Result<(), CatermError> {
    if old_pass == new_pass {
        return Err(vault_err("Master password baru harus berbeda."));
    }
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);
    if !canary_path.exists() {
        return Err(vault_err("Vault belum dibuat."));
    }

    let salt = load_or_create_salt(&data_dir)?;
    let old_key = derive_key(old_pass, &salt)?;
    verify_canary(&old_key, &canary_path)?;

    let new_key = derive_key(new_pass, &salt)?;

    // PRAGMA rekey on active DB
    let conn = crate::db::open()?;
    let hex_new = hex::encode(new_key);
    conn.execute(&format!("PRAGMA rekey = \"x'{hex_new}'\";"), [])
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let encrypted = crate::secret::encrypt_bytes(&new_key, CANARY_PLAINTEXT)?;
    std::fs::write(&canary_path, encrypted).map_err(|e| vault_err(e.to_string()))?;

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        *guard = Some(new_key);
    }

    let _ = crate::audit::log_event("VAULT_REKEY", None, "Master password changed");
    Ok(())
}

pub fn load_or_create_local_key(data_dir: &std::path::Path) -> Result<String, CatermError> {
    let path = data_dir.join("vault.key");
    if let Ok(content) = std::fs::read_to_string(&path) {
        let trimmed = content.trim();
        if trimmed.len() == 64 && hex::decode(trimmed).is_ok() {
            return Ok(trimmed.to_string());
        }
    }

    std::fs::create_dir_all(data_dir).map_err(|e| vault_err(e.to_string()))?;
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let hex_key = hex::encode(bytes);
    std::fs::write(&path, &hex_key).map_err(|e| vault_err(e.to_string()))?;
    Ok(hex_key)
}
