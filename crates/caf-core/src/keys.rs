//! Managed SSH Keys module (`T2-TOOL-05`).
//! Provides key generation (Ed25519, RSA-4096), import (OpenSSH, PKCS#8),
//! fingerprinting (SHA-256), and safe storage inside the encrypted SQLite vault.

use crate::error::{CatermError, ValidationError};
use serde::{Deserialize, Serialize};
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyRecord {
    pub id: String,
    pub name: String,
    pub algorithm: String,
    pub fingerprint: String,
    pub public_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyInput {
    pub name: String,
    pub algorithm: String, // "Ed25519" or "RSA-4096"
}

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "{field} cannot be empty"
        ))));
    }
    Ok(())
}

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("key-{nanos:x}")
}

/// Create the `ssh_keys` database table if it doesn't already exist.
pub fn init_table(conn: &rusqlite::Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ssh_keys (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            algorithm TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            public_key TEXT NOT NULL,
            secret_enc BLOB NOT NULL,
            created_at TEXT NOT NULL
        );",
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to create ssh_keys table: {e}"
        )))
    })?;
    Ok(())
}

/// List all stored public keys.
pub fn list_keys() -> Result<Vec<KeyRecord>, CatermError> {
    let data_info = crate::paths::resolve_data_dir()?;
    // Listing never decrypts anything, but the local key must exist before the DB is opened.
    let _key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open()?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, name, algorithm, fingerprint, public_key, created_at FROM ssh_keys ORDER BY name ASC")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(KeyRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                algorithm: row.get(2)?,
                fingerprint: row.get(3)?,
                public_key: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result
            .push(r.map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?);
    }
    Ok(result)
}

/// Generate a new Ed25519 or RSA-4096 SSH key pair.
pub fn generate_key(input: KeyInput) -> Result<KeyRecord, CatermError> {
    require_non_empty("name", &input.name)?;

    let mut rng = rand::thread_rng();
    let priv_key = match input.algorithm.as_str() {
        "Ed25519" => PrivateKey::random(&mut rng, Algorithm::Ed25519).map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Failed to generate Ed25519 key: {e}"
            )))
        })?,
        "RSA-4096" | "RSA" => {
            PrivateKey::random(&mut rng, Algorithm::Rsa { hash: None }).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to generate RSA key: {e}"
                )))
            })?
        }
        other => {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "Algorithm '{other}' is not supported. Use 'Ed25519' or 'RSA-4096'."
            ))));
        }
    };

    let fingerprint = priv_key.fingerprint(HashAlg::Sha256).to_string();
    let public_key = priv_key.public_key().to_openssh().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to encode public key: {e}"
        )))
    })?;

    let private_key_pem = priv_key.to_openssh(LineEnding::LF).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to encode private key: {e}"
        )))
    })?;

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let encrypted_secret = crate::secret::encrypt(&key, private_key_pem.as_str())?;

    let id = generate_id();
    let created_at = chrono::Utc::now().to_rfc3339();

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO ssh_keys (id, name, algorithm, fingerprint, public_key, secret_enc, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            input.name,
            input.algorithm,
            fingerprint,
            public_key,
            encrypted_secret,
            created_at
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save key: {e}"))))?;

    Ok(KeyRecord {
        id,
        name: input.name,
        algorithm: input.algorithm,
        fingerprint,
        public_key,
        created_at,
    })
}

