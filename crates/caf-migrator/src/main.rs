use chrono::Utc;
use clap::Parser;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(
    name = "caf-migrator",
    version,
    about = "Run DB migrations for caframework"
)]
struct Args {
    /// Path to the SQLite database file
    #[arg(short, long, default_value = "caframework.db")]
    db_path: PathBuf,
    /// Path to the migrations directory (defaults to ./migrations)
    #[arg(short, long, default_value = "migrations")]
    migrations_dir: PathBuf,
}

#[derive(Error, Debug)]
pub enum MigrateError {
    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("migration error: {0}")]
    Other(String),
}

fn main() -> Result<(), MigrateError> {
    let args = Args::parse();
    let conn = Connection::open(&args.db_path)?;
    // ensure schema_version exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (\n            version INTEGER PRIMARY KEY,\n            applied_at INTEGER NOT NULL\n        );",
    )?;
    let applied = run_migrations(&conn, &args.migrations_dir)?;
    println!("Applied {} migrations", applied);
    Ok(())
}

fn current_version(conn: &Connection) -> Result<i64, rusqlite::Error> {
    let v: Option<i64> = conn.query_row(
        "SELECT MAX(version) FROM schema_version",
        [],
        |row| row.get(0),
    )?;
    Ok(v.unwrap_or(0))
}

pub fn run_migrations(conn: &Connection, migrations_dir: &PathBuf) -> Result<usize, MigrateError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;
    let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();
    // sort by filename (assumes numeric prefix)
    entries.sort_by_key(|e| e.file_name());
    let cur = current_version(conn)?;
    let mut applied = 0usize;
    for entry in entries {
        let fname = entry
            .file_name()
            .into_string()
            .map_err(|_| MigrateError::Other("Invalid filename".into()))?;
        // extract leading number
        let num_part: String = fname.chars().take_while(|c| c.is_ascii_digit()).collect();
        let version: i64 = if num_part.is_empty() {
            continue;
        } else {
            num_part
                .parse()
                .map_err(|_| MigrateError::Other(format!("Invalid version in {}", fname)))?
        };
        if version <= cur {
            continue;
        }
        let sql = std::fs::read_to_string(entry.path())?;
        conn.execute_batch(&sql)?;
        let ts = Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            params![version, ts],
        )?;
        applied += 1;
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_migration_runner() -> Result<(), MigrateError> {
        // temporary DB
        let db_file = NamedTempFile::new()?;
        let conn = Connection::open(db_file.path())?;
        // temporary migrations dir
        let tmp_dir = tempfile::tempdir()?;
        let mut f1 = fs::File::create(tmp_dir.path().join("001_create_test1.sql"))?;
        writeln!(f1, "CREATE TABLE test1 (id INTEGER PRIMARY KEY);")?;
        let mut f2 = fs::File::create(tmp_dir.path().join("002_create_test2.sql"))?;
        writeln!(f2, "CREATE TABLE test2 (id INTEGER PRIMARY KEY);")?;
        let applied = run_migrations(&conn, &tmp_dir.path().to_path_buf())?;
        assert_eq!(applied, 2);
        let applied2 = run_migrations(&conn, &tmp_dir.path().to_path_buf())?;
        assert_eq!(applied2, 0);
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='test1'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(exists, 1);
        let exists2: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='test2'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(exists2, 1);
        Ok(())
    }
}
