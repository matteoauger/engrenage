//! Session logic: token validation orchestration + the client-facing
//! session context. All session SQL and the lifetime policy live in
//! `db::sessions`; this module maps repository rows into the request-level
//! types handlers use.

use anyhow::{Context, Result};

use crate::auth::crypto;
use crate::db::{self, sessions};

/// Session validity + owning account, injected into request extensions by
/// the auth middleware.
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session_id: i64,
    pub account_id: i64,
    pub username: String,
    pub must_change_password: bool,
    /// Hashed form of the session token (revocation + logout need it).
    pub token_hash: String,
}

pub const COOKIE_NAME: &str = "engrenage_session";

// Lifetime policy lives with the sessions table; re-exported here because
// cookie expiry is an auth-level concern.
pub use sessions::{HARD_CAP_SECS, is_expired, renewed_expiry};

/// Validate a raw token: lookup by hash (expired sessions dropped, sliding
/// window renewed by the repository), then map to the request context.
/// `Ok(None)` means no valid session.
pub async fn validate(db: &db::Db, raw_token: &str) -> Result<Option<SessionContext>> {
    let token_hash = crypto::hash_token(raw_token)?;
    Ok(sessions::lookup_and_renew(db, token_hash.clone())
        .await?
        .map(|row| SessionContext {
            session_id: row.session_id,
            account_id: row.account_id,
            username: row.username,
            must_change_password: row.must_change,
            token_hash,
        }))
}

/// Create a session for an account from a fresh raw token (stored hashed).
pub async fn create(db: &db::Db, account_id: i64, raw_token: &str) -> Result<()> {
    let token_hash = crypto::hash_token(raw_token).context("hashing session token")?;
    sessions::create(db, account_id, token_hash).await
}

/// Revoke the session owning `token_hash` (logout).
pub async fn revoke_by_token_hash(db: &db::Db, token_hash: &str) -> Result<()> {
    sessions::revoke_by_token_hash(db, token_hash.to_owned()).await
}

/// Revoke every session of `account_id` except `keep_session_id`
/// (used on password change).
pub async fn revoke_all_for_account_except(
    db: &db::Db,
    account_id: i64,
    keep_session_id: i64,
) -> Result<()> {
    sessions::revoke_all_for_account_except(db, account_id, keep_session_id).await
}
