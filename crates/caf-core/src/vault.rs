//! Zero-Knowledge Argon2id Vault Manager (`T2-CORE-01` .. `T2-CORE-03`).
//!
//! Derives a 256-bit Key Encryption Key (KEK) from the user's master password
//! using Argon2id (m=64MB, t=3, p=4), which in turn protects the Database Encryption Key (DEK).
//!
//! Includes in-memory zeroization on lock to eliminate forensic RAM leakage.

use crate::error::{CatermError, DbError, VaultError};
use argon2::{Argon2, Params, Version};
use parking_lot::RwLock;
use std::sync::LazyLock;
use zeroize::Zeroize;

pub const MIN_PASSWORD_LEN: usize = 8;

const CANARY_FILE: &str = "vault_canary.bin";
const CANARY_PLAINTEXT: &[u8] = b"CATERM_VAULT_CANARY_V2";
const ARGON2_M_COST: u32 = 64 * 1024; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

/// Random, per-installation Argon2id salt (C-15), generated once and kept beside the vault it
/// belongs to.
const SALT_FILE: &str = "vault_salt.bin";
const SALT_LEN: usize = 16;

/// Every installation before C-15 derived its KEK with this single hardcoded salt, meaning a
/// precomputed Argon2id table for it would crack *every* CAFramework vault's master password at
/// once, not just one victim's — the whole point of a salt is to rule that out. Kept only so an
/// existing vault can be opened one last time and migrated onto a real random salt; never used
/// for anything else.
const LEGACY_STATIC_SALT: &[u8] = b"caframework.zero_knowledge.v2.domain_salt_2026";

/// Vault state holding the active decrypted DEK in protected memory.
static ACTIVE_VAULT_KEY: LazyLock<RwLock<Option<[u8; 32]>>> = LazyLock::new(|| RwLock::new(None));

pub fn is_unlocked() -> Result<bool, CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    Ok(guard.is_some())
}

pub fn is_vault_initialized() -> Result<bool, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    Ok(data_dir.join("vault_canary.bin").exists())
}

pub fn reset_vault() -> Result<(), CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;

    // 1. Disconnect all active tunnels and SSH sessions
    let _ = crate::tunnels::stop_all_tunnels();
    if let Ok(mut sessions) = crate::ssh::SESSIONS.lock() {
        for (_id, handle) in sessions.drain() {
            if let Some(channel) = &handle.channel
                && let Ok(mut ch) = channel.lock()
            {
                let _ = ch.close();
            }
            if let Some(child) = &handle.child
                && let Ok(mut ch) = child.lock()
            {
                let _ = ch.kill();
            }
        }
    }

    // 2. Zeroize active vault key directly without logging to DB
    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut key) = guard.take() {
            key.zeroize();
        }
    }

    // 3. Remove all vault security and database files
    let files = [
        "vault_canary.bin",
        SALT_FILE,
        "vault.key",
        "local.key",
        "caframework.db",
        "caframework.db-wal",
        "caframework.db-shm",
        "caframework.db-journal",
        "known_hosts",
    ];
    for f in &files {
        let p = data_dir.join(f);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }

    Ok(())
}

/// Re-keys the vault to a new master password. The password's Argon2id output *is* the
/// SQLCipher key, so this rewrites every database page (`PRAGMA rekey`) and replaces the
/// canary. The vault stays unlocked, now on the new key.
pub fn change_master_password(
    current_password: &str,
    new_password: &str,
) -> Result<(), CatermError> {
    if current_password == new_password {
        return Err(vault_err(
            "Master password baru harus berbeda dari yang lama.",
        ));
    }
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);
    if !canary_path.exists() {
        return Err(vault_err("Vault belum dibuat."));
    }
    // The vault must already have been unlocked once in this run (that's how its in-memory key
    // got here in the first place), which means unlock_vault already migrated it off the legacy
    // static salt if it needed to — so the per-installation salt file is guaranteed to exist.
    let salt = load_or_create_salt(&data_dir)?;

    let mut old_key = derive_key(current_password, &salt)?;
    let mut new_key = match derive_key(new_password, &salt) {
        Ok(key) => key,
        Err(e) => {
            old_key.zeroize();
            return Err(e);
        }
    };

    let result = rekey(&data_dir, &canary_path, &old_key, &new_key);
    if result.is_ok() {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut previous) = guard.replace(new_key) {
            previous.zeroize();
        }
    }
    old_key.zeroize();
    new_key.zeroize();
    result?;

    let _ = crate::audit::log_event("VAULT_PASSWORD_CHANGE", None, "Master password changed");
    Ok(())
}

