mod common;

use axum::http::StatusCode;
use serde_json::json;

// =============================================================================
// TESTES DE LOGIN E AUTENTICAÇÃO
// =============================================================================

#[tokio::test]
async fn test_login_success() {
    let app = common::spawn_app().await;

    // Create unique test user to avoid conflicts with parallel tests
    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Failed to create test user");

    // Ensure MFA is disabled for this test
    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .expect("Failed to disable MFA");

    // Perform login
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::OK,
        "Login should succeed. Body: {}",
        login_response.text()
    );

    let body: serde_json::Value = login_response.json();
    assert!(
        body.get("access_token").is_some(),
        "Response should contain access_token"
    );
    assert!(
        body.get("refresh_token").is_some(),
        "Response should contain refresh_token"
    );
    assert_eq!(body["token_type"], "Bearer");
}

#[tokio::test]
async fn test_login_fail_wrong_password() {
    let app = common::spawn_app().await;

    // Create unique test user
    let (username, _email, _password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Failed to create test user");

    // Ensure MFA is disabled
    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .expect("Failed to disable MFA");

    // Attempt login with wrong password
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": "WrongPassword123!"
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Login with wrong password should fail"
    );

    let body: serde_json::Value = login_response.json();
    assert!(
        body.get("error").is_some(),
        "Response should contain error message"
    );
}

#[tokio::test]
async fn test_login_fail_nonexistent_user() {
    let app = common::spawn_app().await;

    // Try to login with a user that doesn't exist
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": format!("nonexistent_user_{}", uuid::Uuid::new_v4()),
            "password": "anypassword123"
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Login with nonexistent user should fail"
    );
}

// =============================================================================
// TESTES DE REFRESH TOKEN ROTATION
// =============================================================================

#[tokio::test]
async fn test_refresh_token_rotation_success() {
    let app = common::spawn_app().await;

    // Create unique test user and login
    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Failed to create test user");

    // Ensure MFA is disabled
    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .expect("Failed to disable MFA");

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("Login should return refresh_token");

    // Use refresh token to get new tokens
    let refresh_response = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        refresh_response.status_code(),
        StatusCode::OK,
        "Refresh token should work. Body: {}",
        refresh_response.text()
    );

    let refresh_body: serde_json::Value = refresh_response.json();
    assert!(
        refresh_body.get("access_token").is_some(),
        "Should return new access_token"
    );
    assert!(
        refresh_body.get("refresh_token").is_some(),
        "Should return new refresh_token"
    );

    // Verify rotation: new token should be different from old token
    let new_refresh_token = refresh_body["refresh_token"].as_str().unwrap();
    assert_ne!(
        refresh_token, new_refresh_token,
        "New refresh token should be different (rotation)"
    );
}

#[tokio::test]
async fn test_refresh_token_revoked_after_use() {
    let app = common::spawn_app().await;

    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Failed to create test user");

    // Ensure MFA is disabled
    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .expect("Failed to disable MFA");

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap().to_string();

    // Use refresh token once (should work)
    let first_refresh = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        first_refresh.status_code(),
        StatusCode::OK,
        "First use should work"
    );

    // Try to use the same refresh token again (should fail - token is revoked after use)
    let second_refresh = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        second_refresh.status_code(),
        StatusCode::UNAUTHORIZED,
        "Used token should not work again (revoked after first use)"
    );
}

#[tokio::test]
async fn test_refresh_token_theft_detection_revokes_family() {
    let app = common::spawn_app().await;

    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Failed to create test user");

    // Ensure MFA is disabled
    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .expect("Failed to disable MFA");

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let token1 = login_body["refresh_token"].as_str().unwrap().to_string();

    // Rotate to get token2
    let refresh1 = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token1
        }))
        .await;

    let refresh1_body: serde_json::Value = refresh1.json();
    let token2 = refresh1_body["refresh_token"].as_str().unwrap().to_string();

    // Rotate again to get token3
    let refresh2 = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token2
        }))
        .await;

    let refresh2_body: serde_json::Value = refresh2.json();
    let token3 = refresh2_body["refresh_token"].as_str().unwrap().to_string();

    // Simulate theft: attacker tries to use old token1 (already used)
    // This should trigger family revocation
    let theft_attempt = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token1
        }))
        .await;

    assert_eq!(
        theft_attempt.status_code(),
        StatusCode::UNAUTHORIZED,
        "Old token should fail"
    );

    // Verify that token3 (most recent) was also revoked due to family revocation
    let token3_attempt = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": token3
        }))
        .await;

    assert_eq!(
        token3_attempt.status_code(),
        StatusCode::UNAUTHORIZED,
        "Token3 should be revoked after theft detection (family revocation)"
    );
}

// =============================================================================
// TESTES DE REGISTRO
// =============================================================================

#[tokio::test]
async fn test_register_success() {
    let app = common::spawn_app().await;

    // CORREÇÃO: Usar register_test_user e passar &app.api (TestServer)
    let (username, _email, password) = common::register_test_user(&app.api)
        .await
        .expect("Failed to register user");

    // Wait a moment for async operations to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Verify that we can login with the registered credentials
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    assert_eq!(
        login_response.status_code(),
        StatusCode::OK,
        "Should be able to login with registered credentials"
    );
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let app = common::spawn_app().await;

    // CORREÇÃO: Usar register_test_user e passar &app.api
    let (username, _email, _password) = common::register_test_user(&app.api)
        .await
        .expect("Failed to register first user");

    // Try to register with the same username but different email
    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": username,
            "email": "another_email@test.com",
            "password": "AnotherP@ssw0rd!123"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CONFLICT,
        "Duplicate username should return 409 CONFLICT"
    );
}

#[tokio::test]
async fn test_register_weak_password() {
    let app = common::spawn_app().await;

    let counter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": format!("test_user_{}", counter),
            "email": format!("test_{}@test.com", counter),
            "password": "weak" // Weak password
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::BAD_REQUEST,
        "Weak password should return 400 BAD_REQUEST"
    );
}

// =============================================================================
// TESTES DE LOGOUT
// =============================================================================

#[tokio::test]
async fn test_logout_success() {
    let app = common::spawn_app().await;

    // Create unique test user and login
    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Failed to create test user");

    // Ensure MFA is disabled
    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .expect("Failed to disable MFA");

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: serde_json::Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Perform logout
    let logout_response = app
        .api
        .post("/logout")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        logout_response.status_code(),
        StatusCode::OK,
        "Logout should succeed"
    );

    // Try to use the refresh token after logout (should fail - token is revoked)
    let refresh_after_logout = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(
        refresh_after_logout.status_code(),
        StatusCode::UNAUTHORIZED,
        "Revoked token should not work after logout"
    );
}
