//! Ordered, hand-rolled migrations. Add a new `(version, batch)` pair at the
//! end; every batch runs once, guarded by `schema_version`.

use anyhow::Context;
use rusqlite::Connection;

const MIGRATIONS: &[(i64, &str)] = &[(
    1,
    r#"
        CREATE TABLE accounts (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            must_change_password INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE sessions (
            id INTEGER PRIMARY KEY,
            token_hash TEXT NOT NULL UNIQUE,
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            created_at INTEGER NOT NULL,
            last_seen_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            absolute_expires_at INTEGER NOT NULL
        );

        CREATE TABLE audit_log (
            id INTEGER PRIMARY KEY,
            at INTEGER NOT NULL,
            event TEXT NOT NULL,
            username TEXT,
            detail TEXT
        );
        "#,
)];

pub(super) fn migrate(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );",
    )?;

    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?;

    for &(version, batch) in MIGRATIONS {
        if version <= current {
            continue;
        }
        // Atomic: the batch and its version marker commit together, so a
        // crash mid-migration rolls back instead of leaving unversioned
        // tables behind (which would fail the next boot).
        conn.execute_batch(&format!(
            "BEGIN IMMEDIATE; {batch} INSERT INTO schema_version (version) VALUES ({version}); COMMIT;"
        ))
        .with_context(|| format!("applying migration {version}"))?;
        tracing::debug!(version, "applied migration");
    }

    Ok(())
}
