pub mod assets;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::auth::handlers;
use crate::auth::middleware::auth_middleware;
use crate::error::AppError;
use crate::state::AppState;

/// Build the whole application router: health check, auth API, protected API
/// mount, and SPA assets as fallback for anything else.
pub fn build_router(state: AppState) -> Router {
    tracing::debug!(data_dir = %state.config.data_dir.display(), "building router");

    // Everything under /api requires a session, except /api/auth/login
    // (allowlisted inside the middleware).
    let api = Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/login", post(handlers::login))
                .route("/logout", post(handlers::logout))
                .route("/me", get(handlers::me))
                .route("/change-password", post(handlers::change_password)),
        )
        .fallback(api_fallback)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

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
