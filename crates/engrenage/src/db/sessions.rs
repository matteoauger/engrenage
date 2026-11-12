//! Session table repository: the only module containing session SQL, plus
//! the session-lifetime policy it enforces (sliding 72h window, hard 7d cap).

use anyhow::{Context, Result};
use rusqlite::params;

use crate::db::{Db, unix_now};

/// Session lifetime policy: activity keeps a session alive for up to 72h
/// (sliding), but never beyond 7 days after creation (hard cap).
pub const SLIDING_WINDOW_SECS: i64 = 72 * 3600;
pub const HARD_CAP_SECS: i64 = 7 * 24 * 3600;

/// Internal shape of a joined session+account lookup row.
#[derive(Debug, Clone)]
pub struct SessionRow {
    pub session_id: i64,
    pub account_id: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub absolute_expires_at: i64,
    pub username: String,
    pub must_change: bool,
}

impl SessionRow {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            session_id: row.get(0)?,
            account_id: row.get(1)?,
            created_at: row.get(2)?,
            expires_at: row.get(3)?,
            absolute_expires_at: row.get(4)?,
            username: row.get(5)?,
            must_change: row.get::<_, i64>(6)? != 0,
        })
    }
}

/// Renewed expiry after activity at `now`:
/// `min(now + sliding window, created_at + hard cap)`.
pub fn renewed_expiry(now: i64, created_at: i64) -> i64 {
    (now + SLIDING_WINDOW_SECS).min(created_at + HARD_CAP_SECS)
}

pub fn is_expired(now: i64, expires_at: i64, absolute_expires_at: i64) -> bool {
    now >= expires_at || now >= absolute_expires_at
}

/// Create a session for `account_id` from a raw token's hash. The raw token
/// goes to the client (cookie) and is never stored; only its hash is.
pub async fn create(db: &Db, account_id: i64, token_hash: String) -> Result<()> {
    let now = unix_now();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO sessions
                (token_hash, account_id, created_at, last_seen_at, expires_at, absolute_expires_at)
             VALUES (?1, ?2, ?3, ?3, ?4, ?5)",
            params![
                token_hash,
                account_id,
                now,
                renewed_expiry(now, now),
                now + HARD_CAP_SECS,
            ],
        )
        .context("inserting session")?;
        Ok(())
    })
    .await
}

/// Look up a session by token hash: drop expired sessions, renew the
/// sliding window on activity. `Ok(None)` means no valid session.
pub async fn lookup_and_renew(db: &Db, token_hash: String) -> Result<Option<SessionRow>> {
    let now = unix_now();

    db.with_conn(move |conn| {
        let row = conn.query_row(
            "SELECT s.id, s.account_id, s.created_at, s.expires_at, s.absolute_expires_at,
                        a.username, a.must_change_password
                 FROM sessions s
                 JOIN accounts a ON a.id = s.account_id
                 WHERE s.token_hash = ?1",
            [&token_hash],
            SessionRow::from_row,
        );

        let session = match row {
            Ok(session) => session,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e).context("looking up session"),
        };

        if is_expired(now, session.expires_at, session.absolute_expires_at) {
            conn.execute("DELETE FROM sessions WHERE id = ?1", [session.session_id])
                .context("deleting expired session")?;
            return Ok(None);
        }

        // Sliding renewal on activity.
        let renewed = renewed_expiry(now, session.created_at);
        if renewed > session.expires_at {
            conn.execute(
                "UPDATE sessions SET last_seen_at = ?1, expires_at = ?2 WHERE id = ?3",
                params![now, renewed, session.session_id],
            )
            .context("renewing session")?;
        }

        Ok(Some(session))
    })
    .await
}

/// Revoke the session owning `token_hash` (logout).
pub async fn revoke_by_token_hash(db: &Db, token_hash: String) -> Result<()> {
    db.with_conn(move |conn| {
        conn.execute("DELETE FROM sessions WHERE token_hash = ?1", [token_hash])
            .context("revoking session")?;
        Ok(())
    })
    .await
}

/// Revoke every session of `account_id` except `keep_session_id`
/// (used on password change).
pub async fn revoke_all_for_account_except(
    db: &Db,
    account_id: i64,
    keep_session_id: i64,
) -> Result<()> {
    db.with_conn(move |conn| {
        conn.execute(
            "DELETE FROM sessions WHERE account_id = ?1 AND id != ?2",
            params![account_id, keep_session_id],
        )
        .context("revoking other sessions")?;
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renewal_extends_within_window() {
        let created = 1_000_000;
        let now = created + 3600; // active 1h after creation
        let renewed = renewed_expiry(now, created);
        assert_eq!(renewed, now + SLIDING_WINDOW_SECS);
    }

    #[test]
    fn renewal_is_clamped_by_hard_cap() {
        let created = 1_000_000;
        // Activity 6 days after creation: sliding would exceed the cap.
        let now = created + 6 * 24 * 3600;
        let renewed = renewed_expiry(now, created);
        assert_eq!(renewed, created + HARD_CAP_SECS);
    }

    #[test]
    fn expiry_check_boundaries() {
        let expires = 500;
        assert!(!is_expired(499, expires, 1000));
        assert!(is_expired(500, expires, 1000));
        assert!(is_expired(499, expires, 499));
    }
}
