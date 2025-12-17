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

// ============================
// BUILDING TYPE TESTS (Phase 2)
// ============================

#[tokio::test]
async fn test_create_building_type_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Edifício Comercial",
            "description": "Prédio para uso comercial"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Edifício Comercial");
    assert_eq!(body["description"], "Prédio para uso comercial");
}

#[tokio::test]
async fn test_create_building_type_duplicate_name_returns_conflict() {
    let app = common::spawn_app().await;

    // Create first building type
    app.api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Edifício Industrial"}))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Edifício Industrial"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_building_type_success() {
    let app = common::spawn_app().await;

    // Create building type
    let create_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Edifício Residencial",
            "description": "Prédio para moradia"
        }))
        .await;

    let created: Value = create_response.json();
    let building_type_id = created["id"].as_str().unwrap();

    // Get building type
    let response = app
        .api
        .get(&format!("/api/admin/locations/building-types/{}", building_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Edifício Residencial");
}

#[tokio::test]
async fn test_update_building_type_success() {
    let app = common::spawn_app().await;

    // Create building type
    let create_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Galpão"}))
        .await;

    let created: Value = create_response.json();
    let building_type_id = created["id"].as_str().unwrap();

    // Update building type
    let response = app
        .api
        .put(&format!("/api/admin/locations/building-types/{}", building_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Galpão Industrial",
            "description": "Grande área para armazenamento"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Galpão Industrial");
    assert_eq!(body["description"], "Grande área para armazenamento");
}

#[tokio::test]
async fn test_delete_building_type_success() {
    let app = common::spawn_app().await;

    // Create building type
    let create_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Torre"}))
        .await;

    let created: Value = create_response.json();
    let building_type_id = created["id"].as_str().unwrap();

    // Delete building type
    let response = app
        .api
        .delete(&format!("/api/admin/locations/building-types/{}", building_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let get_response = app
        .api
        .get(&format!("/api/admin/locations/building-types/{}", building_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// SPACE TYPE TESTS (Phase 2)
// ============================

#[tokio::test]
async fn test_create_space_type_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala de Reunião",
            "description": "Espaço para reuniões corporativas"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Sala de Reunião");
    assert_eq!(body["description"], "Espaço para reuniões corporativas");
}

#[tokio::test]
async fn test_create_space_type_duplicate_name_returns_conflict() {
    let app = common::spawn_app().await;

    // Create first space type
    app.api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Escritório"}))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Escritório"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_space_type_success() {
    let app = common::spawn_app().await;

    // Create space type
    let create_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Almoxarifado",
            "description": "Local de armazenamento"
        }))
        .await;

    let created: Value = create_response.json();
    let space_type_id = created["id"].as_str().unwrap();

    // Get space type
    let response = app
        .api
        .get(&format!("/api/admin/locations/space-types/{}", space_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Almoxarifado");
}

#[tokio::test]
async fn test_update_space_type_success() {
    let app = common::spawn_app().await;

    // Create space type
    let create_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Cafeteria"}))
        .await;

    let created: Value = create_response.json();
    let space_type_id = created["id"].as_str().unwrap();

    // Update space type
    let response = app
        .api
        .put(&format!("/api/admin/locations/space-types/{}", space_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Refeitório",
            "description": "Área para alimentação dos funcionários"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Refeitório");
    assert_eq!(body["description"], "Área para alimentação dos funcionários");
}

#[tokio::test]
async fn test_delete_space_type_success() {
    let app = common::spawn_app().await;

    // Create space type
    let create_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Auditório"}))
        .await;

    let created: Value = create_response.json();
    let space_type_id = created["id"].as_str().unwrap();

    // Delete space type
    let response = app
        .api
        .delete(&format!("/api/admin/locations/space-types/{}", space_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let get_response = app
        .api
        .get(&format!("/api/admin/locations/space-types/{}", space_type_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// DEPARTMENT CATEGORY TESTS (Phase 2)
// ============================

#[tokio::test]
async fn test_create_department_category_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/locations/department-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Operacional",
            "description": "Departamentos relacionados às operações"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Operacional");
    assert_eq!(body["description"], "Departamentos relacionados às operações");
}

#[tokio::test]
async fn test_create_department_category_duplicate_name_returns_conflict() {
    let app = common::spawn_app().await;

    // Create first department category
    app.api
        .post("/api/admin/locations/department-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Administrativo"}))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/department-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Administrativo"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_department_category_success() {
    let app = common::spawn_app().await;

    // Create department category
    let create_response = app
        .api
        .post("/api/admin/locations/department-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Comercial",
            "description": "Departamentos de vendas e marketing"
        }))
        .await;

    let created: Value = create_response.json();
    let dept_category_id = created["id"].as_str().unwrap();

    // Get department category
    let response = app
        .api
        .get(&format!("/api/admin/locations/department-categories/{}", dept_category_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Comercial");
}

#[tokio::test]
async fn test_update_department_category_success() {
    let app = common::spawn_app().await;

    // Create department category
    let create_response = app
        .api
        .post("/api/admin/locations/department-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Suporte"}))
        .await;

    let created: Value = create_response.json();
    let dept_category_id = created["id"].as_str().unwrap();

    // Update department category
    let response = app
        .api
        .put(&format!("/api/admin/locations/department-categories/{}", dept_category_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Atendimento ao Cliente",
            "description": "Departamentos focados em suporte e relacionamento"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Atendimento ao Cliente");
    assert_eq!(body["description"], "Departamentos focados em suporte e relacionamento");
}

#[tokio::test]
async fn test_delete_department_category_success() {
    let app = common::spawn_app().await;

    // Create department category
    let create_response = app
        .api
        .post("/api/admin/locations/department-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Financeiro"}))
        .await;

    let created: Value = create_response.json();
    let dept_category_id = created["id"].as_str().unwrap();

    // Delete department category
    let response = app
        .api
        .delete(&format!("/api/admin/locations/department-categories/{}", dept_category_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let get_response = app
        .api
        .get(&format!("/api/admin/locations/department-categories/{}", dept_category_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// SITE TESTS (Phase 3A)
// ============================

#[tokio::test]
async fn test_create_site_success() {
    let app = common::spawn_app().await;

    // Create prerequisites: state, city, site_type
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "São Paulo", "code": "SP"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "São Paulo", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Matriz"}))
        .await;
    let site_type: Value = site_type_response.json();

    // Create site
    let response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sede Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"],
            "address": "Av. Paulista, 1000"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Sede Central");
    assert_eq!(body["city_id"], city["id"]);
    assert_eq!(body["city_name"], "São Paulo");
    assert_eq!(body["state_name"], "São Paulo");
    assert_eq!(body["state_code"], "SP");
    assert_eq!(body["site_type_id"], site_type["id"]);
    assert_eq!(body["site_type_name"], "Matriz");
    assert_eq!(body["address"], "Av. Paulista, 1000");
}

#[tokio::test]
async fn test_create_site_invalid_city_returns_404() {
    let app = common::spawn_app().await;

    // Create site_type only
    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Filial"}))
        .await;
    let site_type: Value = site_type_response.json();

    let fake_city_id = Uuid::new_v4();

    let response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Site Teste",
            "city_id": fake_city_id,
            "site_type_id": site_type["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_site_invalid_site_type_returns_404() {
    let app = common::spawn_app().await;

    // Create state and city only
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Rio de Janeiro", "code": "RJ"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Rio de Janeiro", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let fake_site_type_id = Uuid::new_v4();

    let response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Site Teste",
            "city_id": city["id"],
            "site_type_id": fake_site_type_id
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_site_duplicate_name_in_same_city_returns_conflict() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Minas Gerais", "code": "MG"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Belo Horizonte", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Escritório"}))
        .await;
    let site_type: Value = site_type_response.json();

    // Create first site
    app.api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Unidade Centro",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;

    // Try to create duplicate in same city
    let response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Unidade Centro",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_site_same_name_in_different_cities_succeeds() {
    let app = common::spawn_app().await;

    // Create state and two cities
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Paraná", "code": "PR"}))
        .await;
    let state: Value = state_response.json();

    let city1_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Curitiba", "state_id": state["id"]}))
        .await;
    let city1: Value = city1_response.json();

    let city2_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Londrina", "state_id": state["id"]}))
        .await;
    let city2: Value = city2_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Loja"}))
        .await;
    let site_type: Value = site_type_response.json();

    // Create site with same name in city1
    let response1 = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Loja Principal",
            "city_id": city1["id"],
            "site_type_id": site_type["id"]
        }))
        .await;

    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Create site with same name in city2 - should succeed
    let response2 = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Loja Principal",
            "city_id": city2["id"],
            "site_type_id": site_type["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_site_with_relations() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Bahia", "code": "BA"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Salvador", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Depósito"}))
        .await;
    let site_type: Value = site_type_response.json();

    let create_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "CD Bahia",
            "city_id": city["id"],
            "site_type_id": site_type["id"],
            "address": "Rodovia BR-324, Km 10"
        }))
        .await;
    let created: Value = create_response.json();

    // Get site with relations
    let response = app
        .api
        .get(&format!("/api/admin/locations/sites/{}", created["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "CD Bahia");
    assert_eq!(body["city_name"], "Salvador");
    assert_eq!(body["state_name"], "Bahia");
    assert_eq!(body["state_code"], "BA");
    assert_eq!(body["site_type_name"], "Depósito");
    assert_eq!(body["address"], "Rodovia BR-324, Km 10");
}

#[tokio::test]
async fn test_update_site_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Ceará", "code": "CE"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Fortaleza", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Fábrica"}))
        .await;
    let site_type: Value = site_type_response.json();

    let create_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Planta 1",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let created: Value = create_response.json();

    // Update site
    let response = app
        .api
        .put(&format!("/api/admin/locations/sites/{}", created["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Planta Industrial 1",
            "address": "Distrito Industrial, Lote 42"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Planta Industrial 1");
    assert_eq!(body["address"], "Distrito Industrial, Lote 42");
}

#[tokio::test]
async fn test_delete_site_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Pernambuco", "code": "PE"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Recife", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Ponto de Venda"}))
        .await;
    let site_type: Value = site_type_response.json();

    let create_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "PDV Centro",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let created: Value = create_response.json();
    let site_id = created["id"].as_str().unwrap();

    // Delete site
    let response = app
        .api
        .delete(&format!("/api/admin/locations/sites/{}", site_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let get_response = app
        .api
        .get(&format!("/api/admin/locations/sites/{}", site_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_sites_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Santa Catarina", "code": "SC"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Florianópolis", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Filial"}))
        .await;
    let site_type: Value = site_type_response.json();

    // Create multiple sites
    for name in ["Site A", "Site B", "Site C"] {
        app.api
            .post("/api/admin/locations/sites")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "city_id": city["id"],
                "site_type_id": site_type["id"]
            }))
            .await;
    }

    // List sites
    let response = app
        .api
        .get("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["sites"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_list_sites_filtered_by_city() {
    let app = common::spawn_app().await;

    // Create state and two cities
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Goiás", "code": "GO"}))
        .await;
    let state: Value = state_response.json();

    let city1_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Goiânia", "state_id": state["id"]}))
        .await;
    let city1: Value = city1_response.json();

    let city2_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Anápolis", "state_id": state["id"]}))
        .await;
    let city2: Value = city2_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Regional"}))
        .await;
    let site_type: Value = site_type_response.json();

    // Create sites in city1
    app.api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Site Goiânia 1",
            "city_id": city1["id"],
            "site_type_id": site_type["id"]
        }))
        .await;

    // Create site in city2
    app.api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Site Anápolis 1",
            "city_id": city2["id"],
            "site_type_id": site_type["id"]
        }))
        .await;

    // List sites filtered by city1
    let response = app
        .api
        .get(&format!("/api/admin/locations/sites?city_id={}", city1["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let sites = body["sites"].as_array().unwrap();

    // All sites should be from city1
    for site in sites {
        assert_eq!(site["city_id"], city1["id"]);
    }
}

#[tokio::test]
async fn test_list_sites_filtered_by_site_type() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Espírito Santo", "code": "ES"}))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Vitória", "state_id": state["id"]}))
        .await;
    let city: Value = city_response.json();

    let site_type1_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Administrativo"}))
        .await;
    let site_type1: Value = site_type1_response.json();

    let site_type2_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": "Operacional"}))
        .await;
    let site_type2: Value = site_type2_response.json();

    // Create site with site_type1
    app.api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sede Admin",
            "city_id": city["id"],
            "site_type_id": site_type1["id"]
        }))
        .await;

    // Create site with site_type2
    app.api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro Operacional",
            "city_id": city["id"],
            "site_type_id": site_type2["id"]
        }))
        .await;

    // List sites filtered by site_type1
    let response = app
        .api
        .get(&format!("/api/admin/locations/sites?site_type_id={}", site_type1["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let sites = body["sites"].as_array().unwrap();

    // All sites should be of site_type1
    for site in sites {
        assert_eq!(site["site_type_id"], site_type1["id"]);
    }
}
