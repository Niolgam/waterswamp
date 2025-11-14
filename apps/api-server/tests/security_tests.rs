mod common;

use axum::http::StatusCode;
use axum_test::TestResponse;
use common::{spawn_app, TestApp};
use domain::models::{LoginPayload, LoginResponse, RefreshTokenPayload, RegisterPayload};
use serde_json::{json, Value};

/// Teste de login com sucesso
#[tokio::test]
async fn test_login_success() {
    let app = spawn_app().await;
    let (username, password) = ("testuser", "StrongP@ss123");
    let email = "testuser@example.com";

    // 1. Registar um utilizador
    let response = register_user(&app, username, email, password).await;
    assert_eq!(response.status_code(), StatusCode::OK, "Falha ao registar");

    // 2. Fazer login
    let response = login_user(&app, username, password).await;

    // 3. Verificar se os tokens foram retornados
    assert!(
        response.access_token.len() > 0,
        "Access token não foi retornado"
    );
    assert!(
        response.refresh_token.len() > 0,
        "Refresh token não foi retornado"
    );
}

/// Teste de login com senha incorreta
#[tokio::test]
async fn test_login_fail_wrong_password() {
    let app = spawn_app().await;
    let (username, password) = ("testuser2", "StrongP@ss123");
    let email = "testuser2@example.com";
    let wrong_password = "WrongPassword!";

    // 1. Registar o utilizador
    let response = register_user(&app, username, email, password).await;
    assert_eq!(response.status_code(), StatusCode::OK, "Falha ao registar");

    // 2. Tentar fazer login com a senha errada
    let login_payload = LoginPayload {
        username: username.to_string(),
        password: wrong_password.to_string(),
    };

    let response = app.api.post("/login").json(&login_payload).await;

    // 3. Verificar se o status é 401 Unauthorized
    assert_eq!(
        response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Deveria falhar com 401 Unauthorized"
    );

    // 4. Verificar a mensagem de erro pública (JSON)
    // CORREÇÃO: .json::<Value>() é síncrono. Sem .await.
    let error_response: Value = response.json::<Value>(); // <-- MUDANÇA AQUI

    assert_eq!(
        error_response,
        json!({"error": "Usuário ou senha inválidos."}),
        "Mensagem de erro incorreta"
    );
}

// --- Testes de Rotação de Token (Tarefa 3) ---

/// Critério 3.2: Cada refresh gera um novo access token E um novo refresh token
#[tokio::test]
async fn test_refresh_token_rotation_success() {
    let app = spawn_app().await;
    let (username, password) = ("rotatesuccess", "StrongP@ss123");
    let email = "rotatesuccess@example.com";

    register_user(&app, username, email, password).await;
    let login_res_1 = login_user(&app, username, password).await;

    // 1. Usar o primeiro refresh token
    let refresh_payload_1 = RefreshTokenPayload {
        refresh_token: login_res_1.refresh_token.clone(),
    };

    // --- CORREÇÃO AQUI ---
    // Adiciona uma espera de 1 segundo para garantir que o timestamp 'iat' (issued at)
    // do próximo token de acesso seja diferente.
    // Isto é necessário por causa da assinatura determinística do EdDSA.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // --- FIM DA CORREÇÃO ---

    let response = app
        .api
        .post("/refresh-token")
        .json(&refresh_payload_1)
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Falha ao dar refresh no token"
    );

    // 2. Deserializar a *nova* LoginResponse
    let login_res_2: LoginResponse = response.json::<LoginResponse>();

    // 3. Validar que os tokens SÃO DIFERENTES
    assert_ne!(
        login_res_1.access_token, login_res_2.access_token,
        "O Access Token não foi rotacionado"
    );

    // Esta asserção (que não chegou a correr) também deve passar,
    // pois o UUID do refresh token é sempre único.
    assert_ne!(
        login_res_1.refresh_token, login_res_2.refresh_token,
        "O Refresh Token não foi rotacionado"
    );
}

/// Critério 3.2: O token anterior é revogado após o uso
#[tokio::test]
async fn test_refresh_token_revoked_after_use() {
    let app = spawn_app().await;
    let (username, password) = ("rotatesuccess", "StrongP@ss123");
    let email = "revokedafteruse@example.com";

    register_user(&app, username, email, password).await;
    let login_res_1 = login_user(&app, username, password).await;

    // 1. Usar o primeiro refresh token
    let refresh_payload_1 = RefreshTokenPayload {
        refresh_token: login_res_1.refresh_token.clone(),
    };

    // --- CORREÇÃO AQUI ---
    // Adiciona uma espera de 1 segundo para garantir que o timestamp 'iat' (issued at)
    // do próximo token de acesso seja diferente.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // --- FIM DA CORREÇÃO ---

    let response = app
        .api
        .post("/refresh-token")
        .json(&refresh_payload_1)
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Falha ao dar refresh no token"
    );

    // 2. Deserializar a *nova* LoginResponse
    let login_res_2: LoginResponse = response.json::<LoginResponse>();

    // 3. Validar que os tokens SÃO DIFERENTES
    assert_ne!(
        login_res_1.access_token, login_res_2.access_token,
        "O Access Token não foi rotacionado"
    );
    // Esta asserção (que não chegou a correr) também deve passar,
    // pois o UUID é sempre único.
    assert_ne!(
        login_res_1.refresh_token, login_res_2.refresh_token,
        "O Refresh Token não foi rotacionado"
    );
}

