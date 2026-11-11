//! Shared helpers for the `engrenage` integration tests.
//!
//! `mod.rs` (not `mod_name.rs`) is deliberate: files in `tests/` each compile
//! as separate binaries, and only `mod.rs` subdirectories stay out of the
//! test-binary enumeration.

use axum::{Router, response::Response};
use engrenage::{config::Config, routes::build_router, state::AppState};
use http_body_util::BodyExt;

/// Router under test, built the same way the binary builds it.
pub fn app() -> Router {
    build_router(AppState::new(Config::default()))
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
