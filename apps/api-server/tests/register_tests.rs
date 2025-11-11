use http::StatusCode;
use serde_json::{json, Value};

// Importa o setup de 'tests/common.rs'
mod common;

#[tokio::test]
async fn test_register_success() {
    let app = common::spawn_app().await;

    // Usa um username único para evitar conflitos com outros testes
    let unique_username = format!("user_{}", uuid::Uuid::new_v4());

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "password": "S3nh@Forte123"
        }))
        .await;

    // Debugging útil caso o teste falhe
    if response.status_code() != StatusCode::OK {
        let body_text = response.text();
        println!("Erro no registro: {}", body_text);
        panic!("Registro falhou com status {}", response.status_code());
    }

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["access_token"].is_string(), "access_token missing");
    assert!(body["refresh_token"].is_string(), "refresh_token missing");
}

#[tokio::test]
async fn test_register_username_taken() {
    let app = common::spawn_app().await;

    let username = format!("duplicated_{}", uuid::Uuid::new_v4());

    // 1. Primeiro registro (deve funcionar)
    let response1 = app
        .api
        .post("/register")
        .json(&json!({
            "username": username,
            "password": "S3nh@Forte123"
        }))
        .await;

    response1.assert_status_ok();

    // 2. Segundo registro (deve falhar com conflito)
    let response2 = app
        .api
        .post("/register")
        .json(&json!({
            "username": username,
            "password": "Outr@Senh@456"
        }))
        .await;

    // O handler retorna AppError::Conflict
    // que é mapeado para StatusCode::CONFLICT (409)
    assert_eq!(
        response2.status_code(),
        StatusCode::CONFLICT,
        "Esperado 409 CONFLICT, recebido {}",
        response2.status_code()
    );
}

#[tokio::test]
async fn test_register_weak_password() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": format!("user_{}", uuid::Uuid::new_v4()),
            "password": "senha123" // Reprovado pelo zxcvbn
        }))
        .await;

    // Mapeado para AppError::BadRequest
    assert_eq!(
        response.status_code(),
        StatusCode::BAD_REQUEST, // 400
        "Senha fraca deveria retornar 400"
    );
}

#[tokio::test]
async fn test_register_short_password() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": format!("user_{}", uuid::Uuid::new_v4()),
            "password": "Abc1!" // Falha na validação de DTO (min = 8)
        }))
        .await;

    // Mapeado para AppError::Validation, que é 400
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_short_username() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": "ab", // Falha na validação de DTO (min = 3)
            "password": "S3nh@Forte123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}
