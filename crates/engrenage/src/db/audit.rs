//! Audit log table repository. Failures are logged but never fail the
//! caller's request: audit is an observation, not a control.

use anyhow::Context;
use rusqlite::params;

use crate::db::{Db, unix_now};

pub async fn record(db: &Db, event: &str, username: &str, detail: Option<String>) {
    let at = unix_now();
    let event_name = event.to_owned();
    let username = username.to_owned();
    if let Err(err) = db
        .with_conn(move |conn| {
            conn.execute(
                "INSERT INTO audit_log (at, event, username, detail) VALUES (?1, ?2, ?3, ?4)",
                params![at, event_name, username, detail],
            )
            .context("writing audit entry")?;
            Ok(())
        })
        .await
    {
        tracing::warn!(error = ?err, event, "could not write audit entry");
    }
}