/// Ordering keeps a crash from locking the user out: the new canary is staged beside the old
/// one, the database is re-keyed, and only then is the staged canary moved into place. If that
/// last move fails, the database is re-keyed back so the untouched old canary still matches.
fn rekey(
    data_dir: &std::path::Path,
    canary_path: &std::path::Path,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<(), CatermError> {
    verify_canary(old_key, canary_path)?;

    let staged = canary_path.with_extension("bin.new");
    let encrypted_canary = crate::secret::encrypt_bytes(new_key, CANARY_PLAINTEXT)?;
    std::fs::write(&staged, encrypted_canary).map_err(|e| vault_err(e.to_string()))?;

    if let Err(e) = rekey_database(data_dir, old_key, new_key) {
        let _ = std::fs::remove_file(&staged);
        return Err(e);
    }

    if let Err(e) = std::fs::rename(&staged, canary_path) {
        let _ = rekey_database(data_dir, new_key, old_key);
        let _ = std::fs::remove_file(&staged);
        return Err(vault_err(format!("gagal menyimpan canary baru: {e}")));
    }
    Ok(())
}

fn rekey_database(
    data_dir: &std::path::Path,
    from: &[u8; 32],
    to: &[u8; 32],
) -> Result<(), CatermError> {
    let conn = crate::db::open_encrypted(data_dir, &hex::encode(from))?;
    conn.execute_batch(&format!("PRAGMA rekey = '{}';", hex::encode(to)))
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal rekey database: {e}"))))?;
    drop(conn);
    // Proves the new key opens the file before the caller switches the canary over.
    crate::db::open_encrypted(data_dir, &hex::encode(to)).map(drop)
}

pub fn lock_vault() -> Result<(), CatermError> {
    // Audit first: once the key is gone the database can no longer be opened to record it.
    let _ = crate::audit::log_event("VAULT_LOCK", None, "Vault locked");
    // Also stop all active tunnels on lock for Zero-Knowledge containment
    let _ = crate::tunnels::stop_all_tunnels();

    // Scoped so the write guard is released before returning. `parking_lot::RwLock` is not
    // reentrant: holding it across `log_event` (which reads the key to open the database)
    // deadlocked every lock, and every unlock queued behind it hung forever.
    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut key) = guard.take() {
            key.zeroize();
        }
    }
    Ok(())
}

pub fn unlock_vault(master_password: &str) -> Result<(), CatermError> {
    // Checked up front, before any filesystem access: a rejected short password must be a pure
    // no-op (no directory or salt file created), the same as it always was.
    check_password_len(master_password)?;

    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);
    let salt_path = data_dir.join(SALT_FILE);

    // A canary from before C-15, with no per-installation salt file yet: migrate it onto a real
    // random salt (keeping the same password) instead of deriving against the old shared one.
    if canary_path.exists() && !salt_path.exists() {
        return migrate_legacy_static_salt(master_password, &data_dir, &canary_path);
    }

    let salt = load_or_create_salt(&data_dir)?;
    let mut derived_key = derive_key(master_password, &salt)?;

    if canary_path.exists() {
        if let Err(e) = verify_canary(&derived_key, &canary_path) {
            derived_key.zeroize();
            return Err(e);
        }
    } else {
        std::fs::create_dir_all(&data_dir).map_err(|e| vault_err(e.to_string()))?;
        let encrypted_canary = crate::secret::encrypt_bytes(&derived_key, CANARY_PLAINTEXT)?;
        std::fs::write(&canary_path, encrypted_canary).map_err(|e| vault_err(e.to_string()))?;
    }

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        *guard = Some(derived_key);
    }

    let _ = crate::audit::log_event("VAULT_UNLOCK", None, "Vault unlocked");

    Ok(())
}

fn vault_err(message: impl Into<String>) -> CatermError {
    CatermError::Vault(VaultError::Generic(message.into()))
}

fn check_password_len(master_password: &str) -> Result<(), CatermError> {
    if master_password.len() < MIN_PASSWORD_LEN {
        return Err(vault_err(format!(
            "Master password minimal {MIN_PASSWORD_LEN} karakter"
        )));
    }
    Ok(())
}

/// Argon2id key from the master password and `salt` — both the canary key and the SQLCipher
/// key. `salt` should almost always come from [`load_or_create_salt`] (the real, random,
/// per-installation one); [`LEGACY_STATIC_SALT`] is the one narrow exception, used only to open
/// a not-yet-migrated vault during [`migrate_legacy_static_salt`].
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

