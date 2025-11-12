mod common;

use axum::http::StatusCode;
use common::{login_user, register_user, spawn_app, TestApp};
use domain::models::{LoginPayload, LoginResponse, RefreshTokenPayload};
use waterswamp::error::AppError;

/// Teste de login com sucesso
#[tokio::test]
async fn test_login_success() {
    let app = spawn_app().await;
    let (username, password) = ("testuser", "StrongP@ss123");

    let response = register_user(&app, username, password).await;
    assert_eq!(response.status_code(), StatusCode::OK, "Falha ao registrar"); // <-- CORREÇÃO

    let response = login_user(&app, username, password).await;

    // Verifica se o login foi bem-sucedido
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
    let wrong_password = "WrongPassword!";

    let response = register_user(&app, username, password).await;
    assert_eq!(response.status_code(), StatusCode::OK, "Falha ao registrar"); // <-- CORREÇÃO

    // Tenta fazer login com a senha errada
    let login_payload = LoginPayload {
        username: username.to_string(),
        password: wrong_password.to_string(),
    };

    let response = app.api.post("/auth/login").json(&login_payload).await;

    // Verifica se o status é 401 Unauthorized (Senha Inválida)
    assert_eq!(
        response.status_code(), // <-- CORREÇÃO
        StatusCode::UNAUTHORIZED,
        "Deveria falhar com 401 Unauthorized"
    );

    // Verifica a mensagem de erro
    let error_response: AppError = response
        .json()
        .await
        .expect("Falha ao deserializar resposta de erro");
    match error_response {
        AppError::InvalidPassword => { /* Correto */ }
        _ => panic!("Erro inesperado retornado: {:?}", error_response),
    }
}

// --- NOVOS TESTES (TAREFA 3) ---

/// Critério 3.2: Cada refresh gera um novo access token E um novo refresh token
#[tokio::test]
async fn test_refresh_token_rotation_success() {
    let app = spawn_app().await;
    let (username, password) = ("rotatesuccess", "StrongP@ss123");

    register_user(&app, username, password).await;
    let login_res_1 = login_user(&app, username, password).await;

    // 1. Usar o primeiro refresh token
    let refresh_payload_1 = RefreshTokenPayload {
        refresh_token: login_res_1.refresh_token.clone(),
    };

    let response = app
        .api
        .post("/auth/refresh-token")
        .json(&refresh_payload_1)
        .await;

    assert_eq!(
        response.status_code(), // <-- CORREÇÃO
        StatusCode::OK,
        "Falha ao dar refresh no token"
    );

    // 2. Deserializar a *nova* LoginResponse
    let login_res_2: LoginResponse = response
        .json()
        .await
        .expect("Falha ao deserializar LoginResponse do refresh");

    // 3. Validar que os tokens SÃO DIFERENTES
    assert_ne!(
        login_res_1.access_token, login_res_2.access_token,
        "O Access Token não foi rotacionado"
    );
    assert_ne!(
        login_res_1.refresh_token, login_res_2.refresh_token,
        "O Refresh Token não foi rotacionado"
    );
}

/// Critério 3.2: O token anterior é revogado após o uso
#[tokio::test]
async fn test_refresh_token_revoked_after_use() {
    let app = spawn_app().await;
    let (username, password) = ("revokedafteruse", "StrongP@ss123");

    register_user(&app, username, password).await;
    let login_res = login_user(&app, username, password).await;

    // 1. Usar o token pela primeira vez (sucesso)
    let refresh_payload = RefreshTokenPayload {
        refresh_token: login_res.refresh_token.clone(),
    };

    let response = app
        .api
        .post("/auth/refresh-token")
        .json(&refresh_payload)
        .await;
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "Falha no primeiro refresh"
    ); // <-- CORREÇÃO

    // 2. Tentar usar o *mesmo* token pela segunda vez (deve falhar)
    let response_2 = app
        .api
        .post("/auth/refresh-token")
        .json(&refresh_payload)
        .await;

    // 3. Validar que falhou com 401 (Detecção de Roubo ativada)
    assert_eq!(
        response_2.status_code(), // <-- CORREÇÃO
        StatusCode::UNAUTHORIZED,
        "Token reutilizado deveria causar 401 Unauthorized"
    );

    // 4. Validar a mensagem de erro específica
    let error: AppError = response_2.json().await.unwrap();
    assert!(
        matches!(error, AppError::Unauthorized(msg) if msg.contains("Sessão invalidada")),
        "Mensagem de erro incorreta para detecção de roubo"
    );
}

