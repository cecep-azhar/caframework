//! Snippet store. Reusable shell commands a user can inject into an active SSH session.
//! Backed by the same SQLCipher-encrypted `caframework.db` as hosts/groups (see `crate::db`) —
//! no group/host linking here, snippets are standalone.

use crate::db;
use crate::error::{CatermError, DbError};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetRecord {
    pub id: String,
    pub label: String,
    pub description: String,
    pub command: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Payload for `save_snippet`: same shape as `SnippetRecord` minus the fields the store
/// itself owns. `id` omitted/empty means "create new", present means "update that record".
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetInput {
    pub id: Option<String>,
    pub label: String,
    pub description: String,
    pub command: String,
    pub tags: Vec<String>,
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("snippet-{nanos:x}")
}

fn row_to_snippet(row: &rusqlite::Row) -> rusqlite::Result<SnippetRecord> {
    let tags_json: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(SnippetRecord {
        id: row.get("id")?,
        label: row.get("label")?,
        description: row.get("description")?,
        command: row.get("command")?,
        tags,
        created_at: row.get::<_, i64>("created_at")? as u64,
        updated_at: row.get::<_, i64>("updated_at")? as u64,
    })
}

pub(crate) fn list_snippets_in(conn: &Connection) -> Result<Vec<SnippetRecord>, CatermError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, label, description, command, tags, created_at, updated_at \
             FROM snippets ORDER BY created_at ASC",
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query snippets: {e}"))))?;
    let rows = stmt
        .query_map([], row_to_snippet)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query snippets: {e}"))))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| {
            CatermError::Db(DbError::Generic(format!("baris snippets rusak: {e}")))
        })?);
    }
    Ok(out)
}

pub(crate) fn save_snippet_in(
    conn: &Connection,
    input: SnippetInput,
) -> Result<SnippetRecord, CatermError> {
    let now = now_unix();
    let id = input.id.filter(|id| !id.is_empty());
    let created_at = match &id {
        Some(existing_id) => conn
            .query_row(
                "SELECT created_at FROM snippets WHERE id = ?1",
                params![existing_id],
                |r| r.get::<_, i64>(0),
            )
            .map(|v| v as u64)
            .unwrap_or(now),
        None => now,
    };
    let id = id.unwrap_or_else(generate_id);

    let tags_json = serde_json::to_string(&input.tags)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal serialisasi tags: {e}"))))?;

    conn.execute(
        "INSERT INTO snippets (id, label, description, command, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            label = excluded.label,
            description = excluded.description,
            command = excluded.command,
            tags = excluded.tags,
            updated_at = excluded.updated_at",
        params![
            id,
            input.label,
            input.description,
            input.command,
            tags_json,
            created_at as i64,
            now as i64
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal menyimpan snippet: {e}"))))?;

    Ok(SnippetRecord {
        id,
        label: input.label,
        description: input.description,
        command: input.command,
        tags: input.tags,
        created_at,
        updated_at: now,
    })
}

fn delete_snippet_in(conn: &Connection, id: &str) -> Result<(), CatermError> {
    let affected = conn
        .execute("DELETE FROM snippets WHERE id = ?1", params![id])
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal hapus snippet: {e}"))))?;
    if affected == 0 {
        return Err(CatermError::Db(DbError::Generic(format!(
            "snippet with id {id} not found"
        ))));
    }
    Ok(())
}

/// All saved snippets, ordered by creation time.
pub fn list_snippets() -> Result<Vec<SnippetRecord>, CatermError> {
    list_snippets_in(&db::open()?)
}

/// Insert a new snippet (empty/absent `id`) or update an existing one in place,
/// preserving its original `created_at`.
pub fn save_snippet(input: SnippetInput) -> Result<SnippetRecord, CatermError> {
    save_snippet_in(&db::open()?, input)
}

/// Remove a snippet by id. Errors if no snippet with that id exists.
pub fn delete_snippet(id: &str) -> Result<(), CatermError> {
    delete_snippet_in(&db::open()?, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDb(Connection, PathBuf);

    impl TempDb {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("caframework_snippets_test_{label}_{nanos:x}"));
            let conn = db::open_encrypted(&dir, "test-passphrase").expect("gagal buka db test");
            Self(conn, dir)
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.1);
        }
    }

    #[test]
    fn save_then_list_roundtrips() {
        let db = TempDb::new("roundtrip");
        let saved = save_snippet_in(
            &db.0,
            SnippetInput {
                id: None,
                label: "Docker Clean".into(),
                description: "Prune everything".into(),
                command: "docker system prune -a --volumes -f".into(),
                tags: vec!["docker".into()],
            },
        )
        .expect("save_snippet gagal");

        let all = list_snippets_in(&db.0).expect("list_snippets gagal");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);
        assert_eq!(all[0].label, "Docker Clean");
    }

    #[test]
    fn save_with_existing_id_updates_in_place() {
        let db = TempDb::new("update");
        let first = save_snippet_in(
            &db.0,
            SnippetInput {
                id: None,
                label: "Original".into(),
                description: "".into(),
                command: "ls".into(),
                tags: vec![],
            },
        )
        .expect("save_snippet gagal");

        let updated = save_snippet_in(
            &db.0,
            SnippetInput {
                id: Some(first.id.clone()),
                label: "Updated".into(),
                description: "desc".into(),
                command: "ls -la".into(),
                tags: vec!["fs".into()],
            },
        )
        .expect("save_snippet gagal");

        assert_eq!(updated.id, first.id);
        assert_eq!(updated.created_at, first.created_at);

        let all = list_snippets_in(&db.0).expect("list_snippets gagal");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].label, "Updated");
    }

    #[test]
    fn delete_removes_snippet_and_errors_when_missing() {
        let db = TempDb::new("delete");
        let saved = save_snippet_in(
            &db.0,
            SnippetInput {
                id: None,
                label: "To Delete".into(),
                description: "".into(),
                command: "rm -rf /tmp/x".into(),
                tags: vec![],
            },
        )
        .expect("save_snippet gagal");

        delete_snippet_in(&db.0, &saved.id).expect("delete_snippet gagal");
        assert!(
            list_snippets_in(&db.0)
                .expect("list_snippets gagal")
                .is_empty()
        );
        assert!(delete_snippet_in(&db.0, &saved.id).is_err());
    }
}
