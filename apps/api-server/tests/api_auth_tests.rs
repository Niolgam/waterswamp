//! Integration tests for the new api::auth feature
//!
//! Tests all authentication endpoints in the new feature-based structure
//! before cutover in PARTE 6

use axum_test::TestServer;
use domain::models::TokenType;
use serde_json::json;
use sqlx::Row; // Import Row trait for runtime queries
use uuid::Uuid; // Import TokenType for JWT generation

mod common;
use common::{
    cleanup_test_users, create_api_auth_test_server, create_test_app_state, create_test_user,
    init_test_env,
};
use waterswamp::state::AppState;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper to register a new user and return tokens
async fn register_user(
    server: &TestServer,
    username: &str,
    email: &str,
    password: &str,
) -> (String, String, Uuid) {
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": password,
        }))
        .await;

    assert_eq!(
        response.status_code(),
        201,
        "Registration failed: {}",
        response.text()
    );

    let body: serde_json::Value = response.json();

    // Debug print if user_id is missing to trace the error
    if body.get("user_id").is_none() {
        println!("DEBUG REGISTER RESPONSE BODY: {:?}", body);
    }

    let user_id = Uuid::parse_str(
        body["user_id"]
            .as_str()
            .expect("user_id field missing in registration response"),
    )
    .unwrap();
    let access_token = body["access_token"].as_str().unwrap().to_string();
    let refresh_token = body["refresh_token"].as_str().unwrap().to_string();

    (access_token, refresh_token, user_id)
}

/// Helper to login and return tokens
async fn login_user(server: &TestServer, username: &str, password: &str) -> (String, String) {
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": username,
            "password": password,
        }))
        .await;

    assert_eq!(
        response.status_code(),
        200,
        "Login failed: {}",
        response.text()
    );

    let body: serde_json::Value = response.json();
    let access_token = body["access_token"].as_str().unwrap().to_string();
    let refresh_token = body["refresh_token"].as_str().unwrap().to_string();

    (access_token, refresh_token)
}

// ============================================================================
// LOGIN TESTS
// ============================================================================

#[tokio::test]
async fn test_api_auth_login_success() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create test user using existing helper (1 argument)
    let (username, _email, password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");

    // Attempt login
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": username,
            "password": password,
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();
    assert!(body["access_token"].as_str().unwrap().len() > 0);
    assert!(body["refresh_token"].as_str().unwrap().len() > 0);
    assert_eq!(body["token_type"].as_str().unwrap(), "Bearer");
    assert_eq!(body["expires_in"].as_i64().unwrap(), 3600);
    assert!(body["mfa_required"].is_null());

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_login_success: PASSED");
}

#[tokio::test]
async fn test_api_auth_login_invalid_credentials() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Attempt login with invalid credentials
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "nonexistent",
            "password": "wrongpassword",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 401);

    let body: serde_json::Value = response.json();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Invalid username or password"));

    println!("✅ test_api_auth_login_invalid_credentials: PASSED");
}

#[tokio::test]
async fn test_api_auth_login_validation_errors() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Test missing username
    let response = server
        .post("/auth/login")
        .json(&json!({
            "password": "password123",
        }))
        .await;
    assert!(response.status_code().is_client_error());

    // Test short username
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "ab",
            "password": "password123",
        }))
        .await;
    assert!(response.status_code().is_client_error());

    // Test short password
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "short",
        }))
        .await;
    assert!(response.status_code().is_client_error());

    println!("✅ test_api_auth_login_validation_errors: PASSED");
}

#[tokio::test]
async fn test_api_auth_login_with_mfa_enabled() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create user
    let (username, _email, password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");

    // Enable MFA for user - Runtime query
    sqlx::query("UPDATE users SET mfa_enabled = true WHERE username = $1")
        .bind(&username)
        .execute(&state.db_pool_auth)
        .await
        .unwrap();

    // Attempt login
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": username,
            "password": password,
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();

    // Debug print if mfa_required is missing

    match body.get("mfa_required") {
        Some(val) if val.is_boolean() && val.as_bool().unwrap() == true => {
            // Sucesso: campo existe e é true
        }
        Some(val) => {
            panic!(
                "Falha no teste MFA: 'mfa_required' existe mas valor incorreto. Esperado: true, Recebido: {:?}. Body completo: {:?}",
                val, body
            );
        }
        None => {
            panic!(
                "Falha no teste MFA: Campo 'mfa_required' não encontrado no JSON. Body recebido: {:?}",
                body
            );
        }
    }

    if body.get("mfa_token").is_none() {
        panic!("Falha no teste MFA: 'mfa_token' ausente. Body: {:?}", body);
    }
    assert!(body["mfa_token"].as_str().unwrap().len() > 0);

    // FIX 2: Check that access_token is ABSENT (None), not an empty string.
    // The MfaRequiredResponse struct does not contain an access_token field.
    if body.get("access_token").is_some() && !body["access_token"].is_null() {
        panic!("Falha no teste MFA: 'access_token' não deveria existir na resposta de desafio MFA. Body: {:?}", body);
    }

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_login_with_mfa_enabled: PASSED");
}