/// Critério 3.3: Reuso de token revogado revoga toda a família (Detecção de Roubo)
#[tokio::test]
async fn test_refresh_token_theft_detection_revokes_family() {
    let app = spawn_app().await;
    let (username, password) = ("theftdetection", "StrongP@ss123");

    register_user(&app, username, password).await;

    // 1. Login inicial (T1)
    let login_res_1 = login_user(&app, username, password).await;
    let refresh_token_1 = login_res_1.refresh_token;

    // 2. Primeiro Refresh (T2)
    let payload_1 = RefreshTokenPayload {
        refresh_token: refresh_token_1.clone(),
    };
    let res_2 = app.api.post("/auth/refresh-token").json(&payload_1).await;
    assert_eq!(res_2.status_code(), StatusCode::OK, "Falha no Refresh 1"); // <-- CORREÇÃO
    let login_res_2: LoginResponse = res_2.json().await.unwrap();
    let refresh_token_2 = login_res_2.refresh_token; // T2 (agora válido)
                                                     // T1 está agora revogado

    // 3. Segundo Refresh (T3)
    let payload_2 = RefreshTokenPayload {
        refresh_token: refresh_token_2.clone(),
    };
    let res_3 = app.api.post("/auth/refresh-token").json(&payload_2).await;
    assert_eq!(res_3.status_code(), StatusCode::OK, "Falha no Refresh 2"); // <-- CORREÇÃO
    let login_res_3: LoginResponse = res_3.json().await.unwrap();
    let refresh_token_3 = login_res_3.refresh_token; // T3 (agora válido)
                                                     // T2 está agora revogado

    // 4. --- Simulação de Roubo ---
    // O Invasor tenta reusar o T2 (que já foi usado/revogado)
    let stolen_payload_2 = RefreshTokenPayload {
        refresh_token: refresh_token_2.clone(),
    };
    let theft_response = app
        .api
        .post("/auth/refresh-token")
        .json(&stolen_payload_2)
        .await;

    // 5. Validar que a detecção de roubo funcionou
    assert_eq!(
        theft_response.status_code(), // <-- CORREÇÃO
        StatusCode::UNAUTHORIZED,
        "Detecção de roubo (reuso do T2) falhou em retornar 401"
    );
    let error: AppError = theft_response.json().await.unwrap();
    assert!(
        matches!(error, AppError::Unauthorized(msg) if msg.contains("Sessão invalidada")),
        "Mensagem de erro incorreta para detecção de roubo"
    );

    // 6. --- Verificação da Revogação da Família ---
    // O usuário legítimo tenta usar o T3 (que deveria ser válido, mas foi revogado pela família)
    let legit_payload_3 = RefreshTokenPayload {
        refresh_token: refresh_token_3.clone(),
    };
    let revoked_response = app
        .api
        .post("/auth/refresh-token")
        .json(&legit_payload_3)
        .await;

    // 7. Validar que o T3 agora é inválido
    assert_eq!(
        revoked_response.status_code(), // <-- CORREÇÃO
        StatusCode::UNAUTHORIZED,
        "Token T3 (legítimo) não foi revogado pela família"
    );
    let error_t3: AppError = revoked_response.json().await.unwrap();
    assert!(
        matches!(error_t3, AppError::Unauthorized(msg) if msg.contains("Sessão invalidada")),
        "O T3 deveria estar inválido (revogado), mas deu outro erro"
    );
}
