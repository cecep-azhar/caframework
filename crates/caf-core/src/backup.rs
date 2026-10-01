//! Encrypted JSON Vault Backup & Restore (`T2-EXIM-01`).
//! Exports all hosts, groups, snippets, and keys as an encrypted JSON archive.
//! Imports and merges encrypted backup archives with integrity validation.

use crate::error::{CatermError, VaultError};
use crate::groups::{self, GroupInput, GroupRecord};
use crate::keys::{self, KeyRecord};
use crate::snippets::{self, SnippetInput, SnippetRecord};
use crate::store::{self, HostInput, HostRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBackupPayload {
    pub version: String,
    pub timestamp: u64,
    pub hosts: Vec<HostRecord>,
    pub groups: Vec<GroupRecord>,
    pub snippets: Vec<SnippetRecord>,
    pub keys: Vec<KeyRecord>,
}

pub fn export_encrypted_backup(passphrase: &str) -> Result<String, CatermError> {
    if passphrase.len() < crate::vault::MIN_PASSWORD_LEN {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "Backup passphrase must be at least {} characters",
            crate::vault::MIN_PASSWORD_LEN
        ))));
    }

    let hosts = store::list_hosts()?;
    let groups = groups::list_groups()?;
    let snippets = snippets::list_snippets()?;
    let keys = keys::list_keys()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let payload = VaultBackupPayload {
        version: "2.0.10".to_string(),
        timestamp: now,
        hosts,
        groups,
        snippets,
        keys,
    };

    let json_bytes = serde_json::to_vec(&payload)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    // Encrypt with passphrase derived key using Argon2id + AES-256-GCM
    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caframework.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let encrypted_b64 = crate::secret::encrypt_bytes(&derived_key, &json_bytes)?;
    Ok(encrypted_b64)
}

pub fn import_encrypted_backup(
    encrypted_b64: &str,
    passphrase: &str,
) -> Result<usize, CatermError> {
    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caframework.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let decrypted_bytes = crate::secret::decrypt_bytes(&derived_key, encrypted_b64)?;
    let payload: VaultBackupPayload = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!("Corrupt backup file: {}", e)))
    })?;

    let mut imported_count = 0;

    for host in payload.hosts {
        store::save_host(HostInput {
            id: Some(host.id),
            label: host.label,
            address: host.address,
            port: host.port,
            username: host.username,
            auth_method: host.auth_method,
            tags: host.tags,
            os: host.os,
            protocol: Some(host.protocol),
            secret: None,
        })?;
        imported_count += 1;
    }

    for group in payload.groups {
        groups::save_group(GroupInput {
            id: Some(group.id),
            name: group.name,
            color: group.color,
            host_ids: group.host_ids,
        })?;
        imported_count += 1;
    }

    for snippet in payload.snippets {
        snippets::save_snippet(SnippetInput {
            id: Some(snippet.id),
            label: snippet.label,
            description: snippet.description,
            command: snippet.command,
            tags: snippet.tags,
        })?;
        imported_count += 1;
    }

    for key in payload.keys {
        keys::import_key(&key.name, &key.public_key, None)?;
        imported_count += 1;
    }

    Ok(imported_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_roundtrip() {
        // We can test this by mocking the store, but here we can at least test that
        // encryption/decryption works and doesn't leak plaintext.
        let passphrase = "correct-horse-battery";
        let payload = VaultBackupPayload {
            version: "2.0.10".to_string(),
            timestamp: 1234567890,
            hosts: vec![HostRecord {
                id: "h1".into(),
                label: "Test".into(),
                address: "127.0.0.1".into(),
                port: 22,
                username: "root".into(),
                auth_method: crate::store::AuthMethod::Password,
                tags: vec![],
                os: None,
                protocol: crate::store::ConnectionProtocol::Ssh,
                created_at: 1,
                updated_at: 1,
                has_secret: false,
            }],
            groups: vec![],
            snippets: vec![],
            keys: vec![],
        };
        let json_bytes = serde_json::to_vec(&payload).unwrap();

        // Encrypt with passphrase derived key using Argon2id + AES-256-GCM
        let mut derived_key = [0u8; 32];
        let params = argon2::Params::new(64 * 1024, 3, 4, Some(32)).unwrap();
        let argon2 =
            argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
        let salt = b"caframework.vault.backup.salt.2026";
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
            .unwrap();

        let encrypted_b64 = crate::secret::encrypt_bytes(&derived_key, &json_bytes).unwrap();

        // Ensure no plaintext leakage
        assert!(!encrypted_b64.contains("Test"));
        assert!(!encrypted_b64.contains("127.0.0.1"));

        // Decrypt
        let decrypted_bytes = crate::secret::decrypt_bytes(&derived_key, &encrypted_b64).unwrap();
        let decrypted_payload: VaultBackupPayload =
            serde_json::from_slice(&decrypted_bytes).unwrap();

        assert_eq!(decrypted_payload.hosts[0].label, "Test");
    }
}
