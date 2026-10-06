//! Encrypted JSON Vault Backup & Restore.
//! Exports all profiles, notes, and kv records as an encrypted JSON archive.

use crate::error::{CafError, VaultError};
use crate::notes::{self, NoteRecord};
use crate::profiles::{self, ProfileRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBackupPayload {
    pub version: String,
    pub timestamp: u64,
    pub profiles: Vec<ProfileRecord>,
    pub notes: Vec<NoteRecord>,
}

pub fn export_backup(passphrase: &str) -> Result<Vec<u8>, CafError> {
    if passphrase.len() < crate::vault::MIN_PASSWORD_LEN {
        return Err(CafError::Vault(VaultError::Generic(format!(
            "Backup passphrase minimal {} karakter",
            crate::vault::MIN_PASSWORD_LEN
        ))));
    }

    let profiles = profiles::list_profiles()?;
    let notes = notes::list_notes("backup-root", true)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let payload = VaultBackupPayload {
        version: crate::CORE_VERSION.to_string(),
        timestamp: now,
        profiles,
        notes,
    };

    let json_bytes = serde_json::to_vec(&payload)
        .map_err(|e| CafError::Vault(VaultError::Generic(e.to_string())))?;

    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CafError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caframework.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CafError::Vault(VaultError::Generic(e.to_string())))?;

    let encrypted = crate::secret::encrypt_bytes(&derived_key, &json_bytes)?;
    use zeroize::Zeroize;
    derived_key.zeroize();
    Ok(encrypted.into_bytes())
}

pub fn import_backup(data: &[u8], passphrase: &str) -> Result<(), CafError> {
    let encrypted_str = std::str::from_utf8(data).map_err(|e| {
        CafError::Vault(VaultError::Generic(format!("Invalid backup data: {e}")))
    })?;

    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CafError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caframework.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CafError::Vault(VaultError::Generic(e.to_string())))?;

    let decrypted = crate::secret::decrypt_bytes(&derived_key, encrypted_str)
        .map_err(|_| CafError::Vault(VaultError::Generic("Password backup salah.".into())))?;

    let payload: VaultBackupPayload = serde_json::from_slice(&decrypted)
        .map_err(|e| CafError::Vault(VaultError::Generic(format!("Corrupt backup: {e}"))))?;

    for p in payload.profiles {
        let _ = profiles::save_profile(
            profiles::ProfileInput {
                id: Some(p.id),
                name: p.name,
                role: p.role,
                avatar: p.avatar,
                pin: None,
            },
            "import-root",
        );
    }

    for n in payload.notes {
        let _ = notes::save_note(
            notes::NoteInput {
                id: Some(n.id.clone()),
                title: n.title,
                content: n.content,
                tags: n.tags,
                visibility: Some(n.visibility),
                owner_profile_id: n.owner_profile_id.clone(),
            },
            &n.owner_profile_id,
        );
    }

    Ok(())
}
