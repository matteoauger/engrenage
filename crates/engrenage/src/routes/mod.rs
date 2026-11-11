pub mod assets;

use axum::{Json, Router, routing::get};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::{error::AppError, state::AppState};

/// Build the whole application router
pub fn build_router(state: AppState) -> Router {
    tracing::debug!(data_dir = %state.config.data_dir.display(), "building router");

    let api = Router::new().fallback(api_fallback);

    Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .fallback(assets::handler)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn api_fallback() -> AppError {
    AppError::NotFound("unknown api route".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use assets::has_index;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        build_router(AppState::new(Config::default()))
    }

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let response = app()
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
        let response = app()
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
        let response = app()
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
        let response = app()
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
}