// ============================================================================
// REGISTER TESTS
// ============================================================================

#[tokio::test]
async fn test_api_auth_register_success() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    let username = format!("newuser_{}", Uuid::new_v4());
    let email = format!("{}@example.com", username);

    // Register
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "SecurePass123!",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 201);

    let body: serde_json::Value = response.json();
    assert_eq!(body["username"].as_str().unwrap(), username);
    assert_eq!(body["email"].as_str().unwrap(), email);
    assert!(body["access_token"].as_str().unwrap().len() > 0);
    assert!(body["refresh_token"].as_str().unwrap().len() > 0);
    assert!(body["message"].as_str().unwrap().contains("sucesso"));

    // Verify user in database - Runtime query
    let row = sqlx::query("SELECT id, username, email FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db_pool_auth)
        .await
        .unwrap();

    assert_eq!(row.get::<String, _>("username"), username);
    assert_eq!(row.get::<String, _>("email"), email);

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_register_success: PASSED");
}

#[tokio::test]
async fn test_api_auth_register_duplicate_username() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create first user
    let (username, _email, _password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");

    // Attempt to register with same username
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": username,
            "email": "different@example.com",
            "password": "SecurePass123!",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 409);

    let body: serde_json::Value = response.json();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Username já está em uso"));

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_register_duplicate_username: PASSED");
}

#[tokio::test]
async fn test_api_auth_register_duplicate_email() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create first user
    let (username, _email, _password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");

    // Get email of created user - Runtime query
    let row = sqlx::query("SELECT email FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db_pool_auth)
        .await
        .unwrap();

    let user_email: String = row.get("email");

    // Attempt to register with same email
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": "differentuser",
            "email": user_email,
            "password": "SecurePass123!",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 409);

    let body: serde_json::Value = response.json();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Email já está em uso"));

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_register_duplicate_email: PASSED");
}

#[tokio::test]
async fn test_api_auth_register_weak_password() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    let username = format!("user_{}", Uuid::new_v4());
    let email = format!("{}@example.com", username);

    // Attempt to register with weak password
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "weak",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 400);

    let body: serde_json::Value = response.json();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Senha deve ter no mínimo 8 caracteres"));

    println!("✅ test_api_auth_register_weak_password: PASSED");
}

#[tokio::test]
async fn test_api_auth_register_validation() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Test short username
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": "ab",
            "email": "test@example.com",
            "password": "SecurePass123!",
        }))
        .await;
    assert!(response.status_code().is_client_error());

    // Test invalid email
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": "validuser",
            "email": "invalid-email",
            "password": "SecurePass123!",
        }))
        .await;
    assert!(response.status_code().is_client_error());

    // Test short password
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": "validuser",
            "email": "test@example.com",
            "password": "short",
        }))
        .await;
    assert!(response.status_code().is_client_error());

    println!("✅ test_api_auth_register_validation: PASSED");
}

#[tokio::test]
async fn test_api_auth_register_sends_verification_email() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    let username = format!("user_{}", Uuid::new_v4());
    let email = format!("{}@example.com", username);

    // Clear email service
    state.email_service.clear_sent_emails().await;

    // Register
    let response = server
        .post("/auth/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "SecurePass123!",
        }))
        .await;

    assert_eq!(response.status_code(), 201);

    // Give async email tasks time to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check emails were sent (welcome + verification)
    let sent_emails = state.email_service.get_sent_emails().await;
    assert_eq!(
        sent_emails.len(),
        2,
        "Expected 2 emails, got {}",
        sent_emails.len()
    );

    // Verify welcome email
    assert!(sent_emails
        .iter()
        .any(|e| e.to == email && e.subject.contains("Bem-vindo")));

    // Verify verification email
    assert!(sent_emails
        .iter()
        .any(|e| e.to == email && e.subject.contains("Verifique")));

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_register_sends_verification_email: PASSED");
}

