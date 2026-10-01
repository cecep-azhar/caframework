use crate::error::{CatermError, DbError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvestigationRecord {
    pub id: String,
    pub title: String,
    pub host_id: Option<String>,
    pub status: String,
    pub notes: String,
    pub evidence: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvestigationInput {
    pub id: Option<String>,
    pub title: String,
    pub host_id: Option<String>,
    pub status: String,
    pub notes: String,
    pub evidence: String,
}

pub fn list_investigations() -> Result<Vec<InvestigationRecord>, CatermError> {
    let conn = crate::db::open()?;
    let mut stmt = conn
        .prepare("SELECT id, title, host_id, status, notes, evidence, created_at, updated_at FROM investigations ORDER BY created_at DESC")
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to prepare query: {e}"))))?;

    let records = stmt
        .query_map([], |row| {
            Ok(InvestigationRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                host_id: row.get(2)?,
                status: row.get(3)?,
                notes: row.get(4)?,
                evidence: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| {
            CatermError::Db(DbError::Generic(format!(
                "Failed to query investigations: {e}"
            )))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Row read error: {e}"))))?;

    Ok(records)
}

pub fn save_investigation(input: InvestigationInput) -> Result<InvestigationRecord, CatermError> {
    let conn = crate::db::open()?;
    let now = chrono::Utc::now().timestamp_millis();
    let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let existing: Option<i64> = conn
        .query_row(
            "SELECT created_at FROM investigations WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let created_at = existing.unwrap_or(now);

    conn.execute(
        "INSERT INTO investigations (id, title, host_id, status, notes, evidence, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET 
            title = excluded.title,
            host_id = excluded.host_id,
            status = excluded.status,
            notes = excluded.notes,
            evidence = excluded.evidence,
            updated_at = excluded.updated_at",
        rusqlite::params![id, input.title, input.host_id, input.status, input.notes, input.evidence, created_at, now],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to save investigation: {e}"))))?;

    Ok(InvestigationRecord {
        id,
        title: input.title,
        host_id: input.host_id,
        status: input.status,
        notes: input.notes,
        evidence: input.evidence,
        created_at,
        updated_at: now,
    })
}

pub fn delete_investigation(id: &str) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    conn.execute(
        "DELETE FROM investigations WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| {
        CatermError::Db(DbError::Generic(format!(
            "Failed to delete investigation: {e}"
        )))
    })?;
    Ok(())
}

trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}
impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
