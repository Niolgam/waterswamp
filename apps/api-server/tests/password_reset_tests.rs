use domain::models::{
    Claims, ForgotPasswordPayload, LoginPayload, ResetPasswordPayload, TokenType,
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
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
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
        username: "testuser".to_string(),
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

    // "bob" existe no seed, o email dele é "bob@temp.example.com"
    let payload = ForgotPasswordPayload {
        email: "bob@temp.example.com".try_into().unwrap(), // Ajuste para Email type
    };

    // 1. Cenário: Email Existe
    let res = client.post("/forgot-password").json(&payload).await;

    assert_eq!(res.status_code(), 200);
    // CORREÇÃO: Mensagem atualizada conforme contracts.rs
    assert!(res.text().contains("Se o email existir"));

    // Verifica se o MockEmailService "enviou" o email
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // [FIX] Removed .await, added .unwrap()
    let messages = app.email_service.messages.lock().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].to, "bob@temp.example.com");
    assert_eq!(messages[0].subject, "Redefina sua senha do Waterswamp");
    assert_eq!(messages[0].template, "reset_password.html");
    assert_eq!(
        messages[0]
            .context
            .get("username")
            .unwrap()
            .as_str()
            .unwrap(),
        "bob"
    );

    // Limpa a caixa de entrada
    drop(messages); // Libera o lock
                    // [FIX] Removed .await, added .unwrap()
    app.email_service.messages.lock().unwrap().clear();

    // 2. Cenário: Email Não Existe
    let payload_nao_existe = ForgotPasswordPayload {
        email: "naoexiste@example.com".try_into().unwrap(),
    };
    let res_nao_existe = client
        .post("/forgot-password")
        .json(&payload_nao_existe)
        .await;

    // Assert: Deve retornar a MESMA resposta de sucesso para evitar enumeração
    assert_eq!(res_nao_existe.status_code(), 200);
    // CORREÇÃO: Mensagem atualizada
    assert!(res_nao_existe.text().contains("Se o email existir"));

    // Assert: Nenhum email novo foi enviado
    // [FIX] Removed .await, added .unwrap()
    let messages_final = app.email_service.messages.lock().unwrap();
    assert_eq!(messages_final.len(), 0);
}

#[tokio::test]
async fn test_reset_password_happy_path_and_session_revocation() {
    let app = common::spawn_app().await;
    let client = &app.api;

    // [FIX] Ensure MFA is disabled for Bob so standard login works
    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&app.db_auth)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET mfa_enabled = FALSE, mfa_secret = NULL WHERE id = $1")
        .bind(bob_id)
        .execute(&app.db_auth)
        .await
        .unwrap();

    // 1. Bob (user_id 'bob_id') existe com senha "password123"
    let login_payload = LoginPayload {
        username: "bob".to_string(),
        password: "password123".to_string(),
    };
    let res_login = client.post("/login").json(&login_payload).await;
    assert_eq!(res_login.status_code(), 200);
    let original_tokens = res_login.json::<serde_json::Value>();

    let original_refresh_token = original_tokens["refresh_token"].as_str().unwrap();

    // 2. Bob esquece a senha. Geramos um token de reset para ele
    let reset_token = generate_reset_token(bob_id);

    // 3. Bob redefine a senha
    let reset_payload = ResetPasswordPayload {
        token: reset_token,
        new_password: "nova_senha_segura_456!A".to_string(),
    };
    let res_reset = client.post("/reset-password").json(&reset_payload).await;

    assert_eq!(res_reset.status_code(), 200);
    // CORREÇÃO: Mensagem atualizada conforme ResetPasswordResponse
    assert!(res_reset.text().contains("Senha redefinida com sucesso"));

    // 4. VERIFICAÇÃO: Login com senha antiga falha
    let res_login_antigo = client.post("/login").json(&login_payload).await;
    assert_eq!(res_login_antigo.status_code(), 401);

    // 5. VERIFICAÇÃO: Login com nova senha funciona
    let login_payload_novo = LoginPayload {
        username: "bob".to_string(),
        password: "nova_senha_segura_456!A".to_string(),
    };
    let res_login_novo = client.post("/login").json(&login_payload_novo).await;
    assert_eq!(res_login_novo.status_code(), 200);

    // 6. VERIFICAÇÃO: Sessão antiga (refresh token) foi revogada
    let res_refresh = client
        .post("/refresh-token")
        .json(&json!({ "refresh_token": original_refresh_token }))
        .await;

    assert_eq!(res_refresh.status_code(), 401);
    let refresh_error_text = res_refresh.text();
    assert!(
        refresh_error_text.contains("inválido")
            || refresh_error_text.contains("invalidada")
            || refresh_error_text.contains("revogado")
            || refresh_error_text.contains("segurança"),
        "Erro inesperado: {}",
        refresh_error_text
    );
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
        new_password: "password_qualquer_123!A".to_string(),
    };
    let res_expirado = client
        .post("/reset-password")
        .json(&reset_payload_expirado)
        .await;

    assert_eq!(res_expirado.status_code(), 401);
    assert!(res_expirado.text().contains("expirado") || res_expirado.text().contains("inválido"));

    // 2. Cenário: Token de Tipo Errado (usando um Access Token)
    let access_token = common::generate_test_token(bob_id); // Pega o token do bob
    let reset_payload_tipo_errado = ResetPasswordPayload {
        token: access_token,
        new_password: "password_qualquer_123!A".to_string(),
    };
    let res_tipo_errado = client
        .post("/reset-password")
        .json(&reset_payload_tipo_errado)
        .await;

    assert_eq!(res_tipo_errado.status_code(), 401);

    let error_text = res_tipo_errado.text();
    assert!(
        error_text.contains("inválido") || error_text.contains("expirado"),
        "Erro inesperado: '{}'. Esperado 'inválido' ou 'expirado'",
        error_text
    );
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
        new_password: "123".to_string(), // Senha fraca (também muito curta)
    };
    let res_reset = client.post("/reset-password").json(&reset_payload).await;

    assert_eq!(res_reset.status_code(), 400);
    let error_text = res_reset.text();

    assert!(
        error_text.contains("Senha")
            || error_text.contains("senha")
            || error_text.contains("length")
            || error_text.contains("new_password")
            || error_text.contains("fraca")
            || error_text.contains("mínimo"),
        "Erro inesperado: {}",
        error_text
    );
}
