mod common;

use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// STATE TESTS
// ============================

#[tokio::test]
async fn test_create_state_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "code": "SP"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "São Paulo");
    assert_eq!(body["code"], "SP");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_state_duplicate_code_returns_conflict() {
    let app = common::spawn_app().await;

    // Create first state
    app.api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "code": "RJ"
        }))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro Novo",
            "code": "RJ"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_state_invalid_code_returns_400() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "code": "SPP" // Invalid: 3 chars instead of 2
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_state_success() {
    let app = common::spawn_app().await;

    // Create state
    let create_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Minas Gerais",
            "code": "MG"
        }))
        .await;

    let created: Value = create_response.json();
    let state_id = created["id"].as_str().unwrap();

    // Get state
    let response = app
        .api
        .get(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Minas Gerais");
    assert_eq!(body["code"], "MG");
}

#[tokio::test]
async fn test_get_state_not_found() {
    let app = common::spawn_app().await;

    let fake_id = Uuid::new_v4();
    let response = app
        .api
        .get(&format!("/api/admin/locations/states/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_state_success() {
    let app = common::spawn_app().await;

    // Create state
    let create_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bahia",
            "code": "BA"
        }))
        .await;

    let created: Value = create_response.json();
    let state_id = created["id"].as_str().unwrap();

    // Update state
    let response = app
        .api
        .put(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bahia Atualizada"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Bahia Atualizada");
    assert_eq!(body["code"], "BA"); // Code unchanged
}

#[tokio::test]
async fn test_delete_state_success() {
    let app = common::spawn_app().await;

    // Create state
    let create_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Paraná",
            "code": "PR"
        }))
        .await;

    let created: Value = create_response.json();
    let state_id = created["id"].as_str().unwrap();

    // Delete state
    let response = app
        .api
        .delete(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify it's deleted
    let get_response = app
        .api
        .get(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_states_success() {
    let app = common::spawn_app().await;

    // Create multiple states
    for (name, code) in [("Ceará", "CE"), ("Pernambuco", "PE"), ("Alagoas", "AL")] {
        app.api
            .post("/api/admin/locations/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "code": code
            }))
            .await;
    }

    // List states
    let response = app
        .api
        .get("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["states"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_list_states_with_search() {
    let app = common::spawn_app().await;

    // Create states
    app.api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Santa Catarina", "code": "SC"}))
        .await;

    app.api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Rio Grande do Sul", "code": "RS"}))
        .await;

    // Search for "Santa"
    let response = app
        .api
        .get("/api/admin/locations/states?search=Santa")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let states = body["states"].as_array().unwrap();
    assert!(states.iter().any(|s| s["name"].as_str().unwrap().contains("Santa")));
}

#[tokio::test]
async fn test_state_requires_admin_role() {
    let app = common::spawn_app().await;

    // Try with user token (not admin)
    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "name": "Goiás",
            "code": "GO"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

// ============================
// CITY TESTS
// ============================

#[tokio::test]
async fn test_create_city_success() {
    let app = common::spawn_app().await;

    // Create state first
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "code": "SP"
        }))
        .await;

    let state: Value = state_response.json();
    let state_id = state["id"].as_str().unwrap();

    // Create city
    let response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campinas",
            "state_id": state_id
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Campinas");
    assert_eq!(body["state_id"], state_id);
}

