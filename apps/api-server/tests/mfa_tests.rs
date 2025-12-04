use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use uuid::Uuid;

mod common;

fn generate_totp_code(secret_base32: &str, username: &str) -> String {
    let secret_bytes = Secret::Encoded(secret_base32.to_string())
        .to_bytes()
        .unwrap();
    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waterswamp".to_string()),
        username.to_string(),
    )
    .unwrap();
    totp.generate_current().unwrap()
}

fn hash_backup_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.to_uppercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

const TEST_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

#[tokio::test]
async fn test_mfa_setup_initiation() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();
    let body: Value = response.json();

    let qr_url = body["qr_code_url"].as_str().unwrap();
    // Adjusted expectation to match implementation (Data URI)
    assert!(qr_url.starts_with("data:image/png;base64,"));
}

#[tokio::test]
async fn test_mfa_setup_already_enabled() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = 'TEST123' WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), 400);
}

#[tokio::test]
async fn test_mfa_verify_setup_success() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // 1. Initiate
    let setup_response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let setup_body: Value = setup_response.json();
    let secret = setup_body["secret"].as_str().unwrap();
    let setup_token = setup_body["setup_token"].as_str().unwrap();

    // 2. Generate code
    let totp_code = generate_totp_code(secret, "alice");

    // 3. Verify
    let verify_response = app
        .api
        .post("/auth/mfa/verify-setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "setup_token": setup_token,
            "totp_code": totp_code
        }))
        .await;

    verify_response.assert_status_ok();
    let verify_body: Value = verify_response.json();
    assert_eq!(verify_body["enabled"], true);
    assert_eq!(verify_body["backup_codes"].as_array().unwrap().len(), 10);
}

#[tokio::test]
async fn test_mfa_verify_setup_invalid_code() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let setup_response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let setup_body: Value = setup_response.json();
    let setup_token = setup_body["setup_token"].as_str().unwrap();

    let verify_response = app
        .api
        .post("/auth/mfa/verify-setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "setup_token": setup_token,
            "totp_code": "000000"
        }))
        .await;

    // Updated handler now returns 400 for bad requests (invalid input/logic)
    assert_eq!(verify_response.status_code(), 400);
}

#[tokio::test]
async fn test_mfa_verify_with_totp() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = $2 WHERE id = $1")
        .bind(user_id)
        .bind(TEST_SECRET)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let login_response = app
        .api
        .post("/login")
        .json(&json!({ "username": "bob", "password": "password123" }))
        .await;
    let login_body: Value = login_response.json();
    let mfa_token = login_body["mfa_token"].as_str().unwrap();

    let totp_code = generate_totp_code(TEST_SECRET, "bob");

    let mfa_response = app
        .api
        .post("/auth/mfa/verify")
        .json(&json!({ "mfa_token": mfa_token, "code": totp_code }))
        .await;

    mfa_response.assert_status_ok();
}

#[tokio::test]
async fn test_mfa_verify_with_backup_code() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let backup_code_plain = "ABCD1234EFGH";
    let backup_code_hashed = hash_backup_code(backup_code_plain);

    // Setup MFA secret
    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = $2 WHERE id = $1")
        .bind(user_id)
        .bind(TEST_SECRET)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Insert backup code into CORRECT table
    sqlx::query("DELETE FROM mfa_backup_codes WHERE user_id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();
    sqlx::query("INSERT INTO mfa_backup_codes (user_id, code_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(&backup_code_hashed)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let login_response = app
        .api
        .post("/login")
        .json(&json!({ "username": "bob", "password": "password123" }))
        .await;
    let mfa_token = login_response.json::<Value>()["mfa_token"]
        .as_str()
        .unwrap()
        .to_string();

    let mfa_response = app
        .api
        .post("/auth/mfa/verify")
        .json(&json!({ "mfa_token": mfa_token, "code": backup_code_plain }))
        .await;

    mfa_response.assert_status_ok();
    let mfa_body: Value = mfa_response.json();
    assert_eq!(mfa_body["backup_code_used"], true);
}

#[tokio::test]
async fn test_mfa_status_with_backup_codes() {
    let app = common::spawn_app().await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = $2 WHERE id = $1")
        .bind(user_id)
        .bind(TEST_SECRET)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Insert codes
    sqlx::query("DELETE FROM mfa_backup_codes WHERE user_id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();
    for i in 1..=3 {
        sqlx::query("INSERT INTO mfa_backup_codes (user_id, code_hash) VALUES ($1, $2)")
            .bind(user_id)
            .bind(hash_backup_code(&format!("CODE{}", i)))
            .execute(&app.db_auth)
            .await
            .unwrap();
    }

    let status_response = app
        .api
        .get("/auth/mfa/status")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    status_response.assert_status_ok();
    let body: Value = status_response.json();
    assert_eq!(body["backup_codes_remaining"], 3);
}

#[tokio::test]
async fn test_mfa_regenerate_backup_codes() {
    let app = common::spawn_app().await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = $2 WHERE id = $1")
        .bind(user_id)
        .bind(TEST_SECRET)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let totp_code = generate_totp_code(TEST_SECRET, "alice");

    let regen_response = app
        .api
        .post("/auth/mfa/regenerate-backup-codes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "password": "password123",
            "totp_code": totp_code
        }))
        .await;

    regen_response.assert_status_ok();
    let body: Value = regen_response.json();
    assert_eq!(body["backup_codes"].as_array().unwrap().len(), 10);
}
