use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

mod common;

// Helper to hash tokens
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tokio::test]
async fn test_registration_sends_verification_email() {
    let app = common::spawn_app().await;

    let unique_username = format!("verifyuser_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "S3nh@Forte123!"
        }))
        .await;

    response.assert_status_ok();

    // Wait for async email task
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let messages = app.email_service.messages.lock().await;

    // Should have both welcome and verification emails
    assert!(messages.len() >= 1, "Should have sent at least 1 email");

    // Find verification email
    let verification_email = messages
        .iter()
        .find(|m| m.subject.contains("Verifique") || m.subject.contains("verificação"));

    assert!(
        verification_email.is_some(),
        "Should have sent verification email"
    );

    let email = verification_email.unwrap();
    assert_eq!(email.to, unique_email);
    assert!(email.context.get("verification_link").is_some());
    assert_eq!(
        email.context.get("username").unwrap().as_str().unwrap(),
        unique_username
    );
}

#[tokio::test]
async fn test_verify_email_success() {
    let app = common::spawn_app().await;

    // 1. Create a user
    let unique_username = format!("toverify_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    let register_response = app
        .api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "S3nh@Forte123!"
        }))
        .await;

    register_response.assert_status_ok();

    // 2. Get the user ID
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(&unique_username)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // 3. Check user is not verified
    let is_verified: bool = sqlx::query_scalar("SELECT email_verified FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();
    assert!(!is_verified, "User should not be verified initially");

    // 4. Create a verification token manually
    let verification_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&verification_token);

    sqlx::query(
        r#"
        INSERT INTO email_verification_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + '24 hours'::INTERVAL)
        "#,
    )
    .bind(user_id)
    .bind(&token_hash)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // 5. Verify the email
    let verify_response = app
        .api
        .post("/verify-email")
        .json(&json!({
            "token": verification_token
        }))
        .await;

    verify_response.assert_status_ok();
    let body: Value = verify_response.json();
    assert_eq!(body["verified"], true);

    // 6. Check user is now verified
    let is_verified: bool = sqlx::query_scalar("SELECT email_verified FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();
    assert!(is_verified, "User should be verified after verification");
}

#[tokio::test]
async fn test_verify_email_expired_token() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // Create an expired token
    let verification_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&verification_token);

    sqlx::query(
        r#"
        INSERT INTO email_verification_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() - '1 hour'::INTERVAL)
        "#,
    )
    .bind(user_id)
    .bind(&token_hash)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // Try to verify with expired token
    let verify_response = app
        .api
        .post("/verify-email")
        .json(&json!({
            "token": verification_token
        }))
        .await;

    assert_eq!(verify_response.status_code(), 400);
}

#[tokio::test]
async fn test_verify_email_already_used_token() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // Create a used token
    let verification_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&verification_token);

    sqlx::query(
        r#"
        INSERT INTO email_verification_tokens (user_id, token_hash, expires_at, used)
        VALUES ($1, $2, NOW() + '24 hours'::INTERVAL, TRUE)
        "#,
    )
    .bind(user_id)
    .bind(&token_hash)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // Try to verify with used token
    let verify_response = app
        .api
        .post("/verify-email")
        .json(&json!({
            "token": verification_token
        }))
        .await;

    assert_eq!(verify_response.status_code(), 400);
}

#[tokio::test]
async fn test_resend_verification_success() {
    let app = common::spawn_app().await;

    // 1. Create an unverified user
    let unique_username = format!("resendtest_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    app.api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "S3nh@Forte123!"
        }))
        .await;

    // Clear email queue
    app.email_service.messages.lock().await.clear();

    // 2. Request resend
    let resend_response = app
        .api
        .post("/resend-verification")
        .json(&json!({
            "email": unique_email
        }))
        .await;

    resend_response.assert_status_ok();

    // 3. Check email was sent
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let messages = app.email_service.messages.lock().await;
    assert!(messages.len() >= 1, "Should have sent verification email");
}

#[tokio::test]
async fn test_resend_verification_rate_limit() {
    let app = common::spawn_app().await;

    // 1. Create an unverified user
    let unique_username = format!("ratelimit_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    app.api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "S3nh@Forte123!"
        }))
        .await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(&unique_username)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // 2. Create 3 recent verification requests (to hit rate limit)
    for _ in 0..3 {
        let token_hash = hash_token(&Uuid::new_v4().to_string());
        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, NOW() + '24 hours'::INTERVAL)
            "#,
        )
        .bind(user_id)
        .bind(&token_hash)
        .execute(&app.db_auth)
        .await
        .unwrap();
    }

    // 3. Try to resend - should hit rate limit
    let resend_response = app
        .api
        .post("/resend-verification")
        .json(&json!({
            "email": unique_email
        }))
        .await;

    assert_eq!(resend_response.status_code(), 400);
    assert!(resend_response.text().contains("Limite"));
}

#[tokio::test]
async fn test_resend_verification_nonexistent_email() {
    let app = common::spawn_app().await;

    // Try to resend for nonexistent email
    let resend_response = app
        .api
        .post("/resend-verification")
        .json(&json!({
            "email": "nonexistent@example.com"
        }))
        .await;

    // Should return 200 to prevent email enumeration
    resend_response.assert_status_ok();

    // But no email should be sent
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let messages = app.email_service.messages.lock().await;
    assert_eq!(messages.len(), 0);
}

#[tokio::test]
async fn test_resend_verification_already_verified() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // Mark as verified
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Try to resend
    let resend_response = app
        .api
        .post("/resend-verification")
        .json(&json!({
            "email": email
        }))
        .await;

    resend_response.assert_status_ok();
    assert!(resend_response.text().contains("já está verificado"));
}

#[tokio::test]
async fn test_verification_status_authenticated() {
    let app = common::spawn_app().await;

    // Check status for authenticated user (alice)
    let response = app
        .api
        .get("/verification-status")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert!(body["email_verified"].is_boolean());
}