/// Critério 3.3: Reuso de token revogado revoga toda a família (Deteção de Roubo)
#[tokio::test]
async fn test_refresh_token_theft_detection_revokes_family() {
    let app = spawn_app().await;
    let (username, password) = ("theftdetection", "StrongP@ss123");
    let email = "theftdetection@example.com";

    register_user(&app, username, email, password).await;

    // 1. Login inicial (Token T1)
    let login_res_1 = login_user(&app, username, password).await;
    let refresh_token_1 = login_res_1.refresh_token;

    // 2. Primeiro Refresh (Utilizador legítimo usa T1, recebe T2)
    let payload_1 = RefreshTokenPayload {
        refresh_token: refresh_token_1.clone(),
    };
    let res_2 = app.api.post("/refresh-token").json(&payload_1).await;
    assert_eq!(res_2.status_code(), StatusCode::OK, "Falha no Refresh 1");

    // CORREÇÃO: .json::<LoginResponse>() é síncrono. Sem .await.
    let login_res_2: LoginResponse = res_2.json::<LoginResponse>(); // <-- MUDANÇA AQUI
    let refresh_token_2 = login_res_2.refresh_token; // T2 (agora válido)

    // 3. Segundo Refresh (Utilizador legítimo usa T2, recebe T3)
    let payload_2 = RefreshTokenPayload {
        refresh_token: refresh_token_2.clone(),
    };
    let res_3 = app.api.post("/refresh-token").json(&payload_2).await;
    assert_eq!(res_3.status_code(), StatusCode::OK, "Falha no Refresh 2");

    // CORREÇÃO: .json::<LoginResponse>() é síncrono. Sem .await.
    let login_res_3: LoginResponse = res_3.json::<LoginResponse>(); // <-- MUDANÇA AQUI
    let refresh_token_3 = login_res_3.refresh_token; // T3 (agora válido)

    // 4. --- Simulação de Roubo ---
    let stolen_payload_2 = RefreshTokenPayload {
        refresh_token: refresh_token_2.clone(),
    };
    let theft_response = app.api.post("/refresh-token").json(&stolen_payload_2).await;

    // 5. Validar que a deteção de roubo funcionou (retorna 401)
    assert_eq!(
        theft_response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Deteção de roubo (reuso do T2) falhou em retornar 401"
    );
    // CORREÇÃO: .json::<Value>() é síncrono. Sem .await.
    let error: Value = theft_response.json::<Value>(); // <-- MUDANÇA AQUI
    assert_eq!(
        error,
        json!({"error": "Sessão invalidada por segurança"}),
        "Mensagem de erro incorreta para deteção de roubo"
    );

    // 6. --- Verificação da Revogação da Família ---
    let legit_payload_3 = RefreshTokenPayload {
        refresh_token: refresh_token_3.clone(),
    };
    let revoked_response = app.api.post("/refresh-token").json(&legit_payload_3).await;

    // 7. Validar que o T3 agora é inválido
    assert_eq!(
        revoked_response.status_code(),
        StatusCode::UNAUTHORIZED,
        "Token T3 (legítimo) não foi revogado pela família"
    );

    // CORREÇÃO: .json::<Value>() é síncrono. Sem .await.
    let error_t3: Value = revoked_response.json::<Value>(); // <-- MUDANÇA AQUI
    assert_eq!(
        error_t3,
        json!({"error": "Sessão invalidada por segurança"}),
        "O T3 deveria estar inválido (revogado), mas deu outro erro"
    );
}

// =================================A============================================
// HELPERS LOCAIS
// =============================================================================

/// Helper local para registar utilizador
async fn register_user(app: &TestApp, username: &str, email: &str, password: &str) -> TestResponse {
    let payload = RegisterPayload {
        username: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
    };
    app.api.post("/register").json(&payload).await
}

/// Helper local para fazer login e retornar LoginResponse
async fn login_user(app: &TestApp, username: &str, password: &str) -> LoginResponse {
    let payload = LoginPayload {
        username: username.to_string(),
        password: password.to_string(),
    };
    let response = app.api.post("/login").json(&payload).await;
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Falha ao fazer login"
    );
    // CORREÇÃO: .json::<LoginResponse>() é síncrono. Sem .await.
    response.json::<LoginResponse>() // <-- MUDANÇA AQUI
}
