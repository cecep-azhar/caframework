//! Encrypted SQLite storage engine (T9a). Every record `caf-core` persists lives inside a
//! single SQLCipher-encrypted file at `<data_dir>/caframework.db` — see [`crate::vault`] for where
//! the passphrase that keys it comes from. This replaces the earlier plaintext `hosts.json`
//! store that `store.rs` used before the vault existed.

use crate::error::{CatermError, DbError};
use rusqlite::Connection;
use std::path::Path;

/// Opens (creating if absent) the SQLCipher-encrypted database at `<data_dir>/caframework.db`,
/// keys it with `passphrase`, and ensures the schema exists. SQLCipher only actually
/// verifies a key lazily on first real read, so this runs a canary query up front to fail
/// fast with a clear error when the passphrase doesn't match an existing file.
///
/// When `passphrase` is a 64-character hex string — which is what `db::open()` always passes
/// (the vault DEK, already 256 bits of Argon2id output, hex-encoded) — this keys SQLCipher
/// with it as **raw key material** (`PRAGMA key = "x'...'"`) instead of treating it as a
/// passphrase. Passphrase mode runs SQLCipher's own PBKDF2 (256,000 iterations) over the
/// string on every single open, which measured ~156ms/call here; raw-key mode skips that KDF
/// entirely (~0.2ms) since the key is already high-entropy and doesn't need stretching. A
/// database created before this change is still keyed the slow way, so the raw-key attempt
/// below fails against its real content; that failure is expected and triggers a one-time
/// fallback that opens it the old way and `PRAGMA rekey`s it to raw-key form, so every open
/// after that is fast. Any other passphrase (tests, or a future caller not passing the DEK)
/// is not touched by any of this and keeps the original passphrase-mode behavior.
pub fn open_encrypted(data_dir: &Path, passphrase: &str) -> Result<Connection, CatermError> {
    std::fs::create_dir_all(data_dir)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal membuat data dir: {e}"))))?;
    let db_path = crate::paths::db_path(data_dir);

    let conn = if let Some(raw_key) = raw_key_literal(passphrase) {
        match open_keyed(&db_path, &raw_key) {
            Ok(conn) => conn,
            Err(_) => {
                // Not on the fast path yet: open with the legacy passphrase-derived key, then
                // rekey to raw so this (and every later) open is fast from now on.
                let conn = open_keyed(&db_path, &passphrase_literal(passphrase))?;
                conn.execute_batch(&format!("PRAGMA rekey = {raw_key};"))
                    .map_err(|e| {
                        CatermError::Db(DbError::Generic(format!(
                            "failed to migrate database to raw-key mode: {e}"
                        )))
                    })?;
                conn
            }
        }
    } else {
        open_keyed(&db_path, &passphrase_literal(passphrase))?
    };

    init_schema(&conn)?;
    migrate_hosts_secret_column(&conn)?;
    migrate_hosts_os_column(&conn)?;
    migrate_hosts_protocol_column(&conn)?;
    Ok(conn)
}

/// `passphrase` as a `PRAGMA key` string literal. `PRAGMA key` doesn't support bound
/// parameters, so single quotes are escaped defensively rather than interpolated raw.
fn passphrase_literal(passphrase: &str) -> String {
    format!("'{}'", passphrase.replace('\'', "''"))
}

/// If `passphrase` is exactly a 64-character hex string (a hex-encoded 256-bit key, which is
/// all `db::open()` ever passes), returns it as a SQLCipher raw-key literal (`x'...'`). Hex
/// digits need no escaping, so this is safe to interpolate directly.
fn raw_key_literal(passphrase: &str) -> Option<String> {
    if passphrase.len() == 64 && passphrase.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(format!("\"x'{passphrase}'\""))
    } else {
        None
    }
}