/// Import an existing OpenSSH or PKCS#8 private key.
pub fn import_key(
    name: &str,
    private_key_pem: &str,
    passphrase: Option<&str>,
) -> Result<KeyRecord, CatermError> {
    require_non_empty("name", name)?;
    require_non_empty("private_key_pem", private_key_pem)?;

    let parsed_key = if let Some(pass) = passphrase {
        PrivateKey::from_openssh(private_key_pem)
            .and_then(|k| k.decrypt(pass))
            .map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to decrypt private key with passphrase: {e}"
                )))
            })?
    } else {
        PrivateKey::from_openssh(private_key_pem).map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Invalid OpenSSH private key: {e}"
            )))
        })?
    };

    let algorithm = match parsed_key.algorithm() {
        Algorithm::Ed25519 => "Ed25519".to_string(),
        Algorithm::Rsa { .. } => "RSA".to_string(),
        other => format!("{other:?}"),
    };

    let fingerprint = parsed_key.fingerprint(HashAlg::Sha256).to_string();
    let public_key = parsed_key.public_key().to_openssh().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to get public key: {e}"
        )))
    })?;

    let clean_pem = parsed_key.to_openssh(LineEnding::LF).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to encode key: {e}"
        )))
    })?;

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let encrypted_secret = crate::secret::encrypt(&key, clean_pem.as_str())?;

    let id = generate_id();
    let created_at = chrono::Utc::now().to_rfc3339();

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO ssh_keys (id, name, algorithm, fingerprint, public_key, secret_enc, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            name,
            algorithm,
            fingerprint,
            public_key,
            encrypted_secret,
            created_at
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save key: {e}"))))?;

    Ok(KeyRecord {
        id,
        name: name.to_string(),
        algorithm,
        fingerprint,
        public_key,
        created_at,
    })
}

/// Delete an SSH key, ensuring it is not currently assigned to any saved Host.
pub fn delete_key(id: &str) -> Result<(), CatermError> {
    require_non_empty("id", id)?;

    // Check if key is in use by any Host
    let hosts = crate::store::list_hosts()?;
    for h in hosts {
        if let crate::store::AuthMethod::KeyId { id: ref k_id } = h.auth_method
            && k_id == id
        {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "Failed to delete: Key is currently in use by Host '{}'. Please change host authentication method first.",
                h.label
            ))));
        }
    }

    let conn = crate::db::open()?;
    init_table(&conn)?;

    let affected = conn
        .execute("DELETE FROM ssh_keys WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    if affected == 0 {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "Key with id '{id}' not found"
        ))));
    }

    Ok(())
}

/// Retrieve internal decrypted private key PEM for SSH authentication.
pub fn get_private_key(id: &str) -> Result<String, CatermError> {
    require_non_empty("id", id)?;

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open()?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT secret_enc FROM ssh_keys WHERE id = ?1")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let secret_enc: String = stmt
        .query_row(rusqlite::params![id], |row| row.get(0))
        .map_err(|_| {
            CatermError::Validation(ValidationError::Generic(format!("Key '{id}' not found")))
        })?;

    let decrypted = crate::secret::decrypt(&key, &secret_enc)?;
    Ok(decrypted)
}

/// Builds the idempotent "append this key if it isn't already there" remote command.
/// `pub_key_line` includes the key's free-form comment field, which an OpenSSH public key's own
/// format never disallows a `'` in — this is the one place that text reaches a shell command
/// string, so it goes through `shell_quote` (the POSIX single-quote escaping already used for
/// SFTP's compress/extract commands) rather than being interpolated raw.
fn build_deploy_key_cmd(pub_key_line: &str) -> String {
    let q_key = crate::sftp::shell_quote(pub_key_line);
    format!(
        "mkdir -p ~/.ssh && chmod 700 ~/.ssh && touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && \
         grep -qF {q_key} ~/.ssh/authorized_keys || echo {q_key} >> ~/.ssh/authorized_keys"
    )
}

