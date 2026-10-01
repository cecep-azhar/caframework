use crate::error::{CatermError, DbError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandLog {
    pub id: String,
    pub event_type: String,
    pub timestamp: i64,
    pub host_id: Option<String>,
    pub details: String,
}

static SECRET_REGEX: OnceLock<Regex> = OnceLock::new();

pub(crate) fn mask_secrets(text: &str) -> String {
    let re = match SECRET_REGEX.get() {
        Some(r) => r,
        None => {
            if let Ok(compiled) = Regex::new(
                r#"(?i)(password|pass|secret|token|key)\s*(=|:|\s)\s*['"]?([^'"\s]+)['"]?"#,
            ) {
                let _ = SECRET_REGEX.set(compiled);
            }
            if let Some(r) = SECRET_REGEX.get() {
                r
            } else {
                return text.to_string();
            }
        }
    };
    re.replace_all(text, "$1$2***").to_string()
}

pub fn log_event(
    event_type: &str,
    host_id: Option<&str>,
    details: &str,
) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let masked_details = mask_secrets(details);

    conn.execute(
        "INSERT INTO command_logs (id, event_type, timestamp, host_id, details) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, event_type, timestamp, host_id, masked_details],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to log event: {e}"))))?;

    Ok(())
}

pub fn get_logs(
    host_id_filter: Option<&str>,
    search: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<CommandLog>, CatermError> {
    let conn = crate::db::open()?;
    let mut query =
        "SELECT id, event_type, timestamp, host_id, details FROM command_logs WHERE 1=1"
            .to_string();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(hid) = host_id_filter.filter(|h| !h.is_empty()) {
        query.push_str(" AND host_id = ?");
        params.push(hid.to_string().into());
    }

    if let Some(s) = search.filter(|s| !s.is_empty()) {
        query.push_str(" AND (event_type LIKE ? OR details LIKE ?)");
        let like_str = format!("%{}%", s);
        params.push(like_str.clone().into());
        params.push(like_str.into());
    }

    query.push_str(" ORDER BY timestamp DESC");
    let max_limit = limit.unwrap_or(1000);
    query.push_str(&format!(" LIMIT {}", max_limit));

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to prepare query: {e}"))))?;

    let logs = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(CommandLog {
                id: row.get(0)?,
                event_type: row.get(1)?,
                timestamp: row.get(2)?,
                host_id: row.get(3)?,
                details: row.get(4)?,
            })
        })
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to query logs: {e}"))))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Row read error: {e}"))))?;

    Ok(logs)
}
