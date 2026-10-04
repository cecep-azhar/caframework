use chrono::Utc;
use rusqlite::Connection;

pub fn write_change_log(
    conn: &Connection,
    entity: &str,
    entity_id: &str,
    action: &str,
) -> Result<(), rusqlite::Error> {
    let now = Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO sync_changelog (entity_type, entity_id, action, timestamp) VALUES (?1, ?2, ?3, ?4)",
        (entity, entity_id, action, now),
    )?;
    Ok(())
}

pub fn create_changelog_triggers(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sync_changelog (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            action TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            synced INTEGER DEFAULT 0
        );
    ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sync_changelog() {
        let conn = Connection::open_in_memory().unwrap();
        create_changelog_triggers(&conn).unwrap();
        write_change_log(&conn, "note", "n_123", "INSERT").unwrap();
        let ct: i64 = conn
            .query_row("SELECT COUNT(*) FROM sync_changelog", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ct, 1);
    }
}
