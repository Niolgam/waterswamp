use http::StatusCode;
use serde_json::{json, Value};

mod common;

#[tokio::test]
async fn test_register_success() {
    let app = common::spawn_app().await;

    // Usa um username único para evitar conflitos com outros testes
    let unique_username = format!("user_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": unique_username,
        "email": unique_email,
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
    let email1 = format!("dup1_{}@example.com", username);
    let email2 = format!("dup2_{}@example.com", username);

    // 1. Primeiro registro (deve funcionar)
    let response1 = app
        .api
        .post("/register")
        .json(&json!({
            "username": username,
             "email": email1,
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
            "email": email2,
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
            "email": format!("weakpass_{}@example.com", uuid::Uuid::new_v4()),
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
            "email": format!("shortpass_{}@example.com", uuid::Uuid::new_v4()),
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
            "username": "ab",
            "email": format!("shortuser_{}@example.com", uuid::Uuid::new_v4()),
            "password": "S3nh@Forte123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_success_sends_welcome_email() {
    let app = common::spawn_app().await;

    let unique_username = format!("user_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    // 1. Fazer o pedido de registo
    let response = app
        .api
        .post("/register")
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "S3nh@Forte123"
        }))
        .await;

    // 2. Verificar se a API funcionou
    response.assert_status_ok();
    let body: Value = response.json();
    assert!(body["access_token"].is_string(), "access_token missing");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let received_emails = app.email_service.messages.lock().await;

    // Verificar se 2 emails foram recebidos (verification + welcome)
    assert_eq!(
        received_emails.len(),
        2,
        "Deveria ter recebido 2 emails (verification + welcome)"
    );

    // Find verification email
    let verification_email = received_emails
        .iter()
        .find(|e| e.subject.contains("Verifique"));
    assert!(
        verification_email.is_some(),
        "Deveria ter email de verificação"
    );
    let ver_email = verification_email.unwrap();
    assert_eq!(ver_email.to, unique_email);
    assert_eq!(ver_email.template, "email_verification.html");
    assert!(ver_email.context.get("verification_link").is_some());

    // Find welcome email
    let welcome_email = received_emails
        .iter()
        .find(|e| e.subject.contains("Bem-vindo"));
    assert!(welcome_email.is_some(), "Deveria ter email de boas-vindas");
    let wel_email = welcome_email.unwrap();
    assert_eq!(wel_email.to, unique_email);
    assert_eq!(wel_email.template, "welcome.html");
    assert_eq!(
        wel_email.context.get("username").unwrap().as_str().unwrap(),
        unique_username
    );
}
