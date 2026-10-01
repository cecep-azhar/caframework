//! Groups store (T6). `GroupRecord::host_ids` is the Hosts<->Groups link — every id in it is
//! validated against `crate::store::list_hosts` at save time, and kept in sync when a host is
//! deleted (`strip_host_from_groups`, called from `store::delete_host`). Backed by the same
//! SQLCipher-encrypted `caframework.db` as the host store (see `crate::db`).

use crate::db;
use crate::error::{CatermError, DbError, ValidationError};
use crate::store;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRecord {
    pub id: String,
    pub name: String,
    pub color: String,
    pub host_ids: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Payload for `save_group`: same shape as `GroupRecord` minus the fields the store itself
/// owns. `id` omitted/empty means "create new", present means "update that record".
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInput {
    pub id: Option<String>,
    pub name: String,
    pub color: String,
    pub host_ids: Vec<String>,
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
    format!("group-{nanos:x}")
}

fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<GroupRecord> {
    let host_ids_json: String = row.get("host_ids")?;
    let host_ids: Vec<String> = serde_json::from_str(&host_ids_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(GroupRecord {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        host_ids,
        created_at: row.get::<_, i64>("created_at")? as u64,
        updated_at: row.get::<_, i64>("updated_at")? as u64,
    })
}

pub(crate) fn list_groups_in(conn: &Connection) -> Result<Vec<GroupRecord>, CatermError> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, host_ids, created_at, updated_at FROM groups ORDER BY created_at ASC")
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query groups: {e}"))))?;
    let rows = stmt
        .query_map([], row_to_group)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query groups: {e}"))))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(
            row.map_err(|e| CatermError::Db(DbError::Generic(format!("baris groups rusak: {e}"))))?,
        );
    }
    Ok(out)
}

pub(crate) fn save_group_in(
    conn: &Connection,
    input: GroupInput,
) -> Result<GroupRecord, CatermError> {
    let known_ids: HashSet<String> = store::list_hosts_in(conn)?
        .into_iter()
        .map(|h| h.id)
        .collect();
    for host_id in &input.host_ids {
        if !known_ids.contains(host_id) {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "host with id {host_id} not found"
            ))));
        }
    }

    let now = now_unix();
    let id = input.id.filter(|id| !id.is_empty());
    let created_at = match &id {
        Some(existing_id) => conn
            .query_row(
                "SELECT created_at FROM groups WHERE id = ?1",
                params![existing_id],
                |r| r.get::<_, i64>(0),
            )
            .map(|v| v as u64)
            .unwrap_or(now),
        None => now,
    };
    let id = id.unwrap_or_else(generate_id);

    let host_ids_json = serde_json::to_string(&input.host_ids).map_err(|e| {
        CatermError::Db(DbError::Generic(format!("gagal serialisasi host_ids: {e}")))
    })?;

    conn.execute(
        "INSERT INTO groups (id, name, color, host_ids, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            color = excluded.color,
            host_ids = excluded.host_ids,
            updated_at = excluded.updated_at",
        params![
            id,
            input.name,
            input.color,
            host_ids_json,
            created_at as i64,
            now as i64
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal menyimpan group: {e}"))))?;

    Ok(GroupRecord {
        id,
        name: input.name,
        color: input.color,
        host_ids: input.host_ids,
        created_at,
        updated_at: now,
    })
}

fn delete_group_in(conn: &Connection, id: &str) -> Result<(), CatermError> {
    let affected = conn
        .execute("DELETE FROM groups WHERE id = ?1", params![id])
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal hapus group: {e}"))))?;
    if affected == 0 {
        return Err(CatermError::Db(DbError::Generic(format!(
            "group with id {id} not found"
        ))));
    }
    crate::teams::strip_group_from_teams(conn, id)
}

/// Removes `host_id` from every group's `host_ids`, keeping the Hosts<->Groups link
/// consistent after a host is deleted. Called from `crate::store::delete_host`.
pub(crate) fn strip_host_from_groups(conn: &Connection, host_id: &str) -> Result<(), CatermError> {
    for mut group in list_groups_in(conn)? {
        if !group.host_ids.iter().any(|id| id == host_id) {
            continue;
        }
        group.host_ids.retain(|id| id != host_id);
        let host_ids_json = serde_json::to_string(&group.host_ids).map_err(|e| {
            CatermError::Db(DbError::Generic(format!("gagal serialisasi host_ids: {e}")))
        })?;
        conn.execute(
            "UPDATE groups SET host_ids = ?1, updated_at = ?2 WHERE id = ?3",
            params![host_ids_json, now_unix() as i64, group.id],
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal update group: {e}"))))?;
    }
    Ok(())
}

/// All saved groups, ordered by creation time.
pub fn list_groups() -> Result<Vec<GroupRecord>, CatermError> {
    list_groups_in(&db::open()?)
}

/// Insert a new group (empty/absent `id`) or update an existing one in place, preserving its
/// original `created_at`. Every `host_ids` entry must reference an existing host.
pub fn save_group(input: GroupInput) -> Result<GroupRecord, CatermError> {
    save_group_in(&db::open()?, input)
}

/// Remove a group by id. Errors if no group with that id exists.
pub fn delete_group(id: &str) -> Result<(), CatermError> {
    delete_group_in(&db::open()?, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{AuthMethod, HostInput};
    use std::path::PathBuf;

    struct TempDb(Connection, PathBuf);

    impl TempDb {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("caframework_groups_test_{label}_{nanos:x}"));
            let conn = db::open_encrypted(&dir, "test-passphrase").expect("gagal buka db test");
            Self(conn, dir)
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.1);
        }
    }

    fn make_host(conn: &Connection, label: &str) -> String {
        store::save_host_in(
            conn,
            HostInput {
                id: None,
                label: label.into(),
                address: "10.0.0.1".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec![],
                os: None,
                protocol: None,
                secret: None,
            },
            "test-local-key",
        )
        .expect("gagal buat host")
        .id
    }

    #[test]
    fn save_group_rejects_unknown_host_id() {
        let db = TempDb::new("unknown_host");
        let result = save_group_in(
            &db.0,
            GroupInput {
                id: None,
                name: "Prod".into(),
                color: "#ef4444".into(),
                host_ids: vec!["host-does-not-exist".into()],
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn save_then_list_roundtrips_with_linked_hosts() {
        let db = TempDb::new("roundtrip");
        let host_id = make_host(&db.0, "Web 1");

        let saved = save_group_in(
            &db.0,
            GroupInput {
                id: None,
                name: "Web Tier".into(),
                color: "#3b82f6".into(),
                host_ids: vec![host_id.clone()],
            },
        )
        .expect("save_group gagal");

        let all = list_groups_in(&db.0).expect("list_groups gagal");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);
        assert_eq!(all[0].host_ids, vec![host_id]);
    }

    #[test]
    fn delete_removes_group_and_errors_when_missing() {
        let db = TempDb::new("delete");
        let saved = save_group_in(
            &db.0,
            GroupInput {
                id: None,
                name: "Temp".into(),
                color: "#10b981".into(),
                host_ids: vec![],
            },
        )
        .expect("save_group gagal");

        delete_group_in(&db.0, &saved.id).expect("delete_group gagal");
        assert!(list_groups_in(&db.0).expect("list_groups gagal").is_empty());
        assert!(delete_group_in(&db.0, &saved.id).is_err());
    }
}
