// apps/api-server/tests/password_reset_tests.rs

// ⭐ CORREÇÃO: Remover import não utilizado
// use axum_test::TestServer;
use domain::models::{
    Claims, ForgotPasswordPayload, LoginPayload, ResetPasswordPayload, TokenType,
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
// ⭐ CORREÇÃO: Erro de digitação 'UNIX_EPOCH'
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Importa os helpers do nosso common.rs
mod common;

/// Helper específico para este teste: Gera um token JWT
/// com um tipo e expiração personalizados.
fn generate_custom_token(user_id: Uuid, token_type: TokenType, expires_in_seconds: i64) -> String {
    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let encoding_key = EncodingKey::from_ed_pem(private_pem).unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH) // <-- ⭐ CORREÇÃO: Usar o import correto
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
        exp: now + expires_in_seconds,
        iat: now,
        token_type,
    };

    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &encoding_key).unwrap()
}

// Helper para gerar um token de reset padrão (válido por 15 min)
fn generate_reset_token(user_id: Uuid) -> String {
    generate_custom_token(user_id, TokenType::PasswordReset, 900) // 15 minutos
}

#[tokio::test]
async fn test_forgot_password_flow_and_email_mocking() {
    let app = common::spawn_app().await;
    let client = &app.api;

    // "bob" existe no seed, o email dele é "bob@example.com"
    let payload = ForgotPasswordPayload {
        email: "bob@example.com".to_string(),
    };

    // 1. Cenário: Email Existe
    let res = client.post("/auth/forgot-password").json(&payload).await;

    assert_eq!(res.status_code(), 200); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res
        .text() // <-- ⭐ CORREÇÃO: .await removido
        .contains("Se este email estiver registado"));

    // Verifica se o MockEmailService "enviou" o email
    let messages = app.email_service.messages.lock().await;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].to, "bob@example.com");
    assert_eq!(messages[0].subject, "Redefina sua senha do Waterswamp");
    assert_eq!(messages[0].template, "reset_password.html");
    assert_eq!(messages[0].context.get("username").unwrap(), "bob");

    // Limpa a caixa de entrada
    drop(messages); // Libera o lock
    app.email_service.messages.lock().await.clear();

    // 2. Cenário: Email Não Existe
    let payload_nao_existe = ForgotPasswordPayload {
        email: "naoexiste@example.com".to_string(),
    };
    let res_nao_existe = client
        .post("/auth/forgot-password")
        .json(&payload_nao_existe)
        .await;

    // Assert: Deve retornar a MESMA resposta de sucesso para evitar enumeração
    assert_eq!(res_nao_existe.status_code(), 200); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res_nao_existe
        .text() // <-- ⭐ CORREÇÃO: .await removido
        .contains("Se este email estiver registado"));

    // Assert: Nenhum email novo foi enviado
    let messages_final = app.email_service.messages.lock().await;
    assert_eq!(messages_final.len(), 0);
}

#[tokio::test]
async fn test_reset_password_happy_path_and_session_revocation() {
    let app = common::spawn_app().await;
    let client = &app.api;

    // 1. Bob (user_id 'bob_id') existe com senha "password123"
    // Vamos primeiro fazer login para obter um refresh token
    let login_payload = LoginPayload {
        username: "bob".to_string(),
        password: "password123".to_string(),
    };
    let res_login = client.post("/auth/login").json(&login_payload).await;
    assert_eq!(res_login.status_code(), 200);
    let original_tokens = res_login.json::<serde_json::Value>();

    let original_refresh_token = original_tokens["refresh_token"].as_str().unwrap();

    // 2. Bob esquece a senha. Geramos um token de reset para ele
    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let reset_token = generate_reset_token(bob_id);

    // 3. Bob redefine a senha
    let reset_payload = ResetPasswordPayload {
        token: reset_token,
        new_password: "nova_senha_segura_456".to_string(),
    };
    let res_reset = client
        .post("/auth/reset-password")
        .json(&reset_payload)
        .await;

    assert_eq!(res_reset.status_code(), 200); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res_reset.text().contains("Senha atualizada")); // <-- ⭐ CORREÇÃO: .await removido

    // 4. VERIFICAÇÃO: Login com senha antiga falha
    let res_login_antigo = client.post("/auth/login").json(&login_payload).await;
    assert_eq!(res_login_antigo.status_code(), 401); // <-- ⭐ CORREÇÃO: .status_code()

    // 5. VERIFICAÇÃO: Login com nova senha funciona
    let login_payload_novo = LoginPayload {
        username: "bob".to_string(),
        password: "nova_senha_segura_456".to_string(),
    };
    let res_login_novo = client.post("/auth/login").json(&login_payload_novo).await;
    assert_eq!(res_login_novo.status_code(), 200); // <-- ⭐ CORREÇÃO: .status_code()

    // 6. VERIFICAÇÃO: Sessão antiga (refresh token) foi revogada
    let res_refresh = client
        .post("/auth/refresh-token")
        .json(&json!({ "refresh_token": original_refresh_token }))
        .await;

    assert_eq!(res_refresh.status_code(), 401); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res_refresh.text().contains("inválido")); // <-- ⭐ CORREÇÃO: .await removido
}

#[tokio::test]
async fn test_reset_password_invalid_tokens() {
    let app = common::spawn_app().await;
    let client = &app.api;

    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    // 1. Cenário: Token Expirado
    let expired_token = generate_custom_token(bob_id, TokenType::PasswordReset, -60); // Expirou há 60s
    let reset_payload_expirado = ResetPasswordPayload {
        token: expired_token,
        new_password: "password_qualquer_123".to_string(),
    };
    let res_expirado = client
        .post("/auth/reset-password")
        .json(&reset_payload_expirado)
        .await;

    assert_eq!(res_expirado.status_code(), 401); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res_expirado.text().contains("expirado")); // <-- ⭐ CORREÇÃO: .await removido

    // 2. Cenário: Token de Tipo Errado (usando um Access Token)
    let access_token = common::generate_test_token(bob_id); // Pega o token do bob
    let reset_payload_tipo_errado = ResetPasswordPayload {
        token: access_token, // <--- Token de acesso
        new_password: "password_qualquer_123".to_string(),
    };
    let res_tipo_errado = client
        .post("/auth/reset-password")
        .json(&reset_payload_tipo_errado)
        .await;

    assert_eq!(res_tipo_errado.status_code(), 401); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res_tipo_errado.text().contains("tipo incorreto")); // <-- ⭐ CORREÇÃO: .await removido
}

#[tokio::test]
async fn test_reset_password_weak_password() {
    let app = common::spawn_app().await;
    let client = &app.api;

    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    let reset_token = generate_reset_token(bob_id);

    let reset_payload = ResetPasswordPayload {
        token: reset_token,
        new_password: "123".to_string(), // Senha fraca
    };
    let res_reset = client
        .post("/auth/reset-password")
        .json(&reset_payload)
        .await;

    assert_eq!(res_reset.status_code(), 400); // <-- ⭐ CORREÇÃO: .status_code()
    assert!(res_reset.text().contains("A senha deve ter")); // <-- ⭐ CORREÇÃO: .await removido
}
