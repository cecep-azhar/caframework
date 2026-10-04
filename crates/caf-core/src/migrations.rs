use crate::error::{CatermError, DbError};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const MIGRATIONS: &[(i32, &str, &str)] = &[(
    1,
    "0001_baseline_schema",
    include_str!("migrations/0001_baseline_schema.sql"),
)];

fn compute_checksum(sql: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sql.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn run_migrations(conn: &mut Connection) -> Result<(), CatermError> {
    // Only bootstrap the schema version table if it doesn't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )
    .map_err(|e| {
        CatermError::Db(DbError::Generic(format!(
            "failed to bootstrap schema_version: {e}"
        )))
    })?;

    let tx = conn.transaction().map_err(|e| {
        CatermError::Db(DbError::Generic(format!(
            "failed to start migration transaction: {e}"
        )))
    })?;

    let max_version: i32 = tx
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut applied_any = false;

    for &(version, name, sql) in MIGRATIONS {
        let expected_checksum = compute_checksum(sql);

        if version <= max_version {
            // Verify checksum for already applied migrations
            let stored_checksum: Result<String, _> = tx.query_row(
                "SELECT checksum FROM schema_version WHERE version = ?1",
                [&version],
                |row| row.get(0),
            );

            if let Ok(stored) = stored_checksum {
                if stored != expected_checksum {
                    return Err(CatermError::Db(DbError::MigrationChecksum));
                }
            } else {
                return Err(CatermError::Db(DbError::Generic(format!(
                    "migration {} is marked applied but no checksum found",
                    version
                ))));
            }
            continue;
        }

        // Apply new migration
        tx.execute_batch(sql).map_err(|e| {
            CatermError::Db(DbError::Generic(format!(
                "failed to apply migration {}: {}",
                version, e
            )))
        })?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        tx.execute(
            "INSERT INTO schema_version (version, name, checksum, applied_at) VALUES (?1, ?2, ?3, ?4)",
            (&version, &name, &expected_checksum, &now_ms),
        ).map_err(|e| {
            CatermError::Db(DbError::Generic(format!("failed to record migration {}: {}", version, e)))
        })?;

        applied_any = true;
    }

    if max_version > MIGRATIONS.last().map(|m| m.0).unwrap_or(0) {
        return Err(CatermError::Db(DbError::SchemaTooNew));
    }

    if applied_any {
        tx.commit().map_err(|e| {
            CatermError::Db(DbError::Generic(format!(
                "failed to commit migrations: {e}"
            )))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_forward() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).expect("baseline migration failed");

        let cv: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cv, 1);
    }
}
