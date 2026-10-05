//! Secure SQLCipher storage for billing tokens and device state.

use crate::db;
use crate::error::{CatermError, DbError};

const KEY_DEVICE_TOKEN: &str = "billing.device_token";
const KEY_DEVICE_ID: &str = "billing.device_id";
const KEY_CACHED_TOKEN: &str = "billing.cached_token";
const KEY_LAST_VERIFIED_AT: &str = "billing.last_verified_at";
const KEY_MAX_SEEN_TIME: &str = "billing.max_seen_time";

pub fn save_device_info(device_id: &str, device_token: &str) -> Result<(), CatermError> {
    let conn = db::open()?;
    conn.execute(
        "INSERT OR REPLACE INTO app_kv (key, value) VALUES (?1, ?2), (?3, ?4)",
        [KEY_DEVICE_ID, device_id, KEY_DEVICE_TOKEN, device_token],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    Ok(())
}

pub fn get_device_id() -> Result<Option<String>, CatermError> {
    get_kv(KEY_DEVICE_ID)
}

pub fn get_device_token() -> Result<Option<String>, CatermError> {
    get_kv(KEY_DEVICE_TOKEN)
}

pub fn save_cached_token(token: &str, verified_at_unix: u64) -> Result<(), CatermError> {
    let conn = db::open()?;
    let time_str = verified_at_unix.to_string();

    // Update max seen time as well
    let current_max = get_max_seen_time()?.unwrap_or(0);
    let new_max = current_max.max(verified_at_unix).to_string();

    conn.execute(
        "INSERT OR REPLACE INTO app_kv (key, value) VALUES (?1, ?2), (?3, ?4), (?5, ?6)",
        [
            KEY_CACHED_TOKEN,
            token,
            KEY_LAST_VERIFIED_AT,
            &time_str,
            KEY_MAX_SEEN_TIME,
            &new_max,
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    Ok(())
}

pub fn get_cached_token() -> Result<Option<String>, CatermError> {
    get_kv(KEY_CACHED_TOKEN)
}

pub fn get_last_verified_at() -> Result<Option<u64>, CatermError> {
    if let Some(s) = get_kv(KEY_LAST_VERIFIED_AT)? {
        Ok(s.parse::<u64>().ok())
    } else {
        Ok(None)
    }
}

pub fn get_max_seen_time() -> Result<Option<u64>, CatermError> {
    if let Some(s) = get_kv(KEY_MAX_SEEN_TIME)? {
        Ok(s.parse::<u64>().ok())
    } else {
        Ok(None)
    }
}

pub fn update_max_seen_time(now_unix: u64) -> Result<(), CatermError> {
    let current_max = get_max_seen_time()?.unwrap_or(0);
    if now_unix > current_max {
        let conn = db::open()?;
        conn.execute(
            "INSERT OR REPLACE INTO app_kv (key, value) VALUES (?1, ?2)",
            [KEY_MAX_SEEN_TIME, &now_unix.to_string()],
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    }
    Ok(())
}

pub fn clear_billing_state() -> Result<(), CatermError> {
    let conn = db::open()?;
    conn.execute(
        "DELETE FROM app_kv WHERE key IN (?1, ?2, ?3, ?4)",
        [
            KEY_DEVICE_TOKEN,
            KEY_CACHED_TOKEN,
            KEY_LAST_VERIFIED_AT,
            KEY_MAX_SEEN_TIME,
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    Ok(())
}

fn get_kv(key: &str) -> Result<Option<String>, CatermError> {
    let conn = db::open()?;
    let mut stmt = conn
        .prepare("SELECT value FROM app_kv WHERE key = ?1")
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    let mut rows = stmt
        .query([key])
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?
    {
        Ok(Some(row.get(0).map_err(|e| {
            CatermError::Db(DbError::Generic(e.to_string()))
        })?))
    } else {
        Ok(None)
    }
}
