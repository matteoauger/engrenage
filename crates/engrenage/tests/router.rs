//! Router integration tests, driven through the crate's public API.
//!
//! These exercise `build_router` in-process — the same surface the binary
//! serves. Per the Rust Book's test organization, tests that go through the
//! public API live in `tests/`, not inline `#[cfg(test)]` modules.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

mod common;

use common::{app, body_string, has_index};

fn router() -> Router {
    app()
}

#[tokio::test]
async fn health_returns_ok() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_string(response).await, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_route_is_404() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/api/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(body_string(response).await.contains("unknown api route"));
}

#[tokio::test]
async fn missing_asset_with_extension_is_404() {
    // Holds whether or not `web/dist` is built: a missing extension'd
    // path is never SPA-fallbacked.
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/assets/does-not-exist.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(body_string(response).await.contains("no such asset"));
}

#[tokio::test]
async fn unknown_route_falls_back_to_spa() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/some-page")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if has_index() {
        assert_eq!(response.status(), StatusCode::OK);
    } else {
        // dist not built: fallback must say so, not panic.
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(body_string(response).await.contains("npm run build"));
    }
}
