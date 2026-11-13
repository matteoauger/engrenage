//! First-boot admin bootstrap.

use rand::distr::{Alphanumeric, SampleString};

use crate::auth::blocking;
use crate::db::{self, Db, accounts, audit};

/// Create the default admin account when none exists (first boot).
///
/// Default flow: random 20-character password, printed **once** to the logs.
/// `ENGRENAGE_BOOTSTRAP_PASSWORD` may override it (e.g. for headless/Docker
/// boots); the printed password must still be changed at first login either
/// way. Crypto failures degrade gracefully: the server boots without an
/// admin account instead of crashing.
pub async fn ensure_bootstrap_admin(db: &Db) {
    let account_count = match accounts::count(db).await {
        Ok(count) => count,
        Err(err) => {
            tracing::error!(error = ?err, "could not count accounts, skipping admin bootstrap");
            return;
        }
    };

    match account_count {
        0 => {}
        _ => return,
    }

    let password = match std::env::var("ENGRENAGE_BOOTSTRAP_PASSWORD") {
        Ok(pw) if !pw.is_empty() => {
            tracing::info!("admin account created from ENGRENAGE_BOOTSTRAP_PASSWORD");
            pw
        }
        _ => {
            let generated = Alphanumeric.sample_string(&mut rand::rng(), 20);
            tracing::warn!(
                "\n\n  ================================================\n  \
                 Initial admin account:  username=\"admin\"\n  \
                 Initial admin password: \"{}\"\n  \
                 (change it at first login — shown only this once)\n  \
                 ================================================\n",
                generated
            );
            generated
        }
    };

    match blocking::hash_password(password).await {
        Ok(hash) => {
            let now = db::unix_now();
            match accounts::insert_admin(db, hash, now).await {
                Ok(()) => {
                    audit::record(db, "bootstrap", "admin", None).await;
                    tracing::info!(
                        "default admin account created (password change forced at first login)"
                    );
                }
                Err(err) => {
                    tracing::error!(error = ?err, "could not insert bootstrap admin");
                }
            }
        }
        Err(err) => {
            tracing::error!(
                error = ?err,
                "admin bootstrap skipped: password hashing unavailable (auth/crypto.rs not implemented?)"
            );
        }
    }
}