#[tokio::test]
async fn test_create_city_invalid_state_returns_404() {
    let app = common::spawn_app().await;

    let fake_state_id = Uuid::new_v4();
    let response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Cidade Fantasma",
            "state_id": fake_state_id.to_string()
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_city_with_state_info() {
    let app = common::spawn_app().await;

    // Create state
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Rio de Janeiro", "code": "RJ"}))
        .await;

    let state: Value = state_response.json();
    let state_id = state["id"].as_str().unwrap();

    // Create city
    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Niterói",
            "state_id": state_id
        }))
        .await;

    let city: Value = city_response.json();
    let city_id = city["id"].as_str().unwrap();

    // Get city (should include state info)
    let response = app
        .api
        .get(&format!("/api/admin/locations/cities/{}", city_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Niterói");
    assert_eq!(body["state_name"], "Rio de Janeiro");
    assert_eq!(body["state_code"], "RJ");
}

#[tokio::test]
async fn test_list_cities_filter_by_state() {
    let app = common::spawn_app().await;

    // Create two states
    let sp_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "São Paulo", "code": "SP"}))
        .await;
    let sp: Value = sp_response.json();
    let sp_id = sp["id"].as_str().unwrap();

    let rj_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Rio de Janeiro", "code": "RJ"}))
        .await;
    let rj: Value = rj_response.json();
    let rj_id = rj["id"].as_str().unwrap();

    // Create cities in different states
    app.api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "São Paulo", "state_id": sp_id}))
        .await;

    app.api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Guarulhos", "state_id": sp_id}))
        .await;

    app.api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Rio de Janeiro", "state_id": rj_id}))
        .await;

    // Filter by SP state
    let response = app
        .api
        .get(&format!("/api/admin/locations/cities?state_id={}", sp_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let cities = body["cities"].as_array().unwrap();

    // Should only have SP cities
    for city in cities {
        assert_eq!(city["state_code"], "SP");
    }
}

#[tokio::test]
async fn test_update_city_success() {
    let app = common::spawn_app().await;

    // Create state and city
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Minas Gerais", "code": "MG"}))
        .await;
    let state: Value = state_response.json();
    let state_id = state["id"].as_str().unwrap();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Belo Horizonte", "state_id": state_id}))
        .await;
    let city: Value = city_response.json();
    let city_id = city["id"].as_str().unwrap();

    // Update city name
    let response = app
        .api
        .put(&format!("/api/admin/locations/cities/{}", city_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "BH"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "BH");
}

#[tokio::test]
async fn test_delete_city_success() {
    let app = common::spawn_app().await;

    // Create state and city
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Paraná", "code": "PR"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Curitiba", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();
    let city_id = city["id"].as_str().unwrap();

    // Delete city
    let response = app
        .api
        .delete(&format!("/api/admin/locations/cities/{}", city_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_state_cascades_to_cities() {
    let app = common::spawn_app().await;

    // Create state and city
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Bahia", "code": "BA"}))
        .await;
    let state: Value = state_response.json();
    let state_id = state["id"].as_str().unwrap();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Salvador", "state_id": state_id}))
        .await;
    let city: Value = city_response.json();
    let city_id = city["id"].as_str().unwrap();

    // Delete state (should cascade to city)
    app.api
        .delete(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    // City should be deleted too
    let response = app
        .api
        .get(&format!("/api/admin/locations/cities/{}", city_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// SITE TYPE TESTS
// ============================

#[tokio::test]
async fn test_create_site_type_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Matriz",
            "description": "Escritório principal da empresa"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Matriz");
    assert_eq!(body["description"], "Escritório principal da empresa");
}

#[tokio::test]
async fn test_create_site_type_duplicate_name_returns_conflict() {
    let app = common::spawn_app().await;

    // Create first site type
    app.api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Filial"}))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Filial"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_site_type_without_description() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Depósito"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Depósito");
    assert!(body["description"].is_null());
}

#[tokio::test]
async fn test_get_site_type_success() {
    let app = common::spawn_app().await;

    // Create site type
    let create_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro de Distribuição",
            "description": "CD para logística"
        }))
        .await;

    let created: Value = create_response.json();
    let site_type_id = created["id"].as_str().unwrap();

    // Get site type
    let response = app
        .api
        .get(&format!("/api/admin/locations/site-types/{}", site_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Centro de Distribuição");
}

#[tokio::test]
async fn test_update_site_type_success() {
    let app = common::spawn_app().await;

    // Create site type
    let create_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Loja"}))
        .await;

    let created: Value = create_response.json();
    let site_type_id = created["id"].as_str().unwrap();

    // Update site type
    let response = app
        .api
        .put(&format!("/api/admin/locations/site-types/{}", site_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Loja de Varejo",
            "description": "Ponto de venda ao consumidor final"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Loja de Varejo");
    assert_eq!(body["description"], "Ponto de venda ao consumidor final");
}

#[tokio::test]
async fn test_delete_site_type_success() {
    let app = common::spawn_app().await;

    // Create site type
    let create_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Franquia"}))
        .await;

    let created: Value = create_response.json();
    let site_type_id = created["id"].as_str().unwrap();

    // Delete site type
    let response = app
        .api
        .delete(&format!("/api/admin/locations/site-types/{}", site_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let get_response = app
        .api
        .get(&format!("/api/admin/locations/site-types/{}", site_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_site_types_success() {
    let app = common::spawn_app().await;

    // Create multiple site types
    for name in ["Warehouse", "Office", "Factory"] {
        app.api
            .post("/api/admin/locations/site-types")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({"name": name}))
            .await;
    }

    // List site types
    let response = app
        .api
        .get("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["site_types"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_site_type_requires_admin_role() {
    let app = common::spawn_app().await;

    // Try with user token (not admin)
    let response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({"name": "Test Type"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}