/// Loads this installation's Argon2id salt, generating and persisting a fresh random one the
/// first time (brand-new vault, or right after [`migrate_legacy_static_salt`] has re-keyed an
/// existing one onto it).
fn load_or_create_salt(data_dir: &std::path::Path) -> Result<[u8; SALT_LEN], CatermError> {
    let path = data_dir.join(SALT_FILE);
    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(salt) = <[u8; SALT_LEN]>::try_from(bytes.as_slice())
    {
        return Ok(salt);
    }

    std::fs::create_dir_all(data_dir).map_err(|e| vault_err(e.to_string()))?;
    let mut salt = [0u8; SALT_LEN];
    use ssh_key::rand_core::RngCore;
    ssh_key::rand_core::OsRng.fill_bytes(&mut salt);
    std::fs::write(&path, salt).map_err(|e| vault_err(e.to_string()))?;
    Ok(salt)
}

/// One-time migration (C-15) for a vault created before per-installation salts existed, when
/// every installation derived its KEK with the same hardcoded [`LEGACY_STATIC_SALT`]. Opens the
/// vault with that legacy salt, generates a real random salt, and re-keys the database and
/// canary onto a key derived with it — reusing [`rekey`]'s same staged-write, verify-before-
/// commit discipline as `change_master_password`, so a crash mid-migration can't corrupt the
/// vault or lock the user out of it. The new salt is written to disk only after the database and
/// canary are already keyed to match it; if that write fails, the re-key is rolled back so the
/// vault is left consistently on one salt or the other, never keyed with a salt that exists only
/// in this process's memory.
fn migrate_legacy_static_salt(
    master_password: &str,
    data_dir: &std::path::Path,
    canary_path: &std::path::Path,
) -> Result<(), CatermError> {
    let mut legacy_key = derive_key(master_password, LEGACY_STATIC_SALT)?;
    if let Err(e) = verify_canary(&legacy_key, canary_path) {
        legacy_key.zeroize();
        return Err(e);
    }

    let mut salt = [0u8; SALT_LEN];
    use ssh_key::rand_core::RngCore;
    ssh_key::rand_core::OsRng.fill_bytes(&mut salt);
    let mut new_key = match derive_key(master_password, &salt) {
        Ok(key) => key,
        Err(e) => {
            legacy_key.zeroize();
            return Err(e);
        }
    };

    if let Err(e) = rekey(data_dir, canary_path, &legacy_key, &new_key) {
        legacy_key.zeroize();
        new_key.zeroize();
        return Err(e);
    }

    if let Err(e) = std::fs::write(data_dir.join(SALT_FILE), salt) {
        let rollback = rekey(data_dir, canary_path, &new_key, &legacy_key);
        legacy_key.zeroize();
        new_key.zeroize();
        return match rollback {
            Ok(()) => Err(vault_err(format!("gagal menyimpan salt baru: {e}"))),
            Err(re) => Err(vault_err(format!(
                "gagal menyimpan salt baru ({e}) dan gagal mengembalikan vault ke kondisi semula ({re}) — hubungi dukungan"
            ))),
        };
    }
    legacy_key.zeroize();

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut previous) = guard.replace(new_key) {
            previous.zeroize();
        }
    }

    let _ = crate::audit::log_event(
        "VAULT_UNLOCK",
        None,
        "Vault unlocked (migrated to per-installation Argon2id salt)",
    );
    Ok(())
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

