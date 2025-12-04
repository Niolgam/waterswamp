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
        username.to_string(), // Usa o username correto
    )
    .unwrap();
    totp.generate_current().unwrap()
}

// Helper to hash backup code
fn hash_backup_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.to_uppercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

// Test secret that's at least 128 bits (16 bytes) - this is 20 bytes when decoded
const TEST_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

#[tokio::test]
async fn test_mfa_setup_initiation() {
    let app = common::spawn_app().await;

    // [FIX] Ensure MFA is disabled for Alice before starting.
    // Since tests run in parallel on a shared DB, we must reset the state.
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Setup MFA for Alice (admin user)
    let response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["secret"].is_string(), "Should return TOTP secret");
    assert!(body["qr_code_url"].is_string(), "Should return QR code URL");
    assert!(body["setup_token"].is_string(), "Should return setup token");

    // Verify secret format (base32)
    let secret = body["secret"].as_str().unwrap();
    assert!(secret.len() > 10, "Secret should be reasonably long");

    // Verify QR code URL format
    let qr_url = body["qr_code_url"].as_str().unwrap();
    assert!(qr_url.starts_with("otpauth://totp/"));
    assert!(qr_url.contains("Waterswamp"));
}

#[tokio::test]
async fn test_mfa_setup_already_enabled() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // Enable MFA for Alice
    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = 'TEST123' WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Try to setup MFA again
    let response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), 400);
    assert!(response.text().contains("já está ativado"));
}

#[tokio::test]
async fn test_mfa_verify_setup_success() {
    let app = common::spawn_app().await;

    // [FIX] Ensure MFA is disabled for Alice before starting
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // 1. Initiate MFA setup
    let setup_response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    setup_response.assert_status_ok();
    let setup_body: Value = setup_response.json();

    let secret = setup_body["secret"].as_str().unwrap();
    let setup_token = setup_body["setup_token"].as_str().unwrap();

    // 2. Generate a valid TOTP code
    let totp_code = generate_totp_code(secret, "alice");

    // 3. Verify setup
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
    assert!(verify_body["backup_codes"].is_array());

    let backup_codes = verify_body["backup_codes"].as_array().unwrap();
    assert_eq!(backup_codes.len(), 10, "Should have 10 backup codes");

    // Verify each backup code format
    for code in backup_codes {
        let code_str = code.as_str().unwrap();
        assert_eq!(code_str.len(), 12, "Backup code should be 12 chars");
    }

    // 4. Verify MFA is enabled in database
    let mfa_enabled: bool = sqlx::query_scalar("SELECT mfa_enabled FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    assert!(mfa_enabled, "MFA should be enabled in database");
}

#[tokio::test]
async fn test_mfa_verify_setup_invalid_code() {
    let app = common::spawn_app().await;

    // [FIX] Ensure MFA is disabled for Alice before starting
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // 1. Initiate MFA setup
    let setup_response = app
        .api
        .post("/auth/mfa/setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    setup_response.assert_status_ok();

    let setup_body: Value = setup_response.json();
    let setup_token = setup_body["setup_token"].as_str().unwrap();

    // 2. Try with invalid TOTP code
    let verify_response = app
        .api
        .post("/auth/mfa/verify-setup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "setup_token": setup_token,
            "totp_code": "000000"  // Invalid code
        }))
        .await;

    assert_eq!(verify_response.status_code(), 400);
    assert!(
        verify_response.text().contains("TOTP inválido")
            || verify_response.text().contains("inválido")
    );
}

#[tokio::test]
async fn test_login_with_mfa_enabled_returns_challenge() {
    let app = common::spawn_app().await;

    // 1. Setup MFA for a user
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, mfa_secret = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // 2. Try to login
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "password123"
        }))
        .await;

    login_response.assert_status_ok();
    let body: Value = login_response.json();

    // Should get MFA challenge, not tokens
    assert_eq!(body["mfa_required"], true);
    assert!(body["mfa_token"].is_string());
    assert!(!body.get("access_token").is_some());
}

#[tokio::test]
async fn test_mfa_verify_with_totp() {
    let app = common::spawn_app().await;

    // 1. Setup MFA for bob
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, mfa_secret = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // 2. Login to get MFA challenge
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "password123"
        }))
        .await;

    let login_body: Value = login_response.json();
    let mfa_token = login_body["mfa_token"].as_str().unwrap();

    // 3. Generate valid TOTP code
    let totp_code = generate_totp_code(TEST_SECRET, "bob");

    // 4. Verify MFA
    let mfa_response = app
        .api
        .post("/auth/mfa/verify")
        .json(&json!({
            "mfa_token": mfa_token,
            "code": totp_code
        }))
        .await;

    mfa_response.assert_status_ok();
    let mfa_body: Value = mfa_response.json();

    assert!(mfa_body["access_token"].is_string());
    assert!(mfa_body["refresh_token"].is_string());
    assert_eq!(mfa_body["backup_code_used"], false);
}