/// Opens a fresh connection to `db_path` and keys it with `key_literal` (already a valid
/// `PRAGMA key` argument — either a quoted passphrase or an `x'...'` raw key), verifying the
/// key with a canary query before returning it.
fn open_keyed(db_path: &Path, key_literal: &str) -> Result<Connection, CatermError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal membuka database: {e}"))))?;

    conn.execute_batch(&format!(
        "PRAGMA key = {key_literal};\nPRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;"
    ))
    .map_err(|e| CatermError::Db(DbError::Generic(format!("failed to set vault key: {e}"))))?;

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| {
            CatermError::Db(DbError::Generic(
                "invalid vault key or corrupted database".into(),
            ))
        })?;

    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS hosts (
           id TEXT PRIMARY KEY,
           label TEXT NOT NULL,
           address TEXT NOT NULL,
           port INTEGER NOT NULL,
           username TEXT NOT NULL,
           auth_method TEXT NOT NULL,
           tags TEXT NOT NULL,
           os TEXT,
           protocol TEXT NOT NULL DEFAULT 'ssh',
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
        );
         CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            host_ids TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            avatar TEXT,
            members TEXT NOT NULL,
            host_ids TEXT NOT NULL,
            group_ids TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS snippets (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            description TEXT NOT NULL,
            command TEXT NOT NULL,
            tags TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS command_logs (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            host_id TEXT,
            details TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS investigations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            host_id TEXT,
            status TEXT NOT NULL,
            notes TEXT NOT NULL,
            evidence TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS ai_settings (
            id TEXT PRIMARY KEY,
            provider TEXT NOT NULL,
            api_key TEXT NOT NULL,
            base_url TEXT NOT NULL,
            model TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS transfer_resumable (
            id TEXT PRIMARY KEY,
            host_id TEXT NOT NULL,
            direction TEXT NOT NULL,
            remote_path TEXT NOT NULL,
            local_path TEXT NOT NULL,
            resume_offset INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'Queued',
            speed_limit_bps INTEGER,
            priority INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
         );",
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal inisialisasi skema: {e}"))))
}

/// Adds the `secret_enc` column to `hosts` for databases created before host credential
/// encryption existed. `CREATE TABLE IF NOT EXISTS` above never alters an existing table,
/// so this migration is what actually gets older `caframework.db` files up to date.
fn migrate_hosts_secret_column(conn: &Connection) -> Result<(), CatermError> {
    let has_column: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('hosts') WHERE name = 'secret_enc'")
        .and_then(|mut stmt| stmt.exists([]))
        .unwrap_or(false);
    if !has_column {
        conn.execute_batch("ALTER TABLE hosts ADD COLUMN secret_enc TEXT")
            .map_err(|e| {
                CatermError::Db(DbError::Generic(format!(
                    "gagal migrasi kolom secret_enc: {e}"
                )))
            })?;
    }
    Ok(())
}

fn migrate_hosts_os_column(conn: &Connection) -> Result<(), CatermError> {
    let has_column: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('hosts') WHERE name = 'os'")
        .and_then(|mut stmt| stmt.exists([]))
        .unwrap_or(false);
    if !has_column {
        conn.execute_batch("ALTER TABLE hosts ADD COLUMN os TEXT")
            .map_err(|e| {
                CatermError::Db(DbError::Generic(format!("gagal migrasi kolom os: {e}")))
            })?;
    }
    Ok(())
}

fn migrate_hosts_protocol_column(conn: &Connection) -> Result<(), CatermError> {
    let has_column: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('hosts') WHERE name = 'protocol'")
        .and_then(|mut stmt| stmt.exists([]))
        .unwrap_or(false);
    if !has_column {
        conn.execute_batch("ALTER TABLE hosts ADD COLUMN protocol TEXT NOT NULL DEFAULT 'ssh'")
            .map_err(|e| {
                CatermError::Db(DbError::Generic(format!(
                    "gagal migrasi kolom protocol: {e}"
                )))
            })?;
    }
    Ok(())
}

/// Opens the encrypted database at the resolved data dir, keyed with the local vault key.
pub fn open() -> Result<Connection, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let key = crate::vault::get_active_dek()?;
    let key_hex = hex::encode(key);
    open_encrypted(&data_dir, &key_hex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDataDir(PathBuf);

    impl TempDataDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            Self(std::env::temp_dir().join(format!("caframework_db_test_{label}_{nanos:x}")))
        }
    }

    impl Drop for TempDataDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn wrong_passphrase_is_rejected() {
        let dir = TempDataDir::new("wrong_pass");
        {
            let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buat db");
            drop(conn);
        }
        let reopened = open_encrypted(&dir.0, "totally-different-key");
        assert!(reopened.is_err());
    }

    #[test]
    fn hex_key_opens_reopens_and_rejects_a_different_hex_key() {
        // A 64-hex-char passphrase (what db::open() always passes) takes the raw-key path.
        let dir = TempDataDir::new("hex_key");
        let key_a = "a".repeat(64);
        let key_b = "b".repeat(64);
        {
            let conn = open_encrypted(&dir.0, &key_a).expect("create with hex key");
            conn.execute(
                "INSERT INTO hosts (id, label, address, port, username, auth_method, tags, created_at, updated_at)
                 VALUES ('h1', 'Test', '10.0.0.1', 22, 'root', '{\"type\":\"password\"}', '[]', 1, 1)",
                [],
            )
            .expect("insert");
        }
        let reopened = open_encrypted(&dir.0, &key_a).expect("reopen with same hex key");
        let count: i64 = reopened
            .query_row("SELECT count(*) FROM hosts", [], |r| r.get(0))
            .expect("query hosts");
        assert_eq!(count, 1);

        assert!(open_encrypted(&dir.0, &key_b).is_err());
    }

    #[test]
    fn a_database_keyed_the_old_slow_way_is_migrated_to_raw_key_on_first_open() {
        // Regression for C-08: before this change, db::open() always keyed SQLCipher with the
        // hex DEK as a *passphrase*, so every connection paid SQLCipher's PBKDF2 (256,000
        // iterations, ~156ms measured). Simulate a database created that way (bypassing
        // open_encrypted, which now prefers the raw-key path for any caller), then verify
        // open_encrypted still opens it (the one-time fallback), and that afterwards the file
        // is truly keyed by raw bytes — not still falling back on every call — by opening it
        // again with nothing but the raw-key PRAGMA and no passphrase fallback at all.
        let dir = TempDataDir::new("migrate_legacy");
        let hex_key = "c".repeat(64);
        std::fs::create_dir_all(&dir.0).expect("mkdir");
        let db_path = crate::paths::db_path(&dir.0);
        {
            let conn = Connection::open(&db_path).expect("open raw");
            conn.execute_batch(&format!("PRAGMA key = '{hex_key}';"))
                .expect("key legacy passphrase-mode");
            init_schema(&conn).expect("init schema");
            conn.execute(
                "INSERT INTO hosts (id, label, address, port, username, auth_method, tags, created_at, updated_at)
                 VALUES ('h1', 'Legacy', '10.0.0.1', 22, 'root', '{\"type\":\"password\"}', '[]', 1, 1)",
                [],
            )
            .expect("insert into legacy db");
        }

        let migrated = open_encrypted(&dir.0, &hex_key).expect("open_encrypted migrates legacy db");
        let count: i64 = migrated
            .query_row("SELECT count(*) FROM hosts", [], |r| r.get(0))
            .expect("query hosts after migration");
        assert_eq!(
            count, 1,
            "data survives the passphrase -> raw-key migration"
        );
        drop(migrated);

        // Prove it: key with *only* the raw-key PRAGMA, bypassing open_encrypted's fallback
        // entirely. This only succeeds if the file on disk is now actually raw-key.
        let conn = Connection::open(&db_path).expect("reopen raw");
        conn.execute_batch(&format!("PRAGMA key = \"x'{hex_key}'\";"))
            .expect("key raw-key-mode");
        let count: i64 = conn
            .query_row("SELECT count(*) FROM hosts", [], |r| r.get(0))
            .expect("the file is genuinely raw-key now, not still passphrase-mode");
        assert_eq!(count, 1);
    }

    #[test]
    fn correct_passphrase_reopens_and_keeps_schema() {
        let dir = TempDataDir::new("correct_pass");
        {
            let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buat db");
            conn.execute(
                "INSERT INTO hosts (id, label, address, port, username, auth_method, tags, created_at, updated_at)
                 VALUES ('h1', 'Test', '10.0.0.1', 22, 'root', '{\"type\":\"password\"}', '[]', 1, 1)",
                [],
            )
            .expect("gagal insert");
        }
        let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buka ulang db");
        let count: i64 = conn
            .query_row("SELECT count(*) FROM hosts", [], |r| r.get(0))
            .expect("gagal query hosts");
        assert_eq!(count, 1);
    }

    #[test]
    fn plaintext_file_on_disk_never_contains_the_sqlite_header() {
        // SQLCipher-encrypted files never start with the standard "SQLite format 3\0" magic —
        // that header itself would be a plaintext leak. This is the actual "strong encryption"
        // assertion for REQ-9a: the bytes on disk must not be a readable SQLite file.
        let dir = TempDataDir::new("header_check");
        let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buat db");
        drop(conn);
        let bytes = std::fs::read(crate::paths::db_path(&dir.0)).expect("gagal baca file db");
        assert_ne!(&bytes[..16.min(bytes.len())], b"SQLite format 3\0");
    }
}
