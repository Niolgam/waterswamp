//! Integration Tests for api::auth module
//!
//! Testes de integração para a feature de autenticação.
//! Estes testes rodam em paralelo aos testes existentes em api_auth_tests.rs

mod common;

use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// =============================================================================
// LOGIN TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_login_success() {
    let app = common::spawn_app().await;

    // Garantir que MFA está desabilitado para alice
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
        .post("/login")
        .json(&json!({
            "username": "alice",
            "password": "password123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert_eq!(body["token_type"], "Bearer");
    assert!(body["expires_in"].is_number());
}

#[tokio::test]
async fn test_api_auth_login_invalid_password() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "alice",
            "password": "wrongpassword"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_auth_login_nonexistent_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "nonexistent_user_12345",
            "password": "anypassword"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_auth_login_validation_error() {
    let app = common::spawn_app().await;

    // Username muito curto
    let response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "ab",
            "password": "password123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    // Senha muito curta
    let response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "alice",
            "password": "12345"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_auth_login_with_email() {
    let app = common::spawn_app().await;

    // Garantir que MFA está desabilitado para bob
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Login usando email em vez de username
    let response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob@temp.example.com",
            "password": "password123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["access_token"].is_string());
}

// =============================================================================
// REGISTER TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_register_success() {
    let app = common::spawn_app().await;

    let unique_username = format!("newuser_{}", Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "SecureP@ssw0rd!123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert!(body["message"].as_str().unwrap().contains("Verifique"));

    // Verificar que emails foram enviados
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let messages = app.email_service.messages.lock().await;
    assert!(messages.len() >= 2, "Deveria ter enviado 2 emails");
}

#[tokio::test]
async fn test_api_auth_register_username_conflict() {
    let app = common::spawn_app().await;

    // Tentar registrar com username existente
    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": "alice",
            "email": "newemail@example.com",
            "password": "SecureP@ssw0rd!123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_api_auth_register_email_conflict() {
    let app = common::spawn_app().await;

    // Tentar registrar com email existente
    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": "newuser_unique",
            "email": "alice@temp.example.com",
            "password": "SecureP@ssw0rd!123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_api_auth_register_weak_password() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": format!("user_{}", Uuid::new_v4()),
            "email": format!("weak_{}@example.com", Uuid::new_v4()),
            "password": "weak123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_auth_register_invalid_username() {
    let app = common::spawn_app().await;

    // Username com caracteres inválidos
    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": "user@invalid!",
            "email": "valid@example.com",
            "password": "SecureP@ssw0rd!123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// =============================================================================
// REFRESH TOKEN TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_refresh_token_success() {
    let app = common::spawn_app().await;

    // Criar usuário e fazer login
    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Falha ao criar usuário de teste");

    // Desabilitar MFA
    sqlx::query("UPDATE users SET mfa_enabled = FALSE WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Usar refresh token
    let refresh_response = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(refresh_response.status_code(), StatusCode::OK);

    let body: Value = refresh_response.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());

    // Token deve ser diferente (rotação)
    assert_ne!(body["refresh_token"].as_str().unwrap(), refresh_token);
}

#[tokio::test]
async fn test_api_auth_refresh_token_invalid() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": "invalid-token-12345"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_auth_refresh_token_reuse_detection() {
    let app = common::spawn_app().await;

    // Criar usuário e fazer login
    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Falha ao criar usuário");

    sqlx::query("UPDATE users SET mfa_enabled = FALSE WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let original_refresh = login_body["refresh_token"].as_str().unwrap().to_string();

    // Primeiro uso - deve funcionar
    let first_refresh = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": original_refresh
        }))
        .await;

    assert_eq!(first_refresh.status_code(), StatusCode::OK);

    // Segundo uso do mesmo token - deve falhar (detecção de roubo)
    let second_refresh = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": original_refresh
        }))
        .await;

    assert_eq!(second_refresh.status_code(), StatusCode::UNAUTHORIZED);
}

// =============================================================================
// LOGOUT TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_logout_success() {
    let app = common::spawn_app().await;

    // Criar usuário e fazer login
    let (username, _email, password) = common::create_test_user(&app.db_auth)
        .await
        .expect("Falha ao criar usuário");

    sqlx::query("UPDATE users SET mfa_enabled = FALSE WHERE username = $1")
        .bind(&username)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Logout
    let logout_response = app
        .api
        .post("/logout")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(logout_response.status_code(), StatusCode::OK);

    // Tentar usar o token após logout - deve falhar
    let refresh_after_logout = app
        .api
        .post("/refresh-token")
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    assert_eq!(refresh_after_logout.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_auth_logout_invalid_token() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/logout")
        .json(&json!({
            "refresh_token": "invalid-token"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// =============================================================================
// FORGOT PASSWORD TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_forgot_password_existing_email() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/forgot-password")
        .json(&json!({
            "email": "bob@temp.example.com"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["message"].as_str().unwrap().contains("email"));

    // Verificar que email foi enviado
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let messages = app.email_service.messages.lock().await;
    assert!(messages.len() >= 1, "Deveria ter enviado email de reset");
}

#[tokio::test]
async fn test_api_auth_forgot_password_nonexistent_email() {
    let app = common::spawn_app().await;

    // Limpar emails anteriores
    app.email_service.messages.lock().await.clear();

    let response = app
        .api
        .post("/forgot-password")
        .json(&json!({
            "email": "nonexistent@example.com"
        }))
        .await;

    // Deve retornar sucesso para evitar enumeração
    assert_eq!(response.status_code(), StatusCode::OK);

    // Mas não deve enviar email
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let messages = app.email_service.messages.lock().await;
    assert_eq!(messages.len(), 0, "Não deveria enviar email");
}

#[tokio::test]
async fn test_api_auth_forgot_password_invalid_email() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/forgot-password")
        .json(&json!({
            "email": "not-an-email"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// =============================================================================
// RESET PASSWORD TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_reset_password_success() {
    let app = common::spawn_app().await;

    // Buscar ID do bob
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // Desabilitar MFA
    sqlx::query("UPDATE users SET mfa_enabled = FALSE WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // Gerar token de reset válido
    let reset_token = common::generate_reset_token(user_id);

    let response = app
        .api
        .post("/reset-password")
        .json(&json!({
            "token": reset_token,
            "new_password": "NewSecureP@ss123!"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Verificar que login com nova senha funciona
    let login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "NewSecureP@ss123!"
        }))
        .await;

    assert_eq!(login_response.status_code(), StatusCode::OK);

    // Verificar que login com senha antiga falha
    let old_login = app
        .api
        .post("/login")
        .json(&json!({
            "username": "bob",
            "password": "password123"
        }))
        .await;

    assert_eq!(old_login.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_auth_reset_password_invalid_token() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/reset-password")
        .json(&json!({
            "token": "invalid-token",
            "new_password": "NewSecureP@ss123!"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_auth_reset_password_weak_password() {
    let app = common::spawn_app().await;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let reset_token = common::generate_reset_token(user_id);

    let response = app
        .api
        .post("/reset-password")
        .json(&json!({
            "token": reset_token,
            "new_password": "weak"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// =============================================================================
// MFA CHALLENGE TESTS
// =============================================================================

#[tokio::test]
async fn test_api_auth_login_mfa_required() {
    let app = common::spawn_app().await;

    // Habilitar MFA para alice
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = TRUE, mfa_secret = 'GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ' WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    let response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "alice",
            "password": "password123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["mfa_required"], true);
    assert!(body["mfa_token"].is_string());
    assert!(body.get("access_token").is_none());
}