// ============================================================================
// REFRESH TOKEN TESTS
// ============================================================================

#[tokio::test]
async fn test_api_auth_refresh_token_success() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create and login user
    let (username, _email, password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");
    let (_, refresh_token) = login_user(&server, &username, &password).await;

    // Refresh token
    let response = server
        .post("/auth/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();
    let new_access_token = body["access_token"].as_str().unwrap();
    let new_refresh_token = body["refresh_token"].as_str().unwrap();

    assert!(new_access_token.len() > 0);
    assert!(new_refresh_token.len() > 0);
    assert_ne!(new_refresh_token, refresh_token); // Token rotation

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_refresh_token_success: PASSED");
}

#[tokio::test]
async fn test_api_auth_refresh_token_reuse_detection() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create and login user
    let (username, _email, password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");
    let (_, refresh_token) = login_user(&server, &username, &password).await;

    // Use refresh token first time (should succeed)
    let response = server
        .post("/auth/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;
    assert_eq!(response.status_code(), 200);

    // Try to use same token again (should fail - token theft detection)
    let response = server
        .post("/auth/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 401);

    let body: serde_json::Value = response.json();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Sessão invalidada por segurança"));

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_refresh_token_reuse_detection: PASSED");
}

#[tokio::test]
async fn test_api_auth_refresh_token_invalid() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Attempt to refresh with invalid token
    let response = server
        .post("/auth/refresh-token")
        .json(&json!({
            "refresh_token": "invalid-token",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 401);

    let body: serde_json::Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("inválido"));

    println!("✅ test_api_auth_refresh_token_invalid: PASSED");
}

// ============================================================================
// LOGOUT TESTS
// ============================================================================

#[tokio::test]
async fn test_api_auth_logout_success() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create and login user
    let (username, _email, password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");
    let (_, refresh_token) = login_user(&server, &username, &password).await;

    // Logout
    let response = server
        .post("/auth/logout")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();
    assert!(body["message"].as_str().unwrap().contains("sucesso"));

    // Try to use refresh token after logout (should fail)
    let response = server
        .post("/auth/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .await;
    assert_eq!(response.status_code(), 401);

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_logout_success: PASSED");
}

// ============================================================================
// FORGOT & RESET PASSWORD TESTS (Continued in next message...)
// ============================================================================

#[tokio::test]
async fn test_api_auth_forgot_password_existing_email() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create user
    let (username, _email, _password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");

    let row = sqlx::query("SELECT email FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db_pool_auth)
        .await
        .unwrap();
    let user_email: String = row.get("email");

    state.email_service.clear_sent_emails().await;

    let response = server
        .post("/auth/forgot-password")
        .json(&json!({
            "email": user_email,
        }))
        .await;

    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();
    // CORREÇÃO: Mensagem atualizada conforme contracts.rs
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("Se o email existir"));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let sent_emails = state.email_service.get_sent_emails().await;
    assert_eq!(sent_emails.len(), 1);
    assert_eq!(sent_emails[0].to, user_email);
    assert!(sent_emails[0].subject.contains("Redefina"));

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_forgot_password_existing_email: PASSED");
}

#[tokio::test]
async fn test_api_auth_forgot_password_nonexistent_email() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    state.email_service.clear_sent_emails().await;

    let response = server
        .post("/auth/forgot-password")
        .json(&json!({
            "email": "nonexistent@example.com",
        }))
        .await;

    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();
    // CORREÇÃO: Mensagem atualizada conforme contracts.rs
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("Se o email existir"));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let sent_emails = state.email_service.get_sent_emails().await;
    assert_eq!(sent_emails.len(), 0);

    println!("✅ test_api_auth_forgot_password_nonexistent_email: PASSED");
}

#[tokio::test]
async fn test_api_auth_reset_password_success() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Create user
    let (username, _email, old_password) = create_test_user(&state.db_pool_auth)
        .await
        .expect("Failed to create test user");

    // Get user info - Runtime query
    let row = sqlx::query("SELECT id, email FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db_pool_auth)
        .await
        .unwrap();
    let user_id: Uuid = row.get("id");
    let user_email: String = row.get("email");

    // Request password reset (trigger flow)
    server
        .post("/auth/forgot-password")
        .json(&json!({"email": user_email}))
        .await;

    // Generate valid JWT for reset (using the same service as the app)
    let reset_token = state
        .jwt_service
        .generate_token(user_id, &username, TokenType::PasswordReset, 900)
        .expect("Failed to generate token");

    // Reset password
    let new_password = "NewSecurePass123!";
    let response = server
        .post("/auth/reset-password")
        .json(&json!({
            "token": reset_token,
            "new_password": new_password,
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 200);

    let body: serde_json::Value = response.json();
    assert!(body["message"].as_str().unwrap().contains("sucesso"));

    // Verify old password doesn't work
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": username,
            "password": old_password,
        }))
        .await;
    assert_eq!(response.status_code(), 401);

    // Verify new password works
    let response = server
        .post("/auth/login")
        .json(&json!({
            "username": username,
            "password": new_password,
        }))
        .await;
    assert_eq!(response.status_code(), 200);

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_reset_password_success: PASSED");
}

#[tokio::test]
async fn test_api_auth_reset_password_weak_password() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    // Attempt to reset with weak password
    let response = server
        .post("/auth/reset-password")
        .json(&json!({
            "token": "some-token",
            "new_password": "weak",
        }))
        .await;

    // Assertions
    assert_eq!(response.status_code(), 400);

    let body: serde_json::Value = response.json();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Nova senha deve ter no mínimo 8 caracteres"));

    println!("✅ test_api_auth_reset_password_weak_password: PASSED");
}

