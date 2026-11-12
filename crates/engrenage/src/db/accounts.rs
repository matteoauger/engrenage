//! Account table repository: the only module containing account SQL.

use anyhow::{Context, Result};
use rusqlite::params;

use crate::db::Db;

/// An account row. Named fields over positional tuples for clarity.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub must_change_password: bool,
}

impl Account {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            must_change_password: row.get::<_, i64>(3)? != 0,
        })
    }
}

pub async fn find_by_username(db: &Db, username: &str) -> Result<Option<Account>> {
    let username = username.to_owned();
    db.with_conn(move |conn| {
        let row = conn.query_row(
            "SELECT id, username, password_hash, must_change_password
             FROM accounts WHERE username = ?1",
            [&username],
            Account::from_row,
        );
        Ok(match row {
            Ok(account) => Some(account),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(e.into()),
        })
    })
    .await
}

pub async fn count(db: &Db) -> Result<i64> {
    db.with_conn(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?))
        .await
}

pub async fn find_password_hash(db: &Db, account_id: i64) -> Result<String> {
    db.with_conn(move |conn| {
        conn.query_row(
            "SELECT password_hash FROM accounts WHERE id = ?1",
            [account_id],
            |row| row.get(0),
        )
        .map_err(|e| anyhow::Error::from(e).context("loading account"))
    })
    .await
}

/// Store a new password hash and clear the `must_change_password` flag.
pub async fn update_password(db: &Db, account_id: i64, password_hash: String) -> Result<()> {
    db.with_conn(move |conn| {
        conn.execute(
            "UPDATE accounts SET password_hash = ?1, must_change_password = 0 WHERE id = ?2",
            params![password_hash, account_id],
        )
        .context("updating password")?;
        Ok(())
    })
    .await
}

pub async fn insert_admin(db: &Db, password_hash: String, created_at: i64) -> Result<()> {
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO accounts (username, password_hash, must_change_password, created_at)
             VALUES ('admin', ?1, 1, ?2)",
            params![password_hash, created_at],
        )
        .context("inserting bootstrap admin")?;
        Ok(())
    })
    .await
}