/// Deploy a public key from the local vault to a remote host's `~/.ssh/authorized_keys`.
pub fn deploy_public_key(host_id: &str, key_id: &str) -> Result<(), CatermError> {
    require_non_empty("host_id", host_id)?;
    require_non_empty("key_id", key_id)?;

    // Get public key content
    let keys = list_keys()?;
    let target_key = keys.iter().find(|k| k.id == key_id).ok_or_else(|| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Key '{key_id}' not found"
        )))
    })?;

    // Connect to target host. Shared path = same TOFU host key verification as everything
    // else; this used to be a hand-rolled copy of the connect logic that skipped it entirely.
    let (sess, host) = crate::ssh::open_authenticated_session(host_id)?;

    // Deploy public key idempotently & fix permissions
    let mut channel = sess.channel_session().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Channel creation failed: {e}"
        )))
    })?;

    let cmd = build_deploy_key_cmd(target_key.public_key.trim());

    channel.exec(&cmd).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Exec deploy key failed: {e}"
        )))
    })?;

    // Previously unchecked: `wait_close().ok()` discarded any failure, and the function returned
    // `Ok(())` (and logged "Deployed key...") even if the remote commands above failed outright —
    // no write permission on ~/.ssh, a full disk, a read-only home directory. Capture stderr and
    // the exit status so a failed deploy is reported as one instead of silently claimed to have
    // succeeded.
    let mut stderr = String::new();
    {
        use std::io::Read;
        let _ = channel.stderr().read_to_string(&mut stderr);
    }
    channel.wait_close().ok();
    let exit = channel.exit_status().unwrap_or(1);
    if exit != 0 {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "Deploying key to {} failed (exit {exit}): {}",
            host.label,
            stderr.trim()
        ))));
    }

    let _ = crate::audit::log_event(
        "KEY_DEPLOY",
        Some(&host.id),
        &format!("Deployed key '{}' to host {}", target_key.name, host.label),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_list_ed25519_key() {
        let _data = crate::test_support::isolated_data_dir("keys");
        crate::vault::unlock_vault("12345678").unwrap();

        let input = KeyInput {
            name: "Test Key".to_string(),
            algorithm: "Ed25519".to_string(),
        };
        let key = generate_key(input).expect("Key generation failed");
        assert!(key.fingerprint.starts_with("SHA256:"));
        assert!(key.public_key.starts_with("ssh-ed25519"));

        let keys = list_keys().expect("List keys failed");
        assert!(keys.iter().any(|k| k.id == key.id));

        let priv_key = get_private_key(&key.id).expect("Get private key failed");
        assert!(priv_key.contains("BEGIN OPENSSH PRIVATE KEY"));

        delete_key(&key.id).expect("Delete key failed");
    }

    #[test]
    fn build_deploy_key_cmd_does_not_let_a_malicious_comment_inject_commands() {
        // C-19: a public key's comment field is free-form text an OpenSSH key's own format
        // never disallows a `'` in. Deploy used to interpolate it into a single-quoted remote
        // shell string unescaped, so a comment like `x'; touch pwned; echo 'x` would close the
        // quote early and run `touch pwned` on the remote host. This runs the actual built
        // command through a real shell (locally — no SSH needed, since the vulnerability is in
        // the string, not the transport) and proves the "payload" never executes.
        let tmp = std::env::temp_dir().join(format!("caframework_c19_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).expect("create scratch dir");
        let marker = tmp.join("pwned");

        let malicious_comment = format!(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIA comment'; touch {} ; echo '",
            marker.display()
        );
        let cmd = build_deploy_key_cmd(&malicious_comment);

        // The command targets `~/.ssh/authorized_keys` — override HOME rather than `cd`, so
        // `~` expands into the scratch dir instead of the real one.
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .env("HOME", &tmp)
            .status()
            .expect("run built command through a real shell");

        assert!(
            !marker.exists(),
            "the injected `touch` must never run: {cmd}"
        );
        // The grep-or-append idiom itself should still work end to end: exit 0 (append
        // succeeded, since the file was just created empty) and the line landed in the file
        // exactly once, comment and all.
        assert!(
            status.success(),
            "the deploy command itself should still succeed: {cmd}"
        );
        let authorized_keys =
            std::fs::read_to_string(tmp.join(".ssh/authorized_keys")).expect("read result");
        assert_eq!(authorized_keys.trim(), malicious_comment.trim());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
