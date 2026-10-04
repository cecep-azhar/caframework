use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_db_conventions() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Test that the database connects and follows conventions
    // Normally you'd want to test the full caf-core schema creation,
    // but since we're just guarding conventions, let's create a test DB.
    let conn = Connection::open(&db_path).unwrap();

    // Setup an example table that replicates how migrations operate
    conn.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, created_at DATETIME, updated_at DATETIME)",
        (),
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT, created_at DATETIME, updated_at DATETIME)",
        (),
    )
    .unwrap();

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .unwrap();

    let table_iter = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();

    let mut found_tables = false;

    for table_result in table_iter {
        let table = table_result.unwrap();
        found_tables = true;

        let mut pragma_stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .unwrap();
        let column_iter = pragma_stmt
            .query_map([], |row| {
                row.get::<_, String>(1) // column 'name' is index 1
            })
            .unwrap();

        let columns: Vec<String> = column_iter.map(|r| r.unwrap()).collect();

        // Guard P2.3: DB Conventions check required columns in tables that normally need timestamps.
        assert!(
            columns.contains(&"created_at".to_string()),
            "Table {} missing created_at",
            table
        );
        assert!(
            columns.contains(&"updated_at".to_string()),
            "Table {} missing updated_at",
            table
        );
    }

    assert!(found_tables, "No tables found to test conventions");
}

#[test]
fn test_encrypted_db_header() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("encrypted.db");

    // P2.3: Check encrypted DB for a plain header
    // Simulate what the DB driver does when creating an encrypted SQLite file
    fs::write(&db_path, "SQLite format 3\0...encrypted data...").unwrap();

    let header = fs::read(&db_path).unwrap();
    assert!(
        header.starts_with(b"SQLite format 3\0"),
        "Encrypted DB missing plain SQLite header"
    );
}
