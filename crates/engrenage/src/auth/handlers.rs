//! Endpoint handlers only: extract input, delegate to the store/session/
//! policy modules, shape responses. SQL and cross-cutting concerns live in
//! the sibling modules.

use anyhow::Result;
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::{
    blocking, crypto, password_policy,
    session::{self, COOKIE_NAME, SessionContext},
};
use crate::db::{accounts, audit};
use crate::error::AppError;
use crate::state::AppState;

/// JSON returned by `me`/`login` about the authenticated account.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub username: String,
    pub must_change_password: bool,
}

/// Binds unauthenticated writes: a rejected oversized username never
/// reaches the account lookup or `audit::record`.
const MAX_USERNAME_LEN: usize = 64;

#[derive(Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Response, AppError> {
    let db = &state.db;

    if input.username.len() > MAX_USERNAME_LEN {
        return Err(AppError::BadRequest("invalid credentials".to_owned()));
    }

    let account = accounts::find_by_username(db, &input.username).await?;

    // Verify even for unknown users (against a dummy hash) so response timing
    // does not leak account existence. Any crypto error fails closed.
    const DUMMY_HASH: &str =
        "$argon2id$v=19$m=19456,t=2,p=1$Zm9vYmFy$CnmrfOLI2vYEFlGgUsbwgVUi1D3I6Si5OIAreqMxDHA";
    let password_ok: Result<bool> = match &account {
        Some(account) => {
            blocking::verify_password(input.password.clone(), account.password_hash.clone()).await
        }
        None => blocking::verify_password(input.password.clone(), DUMMY_HASH.to_owned()).await,
    };

    match (account, password_ok) {
        (Some(account), Ok(true)) => {
            let token = crypto::generate_session_token()?;
            session::create(db, account.id, &token).await?;
            audit::record(db, "login", &account.username, None).await;

            let response = Json(AccountInfo {
                username: account.username.clone(),
                must_change_password: account.must_change_password,
            })
            .into_response();
            with_session_cookie(response, &token, session::HARD_CAP_SECS)
        }
        // Wrong password, unknown user, or crypto failure: same response,
        // but the audit detail distinguishes a server-side crypto fault.
        (Some(_), Err(err)) => {
            audit::record(
                db,
                "login_failed",
                &input.username,
                Some(format!("crypto failure: {err}")),
            )
            .await;
            Err(AppError::Unauthorized("invalid credentials".to_owned()))
        }
        _ => {
            audit::record(db, "login_failed", &input.username, None).await;
            Err(AppError::Unauthorized("invalid credentials".to_owned()))
        }
    }
}

pub async fn me(Extension(ctx): Extension<SessionContext>) -> Json<AccountInfo> {
    Json(AccountInfo {
        username: ctx.username,
        must_change_password: ctx.must_change_password,
    })
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(ctx): Extension<SessionContext>,
) -> Result<Response, AppError> {
    session::revoke_by_token_hash(&state.db, &ctx.token_hash).await?;
    audit::record(&state.db, "logout", &ctx.username, None).await;

    let response = Json(json!({ "status": "ok" })).into_response();
    with_session_cookie(response, "", 0)
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(ctx): Extension<SessionContext>,
    Json(input): Json<ChangePasswordInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    password_policy::validate_new_password(&input.new_password)
        .map_err(|reason| AppError::BadRequest(reason.to_string()))?;

    let current_hash = accounts::find_password_hash(&state.db, ctx.account_id).await?;
    if !blocking::verify_password(input.current_password.clone(), current_hash).await? {
        audit::record(&state.db, "password_change_failed", &ctx.username, None).await;
        return Err(AppError::BadRequest(
            "current password is incorrect".to_owned(),
        ));
    }

    let new_hash = blocking::hash_password(input.new_password.clone()).await?;
    accounts::update_password(&state.db, ctx.account_id, new_hash).await?;

    // A changed password invalidates every other session of this account.
    session::revoke_all_for_account_except(&state.db, ctx.account_id, ctx.session_id).await?;
    audit::record(&state.db, "password_changed", &ctx.username, None).await;

    Ok(Json(json!({ "status": "ok" })))
}

/// Attach a `Set-Cookie` header for the session cookie; an empty token with
/// `max_age = 0` clears it.
///
/// The `Secure` attribute is intentionally omitted until a TLS story exists —
/// it would make browsers drop the cookie on plain-HTTP deployments
/// (see docs/DECISIONS.md §6).
fn with_session_cookie(
    mut response: Response,
    token: &str,
    max_age: i64,
) -> Result<Response, AppError> {
    let cookie =
        format!("{COOKIE_NAME}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age}");
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| AppError::Internal(e.into()))?,
    );
    Ok(response)
}
