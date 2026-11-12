//! Async wrappers that move the expensive crypto primitives off the tokio
//! async workers onto the dedicated blocking pool (same pattern as
//! `Db::with_conn`). Argon2 costs tens of milliseconds of pure CPU — running
//! it inline in a handler task would stall everything queued on that worker.

use anyhow::{Context, Result};

use crate::auth::crypto;

/// argon2id hash on the blocking pool.
pub async fn hash_password(plain: String) -> Result<String> {
    tokio::task::spawn_blocking(move || crypto::hash_password(&plain))
        .await
        .context("password hashing task panicked")?
}

/// argon2id verify on the blocking pool.
pub async fn verify_password(plain: String, phc_hash: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || crypto::verify_password(&plain, &phc_hash))
        .await
        .context("password verification task panicked")?
}