#[tokio::test]
async fn test_mfa_verify_with_backup_code() {
    let app = common::spawn_app().await;

    // 1. Setup MFA for bob with backup codes
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let backup_code_plain = "ABCD1234EFGH";
    let backup_code_hashed = hash_backup_code(backup_code_plain);

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, 
            mfa_secret = $2,
            mfa_backup_codes = ARRAY[$3]
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .bind(&backup_code_hashed)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // 2. Login to get MFA challenge
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "password123"
        }))
        .await;

    let login_body: Value = login_response.json();
    let mfa_token = login_body["mfa_token"].as_str().unwrap();

    // 3. Use backup code instead of TOTP
    let mfa_response = app
        .api
        .post("/auth/mfa/verify")
        .json(&json!({
            "mfa_token": mfa_token,
            "code": backup_code_plain
        }))
        .await;

    mfa_response.assert_status_ok();
    let mfa_body: Value = mfa_response.json();

    assert!(mfa_body["access_token"].is_string());
    assert_eq!(mfa_body["backup_code_used"], true);

    // 4. Verify backup code was removed
    let remaining_codes: Option<Vec<String>> =
        sqlx::query_scalar("SELECT mfa_backup_codes FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&app.db_auth)
            .await
            .unwrap();

    assert!(remaining_codes.is_none() || remaining_codes.unwrap().is_empty());
}

#[tokio::test]
async fn test_mfa_verify_invalid_code() {
    let app = common::spawn_app().await;

    // 1. Setup MFA for bob
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, mfa_secret = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .execute(&app.db_auth)
    .await
    .unwrap();

    // 2. Login to get MFA challenge
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "password123"
        }))
        .await;

    let login_body: Value = login_response.json();
    let mfa_token = login_body["mfa_token"].as_str().unwrap();

    // 3. Try with invalid code
    let mfa_response = app
        .api
        .post("/auth/mfa/verify")
        .json(&json!({
            "mfa_token": mfa_token,
            "code": "000000"
        }))
        .await;

    assert_eq!(mfa_response.status_code(), 401);
}

#[tokio::test]
async fn test_mfa_disable() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, 
            mfa_secret = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .execute(&app.db_auth)
    .await
    .unwrap();

    let totp_code = generate_totp_code(TEST_SECRET, "alice");

    // Disable MFA
    let disable_response = app
        .api
        .post("/auth/mfa/disable")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "password": "password123",
            "totp_code": totp_code
        }))
        .await;

    disable_response.assert_status_ok();
    let body: Value = disable_response.json();
    assert_eq!(body["disabled"], true);

    // Verify MFA is disabled in database
    let mfa_enabled: bool = sqlx::query_scalar("SELECT mfa_enabled FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    assert!(!mfa_enabled);
}

#[tokio::test]
async fn test_mfa_disable_wrong_password() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, 
            mfa_secret = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .execute(&app.db_auth)
    .await
    .unwrap();

    let totp_code = generate_totp_code(TEST_SECRET, "alice");

    // Try to disable with wrong password
    let disable_response = app
        .api
        .post("/auth/mfa/disable")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "password": "wrongpassword",
            "totp_code": totp_code
        }))
        .await;

    assert_eq!(disable_response.status_code(), 401);
}

#[tokio::test]
async fn test_mfa_regenerate_backup_codes() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let old_backup = hash_backup_code("OLDCODE12345");

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, 
            mfa_secret = $2,
            mfa_backup_codes = ARRAY[$3]
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .bind(&old_backup)
    .execute(&app.db_auth)
    .await
    .unwrap();

    let totp_code = generate_totp_code(TEST_SECRET, "alice");

    // Regenerate backup codes
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

    let new_codes = body["backup_codes"].as_array().unwrap();
    assert_eq!(new_codes.len(), 10);

    // Verify old code is no longer valid
    let db_codes: Option<Vec<String>> =
        sqlx::query_scalar("SELECT mfa_backup_codes FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&app.db_auth)
            .await
            .unwrap();

    let db_codes = db_codes.unwrap();
    assert!(!db_codes.contains(&old_backup));
}

#[tokio::test]
async fn test_mfa_status() {
    let app = common::spawn_app().await;

    // Check status when MFA is disabled
    let status_response = app
        .api
        .get("/auth/mfa/status")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    status_response.assert_status_ok();
    let body: Value = status_response.json();

    assert!(body["enabled"].is_boolean());

    // If disabled, backup_codes_remaining should be null
    if !body["enabled"].as_bool().unwrap() {
        assert!(body["backup_codes_remaining"].is_null());
    }
}

#[tokio::test]
async fn test_mfa_status_with_backup_codes() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let codes = vec![
        hash_backup_code("CODE1"),
        hash_backup_code("CODE2"),
        hash_backup_code("CODE3"),
    ];

    sqlx::query(
        r#"
        UPDATE users 
        SET mfa_enabled = TRUE, 
            mfa_secret = $2,
            mfa_backup_codes = $3
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(TEST_SECRET)
    .bind(&codes)
    .execute(&app.db_auth)
    .await
    .unwrap();

    let status_response = app
        .api
        .get("/auth/mfa/status")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    status_response.assert_status_ok();
    let body: Value = status_response.json();

    assert_eq!(body["enabled"], true);
    assert_eq!(body["backup_codes_remaining"], 3);
}
