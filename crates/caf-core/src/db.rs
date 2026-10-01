//! Encrypted SQLite storage engine for CAFramework. Every record persisted lives inside a
//! single SQLCipher-encrypted file at `<data_dir>/caframework.db` — see [`crate::vault`].

use crate::error::{CatermError, DbError};
use rusqlite::Connection;
use std::path::Path;

/// Opens (creating if absent) the SQLCipher-encrypted database at `<data_dir>/caframework.db`,
/// keys it with `passphrase`, and ensures the schema exists.
pub fn open_encrypted(data_dir: &Path, passphrase: &str) -> Result<Connection, CatermError> {
    std::fs::create_dir_all(data_dir).map_err(|e| {
        CatermError::Db(DbError::Generic(format!("failed to create data dir: {e}")))
    })?;
    let db_path = crate::paths::db_path(data_dir);

    let conn = if let Some(raw_key) = raw_key_literal(passphrase) {
        match open_keyed(&db_path, &raw_key) {
            Ok(conn) => conn,
            Err(_) => {
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
    Ok(conn)
}

fn passphrase_literal(passphrase: &str) -> String {
    format!("'{}'", passphrase.replace('\'', "''"))
}

fn raw_key_literal(passphrase: &str) -> Option<String> {
    if passphrase.len() == 64 && passphrase.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(format!("\"x'{passphrase}'\""))
    } else {
        None
    }
}

fn open_keyed(db_path: &Path, key_literal: &str) -> Result<Connection, CatermError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("failed to open database: {e}"))))?;

    conn.execute_batch(&format!(
        "PRAGMA key = {key_literal};\nPRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;\nPRAGMA foreign_keys = ON;"
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

pub fn init_schema(conn: &Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
           version INTEGER PRIMARY KEY,
           applied_at TEXT NOT NULL DEFAULT (datetime('now'))
         );

         CREATE TABLE IF NOT EXISTS app_kv (
           key TEXT PRIMARY KEY,
           value TEXT NOT NULL,
           updated_at TEXT NOT NULL DEFAULT (datetime('now'))
         );

         CREATE TABLE IF NOT EXISTS profiles (
           id TEXT PRIMARY KEY,
           name TEXT NOT NULL,
           role TEXT NOT NULL DEFAULT 'owner',
           avatar TEXT,
           pin_hash TEXT,
           rev INTEGER NOT NULL DEFAULT 1,
           created_at TEXT NOT NULL DEFAULT (datetime('now')),
           updated_at TEXT NOT NULL DEFAULT (datetime('now')),
           deleted_at TEXT,
           origin_device_id TEXT NOT NULL,
           owner_profile_id TEXT NOT NULL,
           visibility TEXT NOT NULL DEFAULT 'shared'
         );

         CREATE TABLE IF NOT EXISTS change_log (
           seq INTEGER PRIMARY KEY AUTOINCREMENT,
           entity_type TEXT NOT NULL,
           entity_id TEXT NOT NULL,
           rev INTEGER NOT NULL,
           action TEXT NOT NULL, -- 'create' | 'update' | 'delete'
           payload TEXT,
           synced_at TEXT,
           timestamp TEXT NOT NULL DEFAULT (datetime('now')),
           origin_device_id TEXT NOT NULL,
           owner_profile_id TEXT NOT NULL,
           visibility TEXT NOT NULL DEFAULT 'shared'
         );

         CREATE TABLE IF NOT EXISTS notes (
           id TEXT PRIMARY KEY,
           title TEXT NOT NULL,
           content TEXT NOT NULL DEFAULT '',
           tags TEXT NOT NULL DEFAULT '[]',
           rev INTEGER NOT NULL DEFAULT 1,
           created_at TEXT NOT NULL DEFAULT (datetime('now')),
           updated_at TEXT NOT NULL DEFAULT (datetime('now')),
           deleted_at TEXT,
           origin_device_id TEXT NOT NULL,
           owner_profile_id TEXT NOT NULL,
           visibility TEXT NOT NULL DEFAULT 'shared'
         );",
    )
    .map_err(|e| {
        CatermError::Db(DbError::Generic(format!(
            "failed to initialize schema: {e}"
        )))
    })?;

    Ok(())
}

/// Convenience helper to open the encrypted DB with the current vault key.
pub fn open() -> Result<Connection, CatermError> {
    let data_dir = crate::paths::data_dir()?;
    let key = crate::vault::ensure_unlocked_key()?;
    open_encrypted(&data_dir, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_encrypted_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("caf_db_test_{}", uuid::Uuid::new_v4()));
        let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let conn = open_encrypted(&temp_dir, key).expect("open encrypted db");

        conn.execute(
            "INSERT INTO app_kv (key, value) VALUES (?1, ?2)",
            ["test_k", "test_v"],
        )
        .unwrap();
        let val: String = conn
            .query_row("SELECT value FROM app_kv WHERE key = ?1", ["test_k"], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(val, "test_v");

        drop(conn);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
