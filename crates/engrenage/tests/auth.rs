//! Auth foundation integration tests, driven through the public API.
//!
//! Crypto-dependent tests run against the real `auth/crypto.rs`
//! implementation.

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;

mod common;

use common::{body_string, test_app};

async fn send(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Option<String>,
    cookie: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let body = Body::from(body.unwrap_or_default());
    router.oneshot(builder.body(body).unwrap()).await.unwrap()
}

/// Extract the `engrenage_session` cookie as a full `Cookie` header value
/// (ready to pass to a request).
fn session_cookie(response: &axum::response::Response) -> Option<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            let rest = value.strip_prefix("engrenage_session=")?;
            Some(format!(
                "engrenage_session={}",
                rest.split(';').next().unwrap_or_default()
            ))
        })
}

#[tokio::test]
async fn login_with_unknown_user_is_invalid_credentials() {
    let app = test_app();
    let response = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "ghost", "password": "whatever"}).to_string()),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_string(response).await;
    assert!(body.contains("invalid credentials"), "body: {body}");
}

#[tokio::test]
async fn me_requires_authentication() {
    let app = test_app();
    let response = send(app.router.clone(), "GET", "/api/auth/me", None, None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_success_sets_cookie_and_flags_password_change() {
    let app = test_app();
    let hash = engrenage::auth::crypto::hash_password("correct-horse").unwrap();
    let db = app.db.clone();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO accounts (username, password_hash, must_change_password, created_at)
             VALUES ('admin', ?1, 1, 0)",
            [&hash],
        )
        .unwrap();
        Ok(())
    })
    .await
    .unwrap();

    let response = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "admin", "password": "correct-horse"}).to_string()),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let cookie = session_cookie(&response).expect("session cookie must be set");
    assert!(!cookie.is_empty());

    let body = body_string(response).await;
    assert!(body.contains("\"username\":\"admin\""), "body: {body}");
    assert!(body.contains("\"mustChangePassword\":true"), "body: {body}");
}

#[tokio::test]
async fn me_works_with_valid_session() {
    let app = test_app();
    let hash = engrenage::auth::crypto::hash_password("correct-horse").unwrap();
    let db = app.db.clone();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO accounts (username, password_hash, must_change_password, created_at)
             VALUES ('admin', ?1, 0, 0)",
            [&hash],
        )
        .unwrap();
        Ok(())
    })
    .await
    .unwrap();

    let login = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "admin", "password": "correct-horse"}).to_string()),
        None,
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    let cookie = session_cookie(&login)
        .unwrap_or_else(|| panic!("no session cookie; headers: {:?}", login.headers()));

    let me = send(
        app.router.clone(),
        "GET",
        "/api/auth/me",
        None,
        Some(&cookie),
    )
    .await;
    assert_eq!(me.status(), StatusCode::OK);
    assert!(body_string(me).await.contains("\"username\":\"admin\""));
}

#[tokio::test]
async fn change_password_clears_flag_and_revokes_other_sessions() {
    let app = test_app();
    let hash = engrenage::auth::crypto::hash_password("correct-horse").unwrap();
    let db = app.db.clone();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO accounts (username, password_hash, must_change_password, created_at)
             VALUES ('admin', ?1, 1, 0)",
            [&hash],
        )
        .unwrap();
        Ok(())
    })
    .await
    .unwrap();

    // Two sessions for the same account.
    let login_a = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "admin", "password": "correct-horse"}).to_string()),
        None,
    )
    .await;
    let cookie_a = session_cookie(&login_a).expect("session cookie a");

    let login_b = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "admin", "password": "correct-horse"}).to_string()),
        None,
    )
    .await;
    let cookie_b = session_cookie(&login_b).expect("session cookie b");

    // Change the password with session b.
    let change = send(
        app.router.clone(),
        "POST",
        "/api/auth/change-password",
        Some(
            json!({"currentPassword": "correct-horse", "newPassword": "new-strong-pw"}).to_string(),
        ),
        Some(&cookie_b),
    )
    .await;
    assert_eq!(change.status(), StatusCode::OK);

    // The other session (a) must be revoked.
    let me_a = send(
        app.router.clone(),
        "GET",
        "/api/auth/me",
        None,
        Some(&cookie_a),
    )
    .await;
    assert_eq!(me_a.status(), StatusCode::UNAUTHORIZED);

    // Session b survives and no longer requires a password change.
    let me_b = send(
        app.router.clone(),
        "GET",
        "/api/auth/me",
        None,
        Some(&cookie_b),
    )
    .await;
    assert_eq!(me_b.status(), StatusCode::OK);
    assert!(
        body_string(me_b)
            .await
            .contains("\"mustChangePassword\":false")
    );
}

#[tokio::test]
async fn logout_invalidates_session() {
    let app = test_app();
    let hash = engrenage::auth::crypto::hash_password("correct-horse").unwrap();
    let db = app.db.clone();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO accounts (username, password_hash, must_change_password, created_at)
             VALUES ('admin', ?1, 0, 0)",
            [&hash],
        )
        .unwrap();
        Ok(())
    })
    .await
    .unwrap();

    let login = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "admin", "password": "correct-horse"}).to_string()),
        None,
    )
    .await;
    let cookie = session_cookie(&login).expect("session cookie");

    let logout = send(
        app.router.clone(),
        "POST",
        "/api/auth/logout",
        None,
        Some(&cookie),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::OK);

    let me = send(
        app.router.clone(),
        "GET",
        "/api/auth/me",
        None,
        Some(&cookie),
    )
    .await;
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn bootstrap_creates_admin_exactly_once() {
    let app = test_app();

    engrenage::auth::bootstrap::ensure_bootstrap_admin(&app.db).await;
    engrenage::auth::bootstrap::ensure_bootstrap_admin(&app.db).await;

    let count = app
        .db
        .with_conn(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| {
                row.get::<_, i64>(0)
            })?)
        })
        .await
        .unwrap();
    assert_eq!(count, 1, "bootstrap must create exactly one admin account");
}

#[tokio::test]
async fn change_password_rejects_common_passwords() {
    let app = test_app();
    let hash = engrenage::auth::crypto::hash_password("correct-horse").unwrap();
    let db = app.db.clone();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO accounts (username, password_hash, must_change_password, created_at)
             VALUES ('admin', ?1, 1, 0)",
            [&hash],
        )
        .unwrap();
        Ok(())
    })
    .await
    .unwrap();

    let login = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": "admin", "password": "correct-horse"}).to_string()),
        None,
    )
    .await;
    let cookie = session_cookie(&login).expect("session cookie");

    // Denylisted (case-insensitive) — server-side policy check.
    let response = send(
        app.router.clone(),
        "POST",
        "/api/auth/change-password",
        Some(json!({"currentPassword": "correct-horse", "newPassword": "Password123"}).to_string()),
        Some(&cookie),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(response).await.contains("too common"));
}

#[tokio::test]
async fn login_rejects_oversized_username_without_audit_write() {
    let app = test_app();
    let long_username = "x".repeat(100);

    let response = send(
        app.router.clone(),
        "POST",
        "/api/auth/login",
        Some(json!({"username": long_username, "password": "whatever"}).to_string()),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_string(response).await;
    assert!(body.contains("invalid credentials"), "body: {body}");

    // The rejection must happen before any audit write lands.
    let audit_rows = app
        .db
        .with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM audit_log WHERE username = ?1",
                [long_username],
                |row| row.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(audit_rows, 0, "oversized username must not be audited");
}
