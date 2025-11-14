mod common;

use domain::models::UserDetailDto;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

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
    let resource = "/admin/dashboard";

    // 0. Setup: Garante que Bob (utilizador normal) NÃO tem acesso inicial
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
    assert_eq!(add_response.status_code(), StatusCode::CREATED);

    // 3. Bob tenta aceder novamente (Deve ter sucesso: 200 OK)
    let response2 = app
        .api
        .get(resource)
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(response2.status_code(), StatusCode::OK);

    // 4. Admin (Alice) revoga a permissão
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
    assert_eq!(remove_response.status_code(), StatusCode::NO_CONTENT);

    // 5. Bob tenta aceder mais uma vez (Deve falhar novamente: 403 Forbidden)
    let response3 = app
        .api
        .get(resource)
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(response3.status_code(), StatusCode::FORBIDDEN);
}

// --- ⭐ NOVOS TESTES (CRUD de Utilizadores - Tarefa 4) ---

/// (Subtarefas 4.3 e 4.2) Testa criar e depois buscar um utilizador
#[tokio::test]
async fn test_admin_create_user_and_get_user() {
    let app = common::spawn_app().await;
    let unique_username = format!("user_{}", uuid::Uuid::new_v4());
    let unique_email = format!("{}@example.com", unique_username);

    // 1. Criar Utilizador
    let response = app
        .api
        .post("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "username": unique_username,
            "email": unique_email,
            "password": "Tr0ng$ecuR3!Data#42",
            "role": "user" // Testar a definição de role
        }))
        .await;

    response.assert_status_ok(); // 200 OK

    // 2. Verificar a Resposta (UserDetailDto)
    let user_detail: UserDetailDto = response.json();
    assert_eq!(user_detail.user.username, unique_username);
    assert_eq!(user_detail.user.email, unique_email);
    assert_eq!(user_detail.roles, vec!["user"]);

    let new_user_id = user_detail.user.id;

    // 3. Buscar o Utilizador (Get User)
    let get_response = app
        .api
        .get(&format!("/api/admin/users/{}", new_user_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    get_response.assert_status_ok(); // 200 OK

    // 4. Verificar Resposta do Get
    let fetched_user: UserDetailDto = get_response.json();
    assert_eq!(fetched_user.user.username, unique_username);
    assert_eq!(fetched_user.roles, vec!["user"]);
}

/// (Subtarefa 4.3) Testa conflito de username
#[tokio::test]
async fn test_admin_create_user_conflict() {
    let app = common::spawn_app().await;

    // 'bob' já foi criado no common::spawn_app()
    let response = app
        .api
        .post("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "username": "bob",
            "email": "bob_new@example.com",
            "password": "OutraSenha123!",
            "role": "user"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT); // 409
}

#[tokio::test]
async fn test_admin_create_user_email_conflict() {
    let app = common::spawn_app().await;

    // 'bob@temp.example.com' foi criado na migração
    let response = app
        .api
        .post("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "username": "new_bob",
            "email": "bob@temp.example.com",
            "password": "OutraSenha123!",
            "role": "user"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT); // 409
}

/// (Subtarefa 4.1) Testa listagem, paginação e busca
#[tokio::test]
async fn test_admin_list_users_pagination_and_search() {
    let app = common::spawn_app().await;

    // 1. Testar Listagem (sabemos que 'alice' e 'bob' existem)
    let response = app
        .api
        .get("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    response.assert_status_ok();
    let json: Value = response.json();
    assert!(json["total"].as_i64().unwrap() >= 2);
    assert!(json["users"].as_array().unwrap().len() >= 2);

    // 2. Testar Paginação (limit=1)
    let response_limit = app
        .api
        .get("/api/admin/users?limit=1&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    let json_limit: Value = response_limit.json();
    assert_eq!(json_limit["users"].as_array().unwrap().len(), 1);
    assert_eq!(json_limit["limit"], 1);

    // 3. Testar Busca
    let response_search = app
        .api
        .get("/api/admin/users?search=alice")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    let json_search: Value = response_search.json();
    assert_eq!(json_search["users"].as_array().unwrap().len(), 1);
    assert_eq!(json_search["users"][0]["username"], "alice");
}

/// (Subtarefa 4.4) Testa atualização de password e role
#[tokio::test]
async fn test_admin_update_user_role_and_password() {
    let app = common::spawn_app().await;

    // 1. Criar utilizador 'charlie' com role 'user'
    let create_response = app
        .api
        .post("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "username": "charlie",
            "email": "charlie@example.com",
            "password": "PasswordCharlie123!",
            "role": "user"
        }))
        .await;
    let charlie_id = create_response.json::<UserDetailDto>().user.id;

    // 2. Atualizar Role de 'user' para 'admin'
    let update_role_response = app
        .api
        .put(&format!("/api/admin/users/{}", charlie_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "role": "admin"
        }))
        .await;

    update_role_response.assert_status_ok();
    let updated_user: UserDetailDto = update_role_response.json();
    assert_eq!(updated_user.roles, vec!["admin"]);

    // 3. Atualizar Password
    let update_pass_response = app
        .api
        .put(&format!("/api/admin/users/{}", charlie_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "password": "NovaPasswordSuperForte987!"
        }))
        .await;

    update_pass_response.assert_status_ok();
}

/// (Subtarefa 4.5) Testa apagar utilizador E não se apagar a si mesmo
#[tokio::test]
async fn test_admin_delete_user_and_cannot_delete_self() {
    let app = common::spawn_app().await;

    // 1. Criar utilizador 'dave' para apagar
    let create_response = app
        .api
        .post("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "username": "dave_to_delete",
            "email": "dave@example.com",
            "password": "PasswordDave123!",
            "role": "user"
        }))
        .await;
    let dave_id = create_response.json::<UserDetailDto>().user.id;

    // 2. Apagar 'dave' (DEVE ter sucesso)
    let delete_response = app
        .api
        .delete(&format!("/api/admin/users/{}", dave_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT); // 204

    // 3. Tentar apagar 'alice' (o próprio admin) (DEVE FALHAR)
    // (Temos de ir buscar o ID da 'alice' à DB, tal como o common.rs faz)
    let alice_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&app.db_auth)
        .await
        .expect("Alice não encontrada no seed");

    let delete_self_response = app
        .api
        .delete(&format!("/api/admin/users/{}", alice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token)) // Token de Alice
        .await;

    // O handler admin_delete_user retorna AppError::Forbidden
    assert_eq!(delete_self_response.status_code(), StatusCode::FORBIDDEN); // 403
}

/// Teste de segurança: Utilizador normal não pode aceder a rotas admin
#[tokio::test]
async fn test_normal_user_cannot_access_admin_routes() {
    let app = common::spawn_app().await;

    // Tenta aceder a /api/admin/users usando o token de 'bob' (utilizador normal)
    let response = app
        .api
        .get("/api/admin/users")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    response.assert_status_forbidden(); // 403

    // Tenta aceder sem token
    let response_no_token = app.api.get("/api/admin/users").await;

    response_no_token.assert_status_unauthorized(); // 401
}
