mod common;

use axum_test::TestServer;
use serde_json::{json, Value};

/// Helper to perform real login and retrieve the access_token.
/// Used to test the actual /login endpoint integration.
async fn login_and_get_token(api: &TestServer, username: &str) -> String {
    let response = api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let token = body["access_token"]
        .as_str()
        .expect("Login response missing 'access_token'");

    format!("Bearer {}", token)
}

// --- Integration Tests ---

#[tokio::test]
async fn test_health_check() {
    let app = common::spawn_app().await;
    app.api.get("/health").await.assert_status_ok();
}

#[tokio::test]
async fn test_public_route_access() {
    let app = common::spawn_app().await;
    app.api.get("/public").await.assert_status_ok();
}

#[tokio::test]
async fn test_login_invalid_username_fails_401() {
    let app = common::spawn_app().await;

    app.api
        .post("/login")
        .json(&json!({
            "username": "non_existent_user",
            "password": "password123"
        }))
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn test_login_invalid_password_fails_401() {
    let app = common::spawn_app().await;

    app.api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "wrong_password"
        }))
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn test_protected_routes_missing_token_fail_401() {
    let app = common::spawn_app().await;

    app.api
        .get("/users/profile")
        .await
        .assert_status_unauthorized();

    app.api
        .get("/admin/dashboard")
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn test_standard_user_flow_bob() {
    let app = common::spawn_app().await;

    // 1. Real Login as "bob" (Standard User)
    let token_bob = login_and_get_token(&app.api, "bob").await;

    // 2. Access own profile (Should SUCCEED - 200 OK)
    app.api
        .get("/users/profile")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_ok();

    // 3. Try to access admin dashboard (Should FAIL - 403 Forbidden)
    app.api
        .get("/admin/dashboard")
        .add_header("Authorization", &token_bob)
        .await
        .assert_status_forbidden();
}

#[tokio::test]
async fn test_admin_flow_alice() {
    let app = common::spawn_app().await;

    // 1. Real Login as "alice" (Admin User)
    let token_alice = login_and_get_token(&app.api, "alice").await;

    // 2. Access admin dashboard (Should SUCCEED - 200 OK)
    app.api
        .get("/admin/dashboard")
        .add_header("Authorization", &token_alice)
        .await
        .assert_status_ok();

    // 3. Access profile (Should also SUCCEED - 200 OK)
    app.api
        .get("/users/profile")
        .add_header("Authorization", &token_alice)
        .await
        .assert_status_ok();
}
