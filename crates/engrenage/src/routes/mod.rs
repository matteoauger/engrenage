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