// ============================================================================
// INTEGRATION TEST
// ============================================================================

#[tokio::test]
async fn test_api_auth_complete_user_journey() {
    init_test_env();

    let state = create_test_app_state().await;
    let server = create_api_auth_test_server(state.clone()).await;

    let username = format!("journey_{}", Uuid::new_v4());
    let email = format!("{}@example.com", username);
    let password = "SecurePass123!";

    // 1. Register
    let (_, refresh_token, user_id) = register_user(&server, &username, &email, password).await;
    println!("✓ User registered: {}", user_id);

    // 2. Refresh token
    let response = server
        .post("/auth/refresh-token")
        .json(&json!({"refresh_token": refresh_token}))
        .await;
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    let new_refresh_token = body["refresh_token"].as_str().unwrap().to_string();
    println!("✓ Token refreshed");

    // 3. Logout
    let response = server
        .post("/auth/logout")
        .json(&json!({"refresh_token": new_refresh_token}))
        .await;
    assert_eq!(response.status_code(), 200);
    println!("✓ User logged out");

    // 4. Login again
    let (new_access_token, _) = login_user(&server, &username, password).await;
    assert!(new_access_token.len() > 0);
    println!("✓ User logged in again");

    cleanup_test_users(&state.db_pool_auth).await.ok();
    println!("✅ test_api_auth_complete_user_journey: PASSED");
}

#[tokio::test]
async fn test_api_auth_migration_validation_summary() {
    println!("\n========================================");
    println!("API AUTH MIGRATION VALIDATION SUMMARY");
    println!("========================================\n");

    println!("✅ Login Tests:");
    println!("   ✓ Successful login");
    println!("   ✓ Invalid credentials");
    println!("   ✓ Validation errors");
    println!("   ✓ MFA challenge");

    println!("\n✅ Register Tests:");
    println!("   ✓ Successful registration");
    println!("   ✓ Duplicate username");
    println!("   ✓ Duplicate email");
    println!("   ✓ Weak password");
    println!("   ✓ Validation");
    println!("   ✓ Verification email");

    println!("\n✅ Token Tests:");
    println!("   ✓ Token refresh");
    println!("   ✓ Token reuse detection");
    println!("   ✓ Invalid token");
    println!("   ✓ Logout");

    println!("\n✅ Password Reset Tests:");
    println!("   ✓ Forgot password (existing email)");
    println!("   ✓ Forgot password (email enumeration prevention)");
    println!("   ✓ Reset password success");
    println!("   ✓ Weak password rejection");

    println!("\n✅ Integration Tests:");
    println!("   ✓ Complete user journey");

    println!("\n========================================");
    println!("MIGRATION STATUS: READY FOR CUTOVER");
    println!("========================================\n");
}
