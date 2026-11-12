//! Shared helpers for the `engrenage` integration tests.
//!
//! `mod.rs` (not `mod_name.rs`) is deliberate: files in `tests/` each compile
//! as separate binaries, and only `mod.rs` subdirectories stay out of the
//! test-binary enumeration.
// Each test binary sees this module through its own compilation and only
// uses part of it — dead-code warnings there are expected noise, not signals.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{Router, response::Response};
use engrenage::{config::Config, db::Db, routes::build_router, state::AppState};
use http_body_util::BodyExt;

static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

/// An app wired to its own throwaway data dir + SQLite database.
///
/// Dropping a `TestApp` removes its data dir. Helpers must therefore hand
/// ownership to the caller (never return interior fields of a temporary):
/// a `test_app().router`-style field move would delete the directory before
/// the router serves a single request.
pub struct TestApp {
    pub router: Router,
    pub db: Db,
    pub data_dir: PathBuf,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Best effort: a failed cleanup must not fail the test run (and
        // panicking in Drop during unwinding would abort).
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}

/// App + its DB, for tests that need to seed or inspect state directly.
pub fn test_app() -> TestApp {
    let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
    let data_dir = std::env::temp_dir().join(format!("engrenage-test-{}-{id}", std::process::id()));
    std::fs::create_dir_all(&data_dir).expect("creating test data dir");

    let config = Config {
        data_dir: data_dir.clone(),
        ..Config::default()
    };
    let db = Db::open(&data_dir.join("engrenage.db")).expect("opening test database");

    TestApp {
        router: build_router(AppState::new(config, db.clone())),
        db,
        data_dir,
    }
}

/// Whether the SPA bundle was built (branching helper, mirrors runtime).
pub fn has_index() -> bool {
    engrenage::routes::assets::has_index()
}

/// Collect a response body as a UTF-8 string.
pub async fn body_string(response: Response) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}