pub fn load_or_create_local_key(data_dir: &std::path::Path) -> Result<String, CatermError> {
    let path = data_dir.join("vault.key");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    std::fs::create_dir_all(data_dir)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let mut bytes = [0u8; 32];
    use ssh_key::rand_core::RngCore;
    ssh_key::rand_core::OsRng.fill_bytes(&mut bytes);
    let key: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    std::fs::write(&path, &key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    Ok(key)
}

pub fn get_active_dek() -> Result<[u8; 32], CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    if let Some(key) = *guard {
        Ok(key)
    } else {
        Err(CatermError::Vault(VaultError::Generic(
            "Vault is locked".into(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_password() {
        assert!(unlock_vault("short").is_err());
    }

    #[test]
    fn accepts_password_at_minimum_length_and_verifies_correctly() {
        // Isolated: this used to build a temp dir and then unlock the vault in the real data
        // directory anyway, creating a vault with password "12345678" on a fresh machine.
        let _data = crate::test_support::isolated_data_dir("vault_unlock");

        // Test lock and unlock
        assert!(unlock_vault("12345678").is_ok());
        assert!(is_unlocked().unwrap());
        assert!(lock_vault().is_ok());
        assert!(!is_unlocked().unwrap());
        // Regression: lock_vault used to keep the key's write lock while logging, so it never
        // returned and this second unlock blocked forever.
        assert!(unlock_vault("12345678").is_ok());
        assert!(is_unlocked().unwrap());
        assert!(lock_vault().is_ok());
    }

    #[test]
    fn rekey_moves_database_and_canary_to_the_new_key() {
        let dir = std::env::temp_dir().join(format!("caframework_vault_rekey_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let canary = dir.join(CANARY_FILE);
        let (old_key, new_key) = ([7u8; 32], [9u8; 32]);

        std::fs::write(
            &canary,
            crate::secret::encrypt_bytes(&old_key, CANARY_PLAINTEXT).unwrap(),
        )
        .unwrap();
        {
            let conn = crate::db::open_encrypted(&dir, &hex::encode(old_key)).unwrap();
            conn.execute_batch("CREATE TABLE probe(v TEXT); INSERT INTO probe VALUES ('kept');")
                .unwrap();
        }

        assert!(
            rekey(&dir, &canary, &new_key, &old_key).is_err(),
            "wrong current key must fail"
        );
        rekey(&dir, &canary, &old_key, &new_key).unwrap();

        assert!(verify_canary(&new_key, &canary).is_ok());
        assert!(verify_canary(&old_key, &canary).is_err());
        assert!(crate::db::open_encrypted(&dir, &hex::encode(old_key)).is_err());
        let conn = crate::db::open_encrypted(&dir, &hex::encode(new_key)).unwrap();
        let kept: String = conn
            .query_row("SELECT v FROM probe", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kept, "kept");
        assert!(!canary.with_extension("bin.new").exists());
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_key_is_stable_across_calls() {
        let dir = std::env::temp_dir().join("caframework_vault_test_temp");
        let _ = std::fs::create_dir_all(&dir);
        let k1 = load_or_create_local_key(&dir).unwrap();
        let k2 = load_or_create_local_key(&dir).unwrap();
        assert_eq!(k1, k2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn derive_key_uses_the_salt_it_is_given() {
        // Sanity check that the salt actually participates in the derivation — i.e. this isn't
        // just C-15's hardcoded salt moved one parameter over.
        let a = derive_key("some-password", b"salt-one-salt-one").unwrap();
        let b = derive_key("some-password", b"salt-two-salt-two").unwrap();
        assert_ne!(a, b, "different salts must not derive the same key");
    }

    #[test]
    fn unlock_vault_migrates_a_legacy_static_salt_vault_onto_a_random_per_install_salt() {
        // C-15 regression test: every installation used to derive its KEK with the same
        // hardcoded salt. Hand-build a vault exactly as the old code would have left one — canary
        // and database both keyed off LEGACY_STATIC_SALT, no vault_salt.bin — and confirm
        // unlock_vault detects that shape and migrates it onto a real random salt, in place,
        // without losing the data or requiring a new password.
        let data = crate::test_support::isolated_data_dir("vault_salt_migration");
        let data_dir = data.path.clone();
        let canary_path = data_dir.join(CANARY_FILE);
        let salt_path = data_dir.join(SALT_FILE);
        let password = "correct horse battery staple";

        std::fs::create_dir_all(&data_dir).unwrap();
        let legacy_key = derive_key(password, LEGACY_STATIC_SALT).unwrap();
        std::fs::write(
            &canary_path,
            crate::secret::encrypt_bytes(&legacy_key, CANARY_PLAINTEXT).unwrap(),
        )
        .unwrap();
        {
            let conn = crate::db::open_encrypted(&data_dir, &hex::encode(legacy_key)).unwrap();
            conn.execute_batch("CREATE TABLE probe(v TEXT); INSERT INTO probe VALUES ('kept');")
                .unwrap();
        }
        assert!(
            !salt_path.exists(),
            "test setup should not have created a salt file yet"
        );

        unlock_vault(password).expect("legacy vault should unlock and migrate");

        assert!(is_unlocked().unwrap());
        assert!(
            salt_path.exists(),
            "migration should persist a per-installation salt"
        );

        let salt_bytes = std::fs::read(&salt_path).unwrap();
        assert_eq!(salt_bytes.len(), SALT_LEN);
        assert_ne!(
            salt_bytes.as_slice(),
            LEGACY_STATIC_SALT,
            "the persisted salt must not just be the old static one"
        );
        let expected_key = derive_key(password, &salt_bytes).unwrap();
        assert_eq!(get_active_dek().unwrap(), expected_key);

        // The database is re-keyed too, not just the canary: the old key must no longer open
        // it...
        assert!(crate::db::open_encrypted(&data_dir, &hex::encode(legacy_key)).is_err());
        // ...and the new one does, with the data intact.
        let conn = crate::db::open_encrypted(&data_dir, &hex::encode(expected_key)).unwrap();
        let kept: String = conn
            .query_row("SELECT v FROM probe", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kept, "kept");
    }
}
