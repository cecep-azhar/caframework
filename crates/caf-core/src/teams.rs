//! Teams store.
//! Backed by the same SQLCipher-encrypted `caframework.db` as the host and group store.

use crate::db;
use crate::error::{CatermError, DbError, ValidationError};
use crate::groups;
use crate::store;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRecord {
    pub id: String,
    pub name: String,
    pub color: String,
    pub avatar: Option<String>,
    pub members: Vec<String>,
    pub host_ids: Vec<String>,
    pub group_ids: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInput {
    pub id: Option<String>,
    pub name: String,
    pub color: String,
    pub avatar: Option<String>,
    pub members: Vec<String>,
    pub host_ids: Vec<String>,
    pub group_ids: Vec<String>,
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
    format!("team-{nanos:x}")
}

fn row_to_team(row: &rusqlite::Row) -> rusqlite::Result<TeamRecord> {
    let members_json: String = row.get("members")?;
    let members: Vec<String> = serde_json::from_str(&members_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let host_ids_json: String = row.get("host_ids")?;
    let host_ids: Vec<String> = serde_json::from_str(&host_ids_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let group_ids_json: String = row.get("group_ids")?;
    let group_ids: Vec<String> = serde_json::from_str(&group_ids_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(TeamRecord {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        avatar: row.get("avatar")?,
        members,
        host_ids,
        group_ids,
        created_at: row.get::<_, i64>("created_at")? as u64,
        updated_at: row.get::<_, i64>("updated_at")? as u64,
    })
}

pub(crate) fn list_teams_in(conn: &Connection) -> Result<Vec<TeamRecord>, CatermError> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, avatar, members, host_ids, group_ids, created_at, updated_at FROM teams ORDER BY created_at ASC")
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query teams: {e}"))))?;
    let rows = stmt
        .query_map([], row_to_team)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query teams: {e}"))))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(
            row.map_err(|e| CatermError::Db(DbError::Generic(format!("baris teams rusak: {e}"))))?,
        );
    }
    Ok(out)
}

pub(crate) fn save_team_in(conn: &Connection, input: TeamInput) -> Result<TeamRecord, CatermError> {
    let known_host_ids: HashSet<String> = store::list_hosts_in(conn)?
        .into_iter()
        .map(|h| h.id)
        .collect();
    for host_id in &input.host_ids {
        if !known_host_ids.contains(host_id) {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "host with id {host_id} not found"
            ))));
        }
    }

    let known_group_ids: HashSet<String> = groups::list_groups_in(conn)?
        .into_iter()
        .map(|g| g.id)
        .collect();
    for group_id in &input.group_ids {
        if !known_group_ids.contains(group_id) {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "group with id {group_id} not found"
            ))));
        }
    }

    let now = now_unix();
    let id = input.id.filter(|id| !id.is_empty());
    let created_at = match &id {
        Some(existing_id) => conn
            .query_row(
                "SELECT created_at FROM teams WHERE id = ?1",
                params![existing_id],
                |r| r.get::<_, i64>(0),
            )
            .map(|v| v as u64)
            .unwrap_or(now),
        None => now,
    };
    let id = id.unwrap_or_else(generate_id);

    let members_json = serde_json::to_string(&input.members).map_err(|e| {
        CatermError::Db(DbError::Generic(format!("gagal serialisasi members: {e}")))
    })?;
    let host_ids_json = serde_json::to_string(&input.host_ids).map_err(|e| {
        CatermError::Db(DbError::Generic(format!("gagal serialisasi host_ids: {e}")))
    })?;
    let group_ids_json = serde_json::to_string(&input.group_ids).map_err(|e| {
        CatermError::Db(DbError::Generic(format!(
            "gagal serialisasi group_ids: {e}"
        )))
    })?;

    conn.execute(
        "INSERT INTO teams (id, name, color, avatar, members, host_ids, group_ids, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            color = excluded.color,
            avatar = excluded.avatar,
            members = excluded.members,
            host_ids = excluded.host_ids,
            group_ids = excluded.group_ids,
            updated_at = excluded.updated_at",
        params![id, input.name, input.color, input.avatar, members_json, host_ids_json, group_ids_json, created_at as i64, now as i64],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal menyimpan team: {e}"))))?;

    Ok(TeamRecord {
        id,
        name: input.name,
        color: input.color,
        avatar: input.avatar,
        members: input.members,
        host_ids: input.host_ids,
        group_ids: input.group_ids,
        created_at,
        updated_at: now,
    })
}

fn delete_team_in(conn: &Connection, id: &str) -> Result<(), CatermError> {
    let affected = conn
        .execute("DELETE FROM teams WHERE id = ?1", params![id])
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal hapus team: {e}"))))?;
    if affected == 0 {
        return Err(CatermError::Db(DbError::Generic(format!(
            "team with id {id} not found"
        ))));
    }
    Ok(())
}

pub(crate) fn strip_host_from_teams(conn: &Connection, host_id: &str) -> Result<(), CatermError> {
    for mut team in list_teams_in(conn)? {
        if !team.host_ids.iter().any(|id| id == host_id) {
            continue;
        }
        team.host_ids.retain(|id| id != host_id);
        let host_ids_json = serde_json::to_string(&team.host_ids).map_err(|e| {
            CatermError::Db(DbError::Generic(format!("gagal serialisasi host_ids: {e}")))
        })?;
        conn.execute(
            "UPDATE teams SET host_ids = ?1, updated_at = ?2 WHERE id = ?3",
            params![host_ids_json, now_unix() as i64, team.id],
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal update team: {e}"))))?;
    }
    Ok(())
}

pub(crate) fn strip_group_from_teams(conn: &Connection, group_id: &str) -> Result<(), CatermError> {
    for mut team in list_teams_in(conn)? {
        if !team.group_ids.iter().any(|id| id == group_id) {
            continue;
        }
        team.group_ids.retain(|id| id != group_id);
        let group_ids_json = serde_json::to_string(&team.group_ids).map_err(|e| {
            CatermError::Db(DbError::Generic(format!(
                "gagal serialisasi group_ids: {e}"
            )))
        })?;
        conn.execute(
            "UPDATE teams SET group_ids = ?1, updated_at = ?2 WHERE id = ?3",
            params![group_ids_json, now_unix() as i64, team.id],
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal update team: {e}"))))?;
    }
    Ok(())
}

pub fn list_teams() -> Result<Vec<TeamRecord>, CatermError> {
    list_teams_in(&db::open()?)
}

pub fn save_team(input: TeamInput) -> Result<TeamRecord, CatermError> {
    save_team_in(&db::open()?, input)
}

pub fn delete_team(id: &str) -> Result<(), CatermError> {
    delete_team_in(&db::open()?, id)
}
