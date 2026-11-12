//! SQLite handle shared across requests, plus the typed repository modules
//! that own all SQL in the codebase.
//!
//! Feature modules (`auth`, and later the server domain) contain no SQL:
//! they call typed functions on the table modules here, which map rows into
//! structs (ORM-style) behind private query functions.
//!
//! `rusqlite::Connection` is `Send` but not `Sync`, so concurrent access is
//! serialized through a std mutex. Queries are short and never held across
//! an `.await`: they run inside `tokio::task::spawn_blocking` so the async
//! executor's workers are never blocked by SQLite I/O. The mutex + connection
//! are hidden inside `Db` on purpose: swapping in a connection pool later
//! (deadpool-sqlite) would not change the public API.

pub mod accounts;
pub mod audit;
mod migrate;
pub mod sessions;

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use rusqlite::Connection;

pub use accounts::Account;

#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<Connection>>,
}

impl Db {
    /// Open (or create) the database at `path` and bring the schema up to
    /// date. Safe to call on every boot.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("opening database {}", path.display()))?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        migrate::migrate(&conn)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run a closure with the connection on a blocking thread.
    pub async fn with_conn<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> anyhow::Result<T> + Send + 'static,
    {
        let conn = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().expect("db mutex poisoned");
            f(&conn)
        })
        .await
        .context("db task panicked")?
    }
}

/// Unix timestamp in seconds; shared clock helper for every table module
/// and the boot path.
pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs() as i64
}
