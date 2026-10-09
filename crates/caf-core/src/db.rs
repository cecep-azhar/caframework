//! Encrypted SQLite storage engine for CAFramework. Every record persisted lives inside a
//! single SQLCipher-encrypted file at `<data_dir>/caframework.db` — see [`crate::vault`].

use crate::error::{CafError, DbError};
use rusqlite::Connection;
use std::path::Path;

/// Opens (creating if absent) the SQLCipher-encrypted database at `<data_dir>/caframework.db`,
/// keys it with `passphrase`, and ensures the schema exists.
pub fn open_encrypted(data_dir: &Path, passphrase: &str) -> Result<Connection, CafError> {
    std::fs::create_dir_all(data_dir).map_err(|e| {
        CafError::Db(DbError::Generic(format!("failed to create data dir: {e}")))
    })?;
    let db_path = crate::paths::db_path(data_dir);

    let conn = if let Some(raw_key) = raw_key_literal(passphrase) {
        match open_keyed(&db_path, &raw_key) {
            Ok(conn) => conn,
            Err(_) => {
                let conn = open_keyed(&db_path, &passphrase_literal(passphrase))?;
                conn.execute_batch(&format!("PRAGMA rekey = {raw_key};"))
                    .map_err(|e| {
                        CafError::Db(DbError::Generic(format!(
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
    migrate_ai_routing_and_memory(&conn)?;
    Ok(conn)
}

fn migrate_ai_routing_and_memory(conn: &Connection) -> Result<(), CafError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ai_providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            base_url TEXT NOT NULL,
            api_key_encrypted TEXT,
            default_model TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            custom_headers_json TEXT DEFAULT '{}',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS ai_user_personas (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            system_prompt TEXT NOT NULL,
            custom_rules_json TEXT NOT NULL DEFAULT '[]',
            environment_constraints TEXT,
            is_global_default INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS ai_routing_matrix (
            task_type TEXT PRIMARY KEY,
            primary_provider_id TEXT NOT NULL,
            primary_model TEXT NOT NULL,
            fallback_provider_id TEXT,
            fallback_model TEXT,
            temperature REAL NOT NULL DEFAULT 0.2,
            max_tokens INTEGER NOT NULL DEFAULT 2048,
            system_persona_id TEXT,
            FOREIGN KEY(primary_provider_id) REFERENCES ai_providers(id) ON DELETE RESTRICT,
            FOREIGN KEY(fallback_provider_id) REFERENCES ai_providers(id) ON DELETE SET NULL,
            FOREIGN KEY(system_persona_id) REFERENCES ai_user_personas(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS ai_habits (
            id TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            key_tag TEXT NOT NULL,
            fact_content TEXT NOT NULL,
            source_context TEXT,
            confidence_score REAL NOT NULL DEFAULT 1.0,
            occurrence_count INTEGER NOT NULL DEFAULT 1,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            last_accessed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE TABLE IF NOT EXISTS ai_custom_skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            category TEXT NOT NULL,
            triggers_json TEXT NOT NULL,
            preferred_model_id TEXT,
            system_instructions TEXT NOT NULL,
            allowed_tools_json TEXT NOT NULL,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS ai_habits_fts USING fts5(
            id UNINDEXED,
            category,
            key_tag,
            fact_content,
            tokenize = 'porter unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS trg_ai_habits_ai AFTER INSERT ON ai_habits BEGIN
            INSERT INTO ai_habits_fts(id, category, key_tag, fact_content)
            VALUES (new.id, new.category, new.key_tag, new.fact_content);
        END;

        CREATE TRIGGER IF NOT EXISTS trg_ai_habits_ad AFTER DELETE ON ai_habits BEGIN
            INSERT INTO ai_habits_fts(ai_habits_fts, id, category, key_tag, fact_content)
            VALUES('delete', old.id, old.category, old.key_tag, old.fact_content);
        END;

        CREATE TRIGGER IF NOT EXISTS trg_ai_habits_au AFTER UPDATE ON ai_habits BEGIN
            INSERT INTO ai_habits_fts(ai_habits_fts, id, category, key_tag, fact_content)
            VALUES('delete', old.id, old.category, old.key_tag, old.fact_content);
            INSERT INTO ai_habits_fts(id, category, key_tag, fact_content)
            VALUES (new.id, new.category, new.key_tag, new.fact_content);
        END;",
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("gagal migrasi ai tables: {e}"))))?;

    // Seed default providers if none exist
    let providers_count: i64 = conn
        .query_row("SELECT count(*) FROM ai_providers", [], |r| r.get(0))
        .unwrap_or(0);
    if providers_count == 0 {
        conn.execute_batch(
            "INSERT OR IGNORE INTO ai_providers (id, name, provider_type, base_url, default_model, is_active) VALUES
             ('byo-anthropic', 'Anthropic Claude', 'anthropic', 'https://api.anthropic.com/v1', 'claude-3-7-sonnet-20250219', 1),
             ('byo-openai', 'OpenAI', 'openai_compatible', 'https://api.openai.com/v1', 'gpt-4o', 1),
             ('local-ollama', 'Ollama Local', 'ollama', 'http://localhost:11434', 'llama3.2:latest', 1);"
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("gagal seed ai providers: {e}"))))?;
    }

    // Seed default persona if none exist
    let persona_count: i64 = conn
        .query_row("SELECT count(*) FROM ai_user_personas", [], |r| r.get(0))
        .unwrap_or(0);
    if persona_count == 0 {
        conn.execute(
            "INSERT OR IGNORE INTO ai_user_personas (id, title, description, system_prompt, custom_rules_json, environment_constraints, is_global_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            rusqlite::params![
                "default-general",
                "CAFramework AI Assistant",
                "Asisten AI cerdas untuk aplikasi ekosistem CA",
                "You are an AI assistant in CAFramework. Be concise, objective, and helpful.",
                "[]",
                "Linux/macOS/Windows, POSIX shell"
            ],
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("gagal seed persona: {e}"))))?;
    }

    // Seed default routing matrix if empty
    let matrix_count: i64 = conn
        .query_row("SELECT count(*) FROM ai_routing_matrix", [], |r| r.get(0))
        .unwrap_or(0);
    if matrix_count == 0 {
        conn.execute_batch(
            "INSERT OR IGNORE INTO ai_routing_matrix (task_type, primary_provider_id, primary_model, fallback_provider_id, fallback_model, temperature, max_tokens, system_persona_id) VALUES
             ('chat', 'byo-anthropic', 'claude-3-7-sonnet-20250219', 'byo-openai', 'gpt-4o', 0.7, 4096, 'default-general'),
             ('error_diagnostic', 'byo-anthropic', 'claude-3-7-sonnet-20250219', 'byo-openai', 'gpt-4o-mini', 0.2, 2048, 'default-general'),
             ('prompt_studio', 'byo-anthropic', 'claude-3-7-sonnet-20250219', 'byo-openai', 'gpt-4o', 0.7, 4096, 'default-general'),
             ('command_autocomplete', 'byo-openai', 'gpt-4o-mini', 'local-ollama', 'llama3.2:latest', 0.1, 512, 'default-general'),
             ('security_review', 'byo-anthropic', 'claude-3-7-sonnet-20250219', 'byo-openai', 'gpt-4o', 0.1, 4096, 'default-general');"
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("gagal seed routing matrix: {e}"))))?;
    }

    // Seed default builtin skills if empty
    let skills_count: i64 = conn
        .query_row("SELECT count(*) FROM ai_custom_skills", [], |r| r.get(0))
        .unwrap_or(0);
    if skills_count == 0 {
        for s in crate::ai::skills::builtin_skills() {
            let _ = crate::ai::db::save_skill(conn, &s);
        }
    }

    Ok(())
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

fn open_keyed(db_path: &Path, key_literal: &str) -> Result<Connection, CafError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CafError::Db(DbError::Generic(format!("failed to open database: {e}"))))?;

    conn.execute_batch(&format!("PRAGMA key = {key_literal};"))
        .map_err(|e| CafError::Db(DbError::Generic(format!("failed to set vault key: {e}"))))?;

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| {
            CafError::Db(DbError::Generic(
                "invalid vault key or corrupted database".into(),
            ))
        })?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;\nPRAGMA foreign_keys = ON;",
    )
    .map_err(|e| {
        CafError::Db(DbError::Generic(format!(
            "failed to configure database pragmas: {e}"
        )))
    })?;

    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<(), CafError> {
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
        CafError::Db(DbError::Generic(format!(
            "failed to initialize schema: {e}"
        )))
    })?;

    Ok(())
}

/// Convenience helper to open the encrypted DB with the current vault key.
pub fn open() -> Result<Connection, CafError> {
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
