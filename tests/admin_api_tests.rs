mod common; // Importa o nosso common.rs

use http::StatusCode;
use serde_json::{json, Value};
use waterswamp::models::UserDto; // Usado para deserializar a resposta

#[tokio::test]
async fn test_admin_add_duplicate_policy_returns_200() {
    let app = common::spawn_app().await;

    // 1. Primeira adição (Deve retornar 201 CREATED)
    let response1 = app
        .api
        .post("/api/admin/policies")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "subject": "bob", // O utilizador 'bob' existe
            "object": "/api/test_dup",
            "action": "POST"
        }))
        .await;

    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // 2. Segunda adição (duplicada - Deve retornar 200 OK, idempotente)
    let response2 = app
        .api
        .post("/api/admin/policies")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "subject": "bob",
            "object": "/api/test_dup",
            "action": "POST"
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_add_policy_nonexistent_user_returns_404() {
    let app = common::spawn_app().await;

    // Tenta adicionar política para um utilizador que não existe
    let response = app
        .api
        .post("/api/admin/policies")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "subject": "usuario_fantasma_12345",
            "object": "/api/test",
            "action": "GET"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_invalid_policy_payload_returns_400() {
    let app = common::spawn_app().await;

    // Payload com "subject" vazio, o que deve falhar a validação do DTO
    let response = app
        .api
        .post("/api/admin/policies")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "subject": "",
            "object": "/api/test",
            "action": "GET"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_remove_nonexistent_policy_returns_404() {
    let app = common::spawn_app().await;

    // Tenta remover uma política que nunca foi adicionada
    // CORREÇÃO: Usar .delete() e .json()
    let response = app
        .api
        .delete("/api/admin/policies")
        .json(&json!({
            "subject": "alice", // Utilizador existe
            "object": "/api/rota_inexistente_123",
            "action": "DELETE"
        }))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dynamic_permission_flow() {
    let app = common::spawn_app().await;

    // Recurso que vamos proteger/liberar dinamicamente
    let resource = "/admin/dashboard"; //

    // 0. Setup: Garante que Bob (utilizador normal) NÃO tem acesso inicial
    // CORREÇÃO: Usar .delete() e .json()
    app.api
        .delete("/api/admin/policies")
        .json(&json!({
            "subject": "bob",
            "object": resource,
            "action": "GET"
        }))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    // 1. Bob tenta aceder (Deve falhar: 403 Forbidden)
    let response1 = app
        .api
        .get(resource)
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(response1.status_code(), StatusCode::FORBIDDEN);

    // 2. Admin (Alice) concede permissão para Bob
    let add_response = app
        .api
        .post("/api/admin/policies")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "subject": "bob",
            "object": resource,
            "action": "GET"
        }))
        .await;
    assert_eq!(add_response.status_code(), StatusCode::CREATED); //

    // 3. Bob tenta aceder novamente (Deve ter sucesso: 200 OK)
    let response2 = app
        .api
        .get(resource)
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(response2.status_code(), StatusCode::OK);

    // 4. Admin (Alice) revoga a permissão
    // CORREÇÃO: Usar .delete() e .json()
    let remove_response = app
        .api
        .delete("/api/admin/policies")
        .json(&json!({
            "subject": "bob",
            "object": resource,
            "action": "GET"
        }))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(remove_response.status_code(), StatusCode::NO_CONTENT); //

    // 5. Bob tenta aceder mais uma vez (Deve falhar novamente: 403 Forbidden)
    let response3 = app
        .api
        .get(resource)
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(response3.status_code(), StatusCode::FORBIDDEN);
}

// --- Testes Adicionais (CRUD de Utilizadores) ---

#[tokio::test]
async fn test_admin_list_users_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();
    // Verifica se a lista contém pelo menos 'alice' e 'bob'
    let json: Value = response.json();
    assert!(json["users"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_admin_create_user_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "username": "new_user_by_admin",
            "password": "strongPassword123!"
        }))
        .await;

    // O handler admin_create_user retorna Ok(Json(user)), que é 200 OK
    response.assert_status_ok(); //

    let user = response.json::<UserDto>();
    assert_eq!(user.username, "new_user_by_admin");
}

#[tokio::test]
async fn test_admin_access_denied_for_normal_user() {
    let app = common::spawn_app().await;

    // Tenta aceder a /api/admin/users usando o token de 'bob' (utilizador normal)
    let response = app
        .api
        .get("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    response.assert_status_forbidden();
}

#[tokio::test]
async fn test_admin_access_denied_without_token() {
    let app = common::spawn_app().await;

    // Tenta aceder sem token
    let response = app.api.get("/api/admin/users").await;

    response.assert_status_unauthorized();
}
