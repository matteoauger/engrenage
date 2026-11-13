//! Auth middleware for the `/api` mount: session validation, context
//! injection, login allowlist.

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::session::{self, COOKIE_NAME};
use crate::error::AppError;
use crate::state::AppState;

/// Everything under `/api` requires a valid session except the login
/// endpoint itself. On success the `SessionContext` is injected into
/// request extensions for the handlers.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    // Within the /api nest the login route is "/auth/login".
    if req.uri().path() == "/auth/login" {
        return next.run(req).await;
    }

    let result = match cookie_token(req.headers()) {
        Some(token) => session::validate(&state.db, &token).await,
        None => Ok(None),
    };

    match result {
        Ok(Some(ctx)) => {
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        Ok(None) => AppError::Unauthorized("authentication required".to_owned()).into_response(),
        Err(err) => AppError::Internal(err).into_response(),
    }
}

/// Extract the raw session token from the request's Cookie header.
fn cookie_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    cookies.split(';').find_map(|pair| {
        let pair = pair.trim();
        pair.strip_prefix(COOKIE_NAME)
            .and_then(|rest| rest.strip_prefix('='))
            .map(|value| value.to_owned())
    })
}
