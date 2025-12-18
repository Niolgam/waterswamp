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

// ============================
// Building Tests (Phase 3B)
// ============================

#[tokio::test]
async fn test_create_building_success() {
    let app = common::spawn_app().await;

    // Create prerequisites: state, city, site_type, building_type, site
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "code": "SP"
        }))
        .await;
    assert_eq!(state_response.status_code(), StatusCode::CREATED);
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "state_id": state["id"]
        }))
        .await;
    assert_eq!(city_response.status_code(), StatusCode::CREATED);
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus",
            "description": "Campus universitário"
        }))
        .await;
    assert_eq!(site_type_response.status_code(), StatusCode::CREATED);
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Administrativo",
            "description": "Edifício administrativo"
        }))
        .await;
    assert_eq!(building_type_response.status_code(), StatusCode::CREATED);
    let building_type: Value = building_type_response.json();

    let site_response = app
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
    assert_eq!(site_response.status_code(), StatusCode::CREATED);
    let site: Value = site_response.json();

    // Create building
    let response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Principal",
            "site_id": site["id"],
            "building_type_id": building_type["id"],
            "description": "Prédio principal do campus"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();

    assert_eq!(body["name"], "Prédio Principal");
    assert_eq!(body["site_id"], site["id"]);
    assert_eq!(body["site_name"], "Sede Central");
    assert_eq!(body["city_name"], "São Paulo");
    assert_eq!(body["state_code"], "SP");
    assert_eq!(body["building_type_id"], building_type["id"]);
    assert_eq!(body["building_type_name"], "Administrativo");
    assert_eq!(body["description"], "Prédio principal do campus");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn test_create_building_invalid_site_returns_404() {
    let app = common::spawn_app().await;

    // Create building_type only
    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Administrativo",
            "description": "Edifício administrativo"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    // Try to create building with invalid site_id
    let response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Principal",
            "site_id": "00000000-0000-0000-0000-000000000000",
            "building_type_id": building_type["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_building_invalid_building_type_returns_404() {
    let app = common::spawn_app().await;

    // Create prerequisites: state, city, site_type, site
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

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sede Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    // Try to create building with invalid building_type_id
    let response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Principal",
            "site_id": site["id"],
            "building_type_id": "00000000-0000-0000-0000-000000000000"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_building_duplicate_name_in_same_site_returns_conflict() {
    let app = common::spawn_app().await;

    // Create prerequisites
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

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Administrativo"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sede Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    // Create first building
    let response1 = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Try to create second building with same name in same site
    let response2 = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_building_same_name_in_different_sites_succeeds() {
    let app = common::spawn_app().await;

    // Create prerequisites
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

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Administrativo"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    // Create two different sites
    let site1_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sede Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site1: Value = site1_response.json();

    let site2_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Filial Norte",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site2: Value = site2_response.json();

    // Create building in site1
    let response1 = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Principal",
            "site_id": site1["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Create building with same name in site2 - should succeed
    let response2 = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Principal",
            "site_id": site2["id"],
            "building_type_id": building_type["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CREATED);
    let body2: Value = response2.json();
    assert_eq!(body2["name"], "Prédio Principal");
    assert_eq!(body2["site_id"], site2["id"]);
}

#[tokio::test]
async fn test_get_building_with_relations() {
    let app = common::spawn_app().await;

    // Create prerequisites and building
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "code": "RJ"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Corporativo"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Comercial"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro Empresarial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let create_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Norte",
            "site_id": site["id"],
            "building_type_id": building_type["id"],
            "description": "Torre com 30 andares"
        }))
        .await;
    let created_building: Value = create_response.json();

    // Get building by ID
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/buildings/{}",
            created_building["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    // Verify all related data is included
    assert_eq!(body["name"], "Torre Norte");
    assert_eq!(body["site_name"], "Centro Empresarial");
    assert_eq!(body["city_name"], "Rio de Janeiro");
    assert_eq!(body["state_name"], "Rio de Janeiro");
    assert_eq!(body["state_code"], "RJ");
    assert_eq!(body["building_type_name"], "Torre Comercial");
    assert_eq!(body["description"], "Torre com 30 andares");
}

#[tokio::test]
async fn test_update_building_success() {
    let app = common::spawn_app().await;

    // Create prerequisites and building
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Minas Gerais",
            "code": "MG"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Belo Horizonte",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Industrial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type1_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Fábrica"
        }))
        .await;
    let building_type1: Value = building_type1_response.json();

    let building_type2_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Armazém"
        }))
        .await;
    let building_type2: Value = building_type2_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Complexo Industrial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let create_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Galpão 1",
            "site_id": site["id"],
            "building_type_id": building_type1["id"]
        }))
        .await;
    let created_building: Value = create_response.json();

    // Update building
    let response = app
        .api
        .put(&format!(
            "/api/admin/locations/buildings/{}",
            created_building["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Galpão Principal",
            "building_type_id": building_type2["id"],
            "description": "Armazém central de distribuição"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert_eq!(body["name"], "Galpão Principal");
    assert_eq!(body["building_type_id"], building_type2["id"]);
    assert_eq!(body["building_type_name"], "Armazém");
    assert_eq!(body["description"], "Armazém central de distribuição");
}

#[tokio::test]
async fn test_delete_building_success() {
    let app = common::spawn_app().await;

    // Create prerequisites and building
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bahia",
            "code": "BA"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Salvador",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Comercial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Shopping"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro Comercial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let create_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Edifício A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let created_building: Value = create_response.json();

    // Delete building
    let response = app
        .api
        .delete(&format!(
            "/api/admin/locations/buildings/{}",
            created_building["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify building no longer exists
    let get_response = app
        .api
        .get(&format!(
            "/api/admin/locations/buildings/{}",
            created_building["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_buildings_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Paraná",
            "code": "PR"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Curitiba",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Universitário"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Acadêmico"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus Universitário",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    // Create multiple buildings
    for i in 1..=3 {
        app.api
            .post("/api/admin/locations/buildings")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": format!("Bloco {}", i),
                "site_id": site["id"],
                "building_type_id": building_type["id"]
            }))
            .await;
    }

    // List buildings
    let response = app
        .api
        .get("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    let buildings = body["buildings"].as_array().unwrap();
    assert!(buildings.len() >= 3);

    // Verify structure
    let first_building = &buildings[0];
    assert!(first_building["id"].is_string());
    assert!(first_building["name"].is_string());
    assert!(first_building["site_name"].is_string());
    assert!(first_building["city_name"].is_string());
    assert!(first_building["building_type_name"].is_string());
}

#[tokio::test]
async fn test_list_buildings_filtered_by_site() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio Grande do Sul",
            "code": "RS"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Porto Alegre",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Empresarial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Escritório"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    // Create two sites
    let site1_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Complexo A",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site1: Value = site1_response.json();

    let site2_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Complexo B",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site2: Value = site2_response.json();

    // Create buildings in site1
    app.api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio 1",
            "site_id": site1["id"],
            "building_type_id": building_type["id"]
        }))
        .await;

    // Create building in site2
    app.api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio 2",
            "site_id": site2["id"],
            "building_type_id": building_type["id"]
        }))
        .await;

    // List buildings filtered by site1
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/buildings?site_id={}",
            site1["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let buildings = body["buildings"].as_array().unwrap();

    // All buildings should be of site1
    for building in buildings {
        assert_eq!(building["site_id"], site1["id"]);
    }
}

#[tokio::test]
async fn test_list_buildings_filtered_by_building_type() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Santa Catarina",
            "code": "SC"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Florianópolis",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Tecnológico"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    // Create two building types
    let building_type1_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Laboratório"
        }))
        .await;
    let building_type1: Value = building_type1_response.json();

    let building_type2_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Data Center"
        }))
        .await;
    let building_type2: Value = building_type2_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Parque Tecnológico",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    // Create buildings with different types
    app.api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Lab A",
            "site_id": site["id"],
            "building_type_id": building_type1["id"]
        }))
        .await;

    app.api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "DC 1",
            "site_id": site["id"],
            "building_type_id": building_type2["id"]
        }))
        .await;

    // List buildings filtered by building_type1
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/buildings?building_type_id={}",
            building_type1["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let buildings = body["buildings"].as_array().unwrap();

    // All buildings should be of building_type1
    for building in buildings {
        assert_eq!(building["building_type_id"], building_type1["id"]);
    }
}

// =============================================================================
// FLOOR TESTS (Phase 3C)
// =============================================================================

#[tokio::test]
async fn test_create_floor_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
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

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Empresarial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Comercial"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro Empresarial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Principal",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create floor
    let response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 5,
            "building_id": building["id"],
            "description": "Andar executivo"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();

    assert!(body["id"].is_string());
    assert_eq!(body["floor_number"], 5);
    assert_eq!(body["building_id"], building["id"]);
    assert_eq!(body["building_name"], "Torre Principal");
    assert_eq!(body["site_id"], site["id"]);
    assert_eq!(body["site_name"], "Centro Empresarial");
    assert_eq!(body["city_id"], city["id"]);
    assert_eq!(body["city_name"], "São Paulo");
    assert_eq!(body["state_id"], state["id"]);
    assert_eq!(body["state_name"], "São Paulo");
    assert_eq!(body["state_code"], "SP");
    assert_eq!(body["description"], "Andar executivo");
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn test_create_floor_with_invalid_building_id_returns_404() {
    let app = common::spawn_app().await;

    let fake_building_id = "00000000-0000-0000-0000-000000000000";

    let response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": fake_building_id
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_floor_duplicate_floor_number_in_same_building_returns_conflict() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Paraná",
            "code": "PR"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Curitiba",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Corporativo"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Edifício Comercial"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Complexo A",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create first floor with floor_number 3
    let response1 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 3,
            "building_id": building["id"]
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Try to create another floor with same floor_number in same building
    let response2 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 3,
            "building_id": building["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_floor_same_floor_number_in_different_buildings_succeeds() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Minas Gerais",
            "code": "MG"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Belo Horizonte",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Administrativo"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    // Create two different buildings
    let building1_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building1: Value = building1_response.json();

    let building2_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio B",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building2: Value = building2_response.json();

    // Create floor in building1
    let response1 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 2,
            "building_id": building1["id"]
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Create floor with same floor_number in building2 - should succeed
    let response2 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 2,
            "building_id": building2["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CREATED);
    let body2: Value = response2.json();
    assert_eq!(body2["floor_number"], 2);
    assert_eq!(body2["building_id"], building2["id"]);
}

#[tokio::test]
async fn test_get_floor_with_relations() {
    let app = common::spawn_app().await;

    // Create prerequisites and floor
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "code": "RJ"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Industrial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Fábrica"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Parque Industrial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Fábrica 1",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": building["id"],
            "description": "Produção principal"
        }))
        .await;
    let floor: Value = floor_response.json();

    // Get floor and verify all relations
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert_eq!(body["id"], floor["id"]);
    assert_eq!(body["floor_number"], 1);
    assert_eq!(body["building_id"], building["id"]);
    assert_eq!(body["building_name"], "Fábrica 1");
    assert_eq!(body["site_id"], site["id"]);
    assert_eq!(body["site_name"], "Parque Industrial");
    assert_eq!(body["city_id"], city["id"]);
    assert_eq!(body["city_name"], "Rio de Janeiro");
    assert_eq!(body["state_id"], state["id"]);
    assert_eq!(body["state_name"], "Rio de Janeiro");
    assert_eq!(body["state_code"], "RJ");
    assert_eq!(body["description"], "Produção principal");
}

#[tokio::test]
async fn test_update_floor_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bahia",
            "code": "BA"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Salvador",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hotelaria"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hotel"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Resort Beach",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Norte",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 10,
            "building_id": building["id"],
            "description": "Suítes luxo"
        }))
        .await;
    let floor: Value = floor_response.json();

    // Update floor
    let response = app
        .api
        .put(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 12,
            "description": "Suítes premium"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert_eq!(body["id"], floor["id"]);
    assert_eq!(body["floor_number"], 12);
    assert_eq!(body["building_id"], building["id"]);
    assert_eq!(body["description"], "Suítes premium");
}

#[tokio::test]
async fn test_delete_floor_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Ceará",
            "code": "CE"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Fortaleza",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Educacional"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Academia"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus Universitário",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bloco Acadêmico",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 4,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    // Delete floor
    let response = app
        .api
        .delete(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify floor is deleted
    let get_response = app
        .api
        .get(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_floors_with_pagination() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Pernambuco",
            "code": "PE"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Recife",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Residencial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Apartamentos"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Condomínio Vista Mar",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create 5 floors
    for i in 1..=5 {
        app.api
            .post("/api/admin/locations/floors")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "floor_number": i,
                "building_id": building["id"]
            }))
            .await;
    }

    // List floors with pagination (limit=2)
    let response = app
        .api
        .get("/api/admin/locations/floors?limit=2")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let floors = body["floors"].as_array().unwrap();

    assert_eq!(floors.len(), 2);
    assert_eq!(body["total"].as_i64().unwrap(), 5);
    assert_eq!(body["limit"].as_i64().unwrap(), 2);
    assert_eq!(body["offset"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_list_floors_filtered_by_building() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Goiás",
            "code": "GO"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Goiânia",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Corporativo"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Escritórios"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Parque Empresarial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    // Create two buildings
    let building1_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Alpha",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building1: Value = building1_response.json();

    let building2_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Beta",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building2: Value = building2_response.json();

    // Create floors in building1
    app.api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": building1["id"]
        }))
        .await;

    app.api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 2,
            "building_id": building1["id"]
        }))
        .await;

    // Create floor in building2
    app.api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": building2["id"]
        }))
        .await;

    // List floors filtered by building1
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/floors?building_id={}",
            building1["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let floors = body["floors"].as_array().unwrap();

    // Should return only floors from building1
    assert_eq!(floors.len(), 2);
    for floor in floors {
        assert_eq!(floor["building_id"], building1["id"]);
    }
}

#[tokio::test]
async fn test_create_floor_with_negative_floor_number_underground() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Distrito Federal",
            "code": "DF"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Brasília",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Governamental"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Complexo Administrativo"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Esplanada dos Ministérios",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Anexo Central",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create underground floors (negative floor numbers)
    let response_minus_2 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": -2,
            "building_id": building["id"],
            "description": "Subsolo 2 - Estacionamento"
        }))
        .await;

    assert_eq!(response_minus_2.status_code(), StatusCode::CREATED);
    let body_minus_2: Value = response_minus_2.json();
    assert_eq!(body_minus_2["floor_number"], -2);
    assert_eq!(body_minus_2["description"], "Subsolo 2 - Estacionamento");

    let response_minus_1 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": -1,
            "building_id": building["id"],
            "description": "Subsolo 1 - Arquivo"
        }))
        .await;

    assert_eq!(response_minus_1.status_code(), StatusCode::CREATED);
    let body_minus_1: Value = response_minus_1.json();
    assert_eq!(body_minus_1["floor_number"], -1);
    assert_eq!(body_minus_1["description"], "Subsolo 1 - Arquivo");

    // Create ground floor and upper floors
    let response_0 = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 0,
            "building_id": building["id"],
            "description": "Térreo"
        }))
        .await;

    assert_eq!(response_0.status_code(), StatusCode::CREATED);
    let body_0: Value = response_0.json();
    assert_eq!(body_0["floor_number"], 0);

    // List all floors and verify ordering
    let list_response = app
        .api
        .get(&format!(
            "/api/admin/locations/floors?building_id={}",
            building["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(list_response.status_code(), StatusCode::OK);
    let list_body: Value = list_response.json();
    let floors = list_body["floors"].as_array().unwrap();

    // Verify all three floors are returned
    assert_eq!(floors.len(), 3);

    // Floors should be ordered by floor_number (ascending: -2, -1, 0)
    assert_eq!(floors[0]["floor_number"], -2);
    assert_eq!(floors[1]["floor_number"], -1);
    assert_eq!(floors[2]["floor_number"], 0);
}

#[tokio::test]
async fn test_list_floors_with_floor_number_search() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Amazonas",
            "code": "AM"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Manaus",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Logístico"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro de Distribuição"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hub Logístico Norte",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Armazém Central",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create floors with numbers: 1, 2, 10, 11, 12
    for floor_num in [1, 2, 10, 11, 12] {
        app.api
            .post("/api/admin/locations/floors")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "floor_number": floor_num,
                "building_id": building["id"]
            }))
            .await;
    }

    // Search for floors with "1" in floor_number (should match 1, 10, 11, 12)
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/floors?building_id={}&search=1",
            building["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let floors = body["floors"].as_array().unwrap();

    // Should match 4 floors: 1, 10, 11, 12
    assert_eq!(floors.len(), 4);

    let floor_numbers: Vec<i64> = floors
        .iter()
        .map(|f| f["floor_number"].as_i64().unwrap())
        .collect();

    assert!(floor_numbers.contains(&1));
    assert!(floor_numbers.contains(&10));
    assert!(floor_numbers.contains(&11));
    assert!(floor_numbers.contains(&12));
    assert!(!floor_numbers.contains(&2));
}

#[tokio::test]
async fn test_floor_endpoints_require_admin_authorization() {
    let app = common::spawn_app().await;

    // Create prerequisites with admin token
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Espírito Santo",
            "code": "ES"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Vitória",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Comercial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Shopping"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Shopping Center",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre de Lojas",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 3,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    // Try all floor endpoints with regular user (non-admin) - should fail with 403
    let list_response = app
        .api
        .get("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(list_response.status_code(), StatusCode::FORBIDDEN);

    let get_response = app
        .api
        .get(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(get_response.status_code(), StatusCode::FORBIDDEN);

    let create_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "floor_number": 5,
            "building_id": building["id"]
        }))
        .await;
    assert_eq!(create_response.status_code(), StatusCode::FORBIDDEN);

    let update_response = app
        .api
        .put(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "floor_number": 4
        }))
        .await;
    assert_eq!(update_response.status_code(), StatusCode::FORBIDDEN);

    let delete_response = app
        .api
        .delete(&format!(
            "/api/admin/locations/floors/{}",
            floor["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(delete_response.status_code(), StatusCode::FORBIDDEN);
}

// =============================================================================
// SPACE TESTS (Phase 3D)
// =============================================================================

#[tokio::test]
async fn test_create_space_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
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

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "São Paulo",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Corporativo"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Escritório"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala de Reunião"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sede Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Principal",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 5,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    // Create space
    let response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 501",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"],
            "description": "Sala de reunião executiva"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();

    assert!(body["id"].is_string());
    assert_eq!(body["name"], "Sala 501");
    assert_eq!(body["floor_id"], floor["id"]);
    assert_eq!(body["floor_number"], 5);
    assert_eq!(body["building_id"], building["id"]);
    assert_eq!(body["building_name"], "Torre Principal");
    assert_eq!(body["site_id"], site["id"]);
    assert_eq!(body["site_name"], "Sede Central");
    assert_eq!(body["city_id"], city["id"]);
    assert_eq!(body["city_name"], "São Paulo");
    assert_eq!(body["state_id"], state["id"]);
    assert_eq!(body["state_name"], "São Paulo");
    assert_eq!(body["state_code"], "SP");
    assert_eq!(body["space_type_id"], space_type["id"]);
    assert_eq!(body["space_type_name"], "Sala de Reunião");
    assert_eq!(body["description"], "Sala de reunião executiva");
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn test_create_space_with_invalid_floor_id_returns_404() {
    let app = common::spawn_app().await;

    // Create space_type
    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala de Reunião"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let fake_floor_id = "00000000-0000-0000-0000-000000000000";

    let response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 100",
            "floor_id": fake_floor_id,
            "space_type_id": space_type["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_space_with_invalid_space_type_id_returns_404() {
    let app = common::spawn_app().await;

    // Create full hierarchy up to floor
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "code": "RJ"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Rio de Janeiro",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Comercial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Shopping"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Shopping Center",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 2,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    let fake_space_type_id = "00000000-0000-0000-0000-000000000000";

    let response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Loja 201",
            "floor_id": floor["id"],
            "space_type_id": fake_space_type_id
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_space_duplicate_name_in_same_floor_returns_conflict() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Minas Gerais",
            "code": "MG"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Belo Horizonte",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Industrial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Fábrica"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Laboratório"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Complexo Industrial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    // Create first space
    let response1 = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Lab A1",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"]
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Try to create another space with same name in same floor
    let response2 = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Lab A1",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_space_same_name_in_different_floors_succeeds() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Paraná",
            "code": "PR"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Curitiba",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hotelaria"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hotel"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Suíte"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hotel Plaza",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Norte",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create two different floors
    let floor1_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 5,
            "building_id": building["id"]
        }))
        .await;
    let floor1: Value = floor1_response.json();

    let floor2_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 6,
            "building_id": building["id"]
        }))
        .await;
    let floor2: Value = floor2_response.json();

    // Create space in floor1
    let response1 = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Suíte 01",
            "floor_id": floor1["id"],
            "space_type_id": space_type["id"]
        }))
        .await;
    assert_eq!(response1.status_code(), StatusCode::CREATED);

    // Create space with same name in floor2 - should succeed
    let response2 = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Suíte 01",
            "floor_id": floor2["id"],
            "space_type_id": space_type["id"]
        }))
        .await;

    assert_eq!(response2.status_code(), StatusCode::CREATED);
    let body2: Value = response2.json();
    assert_eq!(body2["name"], "Suíte 01");
    assert_eq!(body2["floor_id"], floor2["id"]);
    assert_eq!(body2["floor_number"], 6);
}

#[tokio::test]
async fn test_get_space_with_relations() {
    let app = common::spawn_app().await;

    // Create full hierarchy
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bahia",
            "code": "BA"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Salvador",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Educacional"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala de Aula"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Campus Universitário",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bloco A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 2,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    let space_response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 201",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"],
            "description": "Sala de aula grande"
        }))
        .await;
    let space: Value = space_response.json();

    // Get space and verify all relations
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert_eq!(body["id"], space["id"]);
    assert_eq!(body["name"], "Sala 201");
    assert_eq!(body["floor_id"], floor["id"]);
    assert_eq!(body["floor_number"], 2);
    assert_eq!(body["building_id"], building["id"]);
    assert_eq!(body["building_name"], "Bloco A");
    assert_eq!(body["site_id"], site["id"]);
    assert_eq!(body["site_name"], "Campus Universitário");
    assert_eq!(body["city_id"], city["id"]);
    assert_eq!(body["city_name"], "Salvador");
    assert_eq!(body["state_id"], state["id"]);
    assert_eq!(body["state_name"], "Bahia");
    assert_eq!(body["state_code"], "BA");
    assert_eq!(body["space_type_id"], space_type["id"]);
    assert_eq!(body["space_type_name"], "Sala de Aula");
    assert_eq!(body["description"], "Sala de aula grande");
}

#[tokio::test]
async fn test_update_space_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Ceará",
            "code": "CE"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Fortaleza",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Residencial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Condomínio"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Apartamento"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Residencial Beira Mar",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 8,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    let space_response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Apto 801",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"],
            "description": "Apartamento 2 quartos"
        }))
        .await;
    let space: Value = space_response.json();

    // Update space
    let response = app
        .api
        .put(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Apto 802",
            "description": "Apartamento 3 quartos"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert_eq!(body["id"], space["id"]);
    assert_eq!(body["name"], "Apto 802");
    assert_eq!(body["floor_id"], floor["id"]);
    assert_eq!(body["description"], "Apartamento 3 quartos");
}

#[tokio::test]
async fn test_delete_space_success() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Pernambuco",
            "code": "PE"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Recife",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Médico"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hospital"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Consultório"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Hospital Central",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Ala Sul",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 3,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    let space_response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Consultório 301",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"]
        }))
        .await;
    let space: Value = space_response.json();

    // Delete space
    let response = app
        .api
        .delete(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify space is deleted
    let get_response = app
        .api
        .get(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_spaces_with_pagination() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Goiás",
            "code": "GO"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Goiânia",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Logístico"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Armazém"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Depósito"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro Logístico",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Armazém A",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    // Create 5 spaces
    for i in 1..=5 {
        app.api
            .post("/api/admin/locations/spaces")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": format!("Dep {}", i),
                "floor_id": floor["id"],
                "space_type_id": space_type["id"]
            }))
            .await;
    }

    // List spaces with pagination (limit=2)
    let response = app
        .api
        .get("/api/admin/locations/spaces?limit=2")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let spaces = body["spaces"].as_array().unwrap();

    assert_eq!(spaces.len(), 2);
    assert_eq!(body["total"].as_i64().unwrap(), 5);
    assert_eq!(body["limit"].as_i64().unwrap(), 2);
    assert_eq!(body["offset"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_list_spaces_filtered_by_floor() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Amazonas",
            "code": "AM"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Manaus",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Comercial"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Comercial"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Escritório"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro Empresarial",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Torre Alpha",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    // Create two floors
    let floor1_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 10,
            "building_id": building["id"]
        }))
        .await;
    let floor1: Value = floor1_response.json();

    let floor2_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 11,
            "building_id": building["id"]
        }))
        .await;
    let floor2: Value = floor2_response.json();

    // Create spaces in floor1
    app.api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 1001",
            "floor_id": floor1["id"],
            "space_type_id": space_type["id"]
        }))
        .await;

    app.api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 1002",
            "floor_id": floor1["id"],
            "space_type_id": space_type["id"]
        }))
        .await;

    // Create space in floor2
    app.api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 1101",
            "floor_id": floor2["id"],
            "space_type_id": space_type["id"]
        }))
        .await;

    // List spaces filtered by floor1
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/spaces?floor_id={}",
            floor1["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let spaces = body["spaces"].as_array().unwrap();

    // Should return only spaces from floor1
    assert_eq!(spaces.len(), 2);
    for space in spaces {
        assert_eq!(space["floor_id"], floor1["id"]);
    }
}

#[tokio::test]
async fn test_list_spaces_filtered_by_space_type() {
    let app = common::spawn_app().await;

    // Create prerequisites
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Distrito Federal",
            "code": "DF"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Brasília",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Coworking"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Prédio Comercial"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    // Create two space types
    let space_type1_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala Privada"
        }))
        .await;
    let space_type1: Value = space_type1_response.json();

    let space_type2_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala de Reunião"
        }))
        .await;
    let space_type2: Value = space_type2_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Coworking Hub",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Edifício Central",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 7,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    // Create spaces with different types
    app.api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 701",
            "floor_id": floor["id"],
            "space_type_id": space_type1["id"]
        }))
        .await;

    app.api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Sala 702",
            "floor_id": floor["id"],
            "space_type_id": space_type1["id"]
        }))
        .await;

    app.api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Reunião A",
            "floor_id": floor["id"],
            "space_type_id": space_type2["id"]
        }))
        .await;

    // List spaces filtered by space_type1
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/spaces?space_type_id={}",
            space_type1["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let spaces = body["spaces"].as_array().unwrap();

    // Should return only spaces with space_type1
    assert_eq!(spaces.len(), 2);
    for space in spaces {
        assert_eq!(space["space_type_id"], space_type1["id"]);
    }
}

#[tokio::test]
async fn test_space_endpoints_require_admin_authorization() {
    let app = common::spawn_app().await;

    // Create prerequisites with admin token
    let state_response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Espírito Santo",
            "code": "ES"
        }))
        .await;
    let state: Value = state_response.json();

    let city_response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Vitória",
            "state_id": state["id"]
        }))
        .await;
    let city: Value = city_response.json();

    let site_type_response = app
        .api
        .post("/api/admin/locations/site-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Laboratório"
        }))
        .await;
    let site_type: Value = site_type_response.json();

    let building_type_response = app
        .api
        .post("/api/admin/locations/building-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Pesquisa"
        }))
        .await;
    let building_type: Value = building_type_response.json();

    let space_type_response = app
        .api
        .post("/api/admin/locations/space-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Lab Química"
        }))
        .await;
    let space_type: Value = space_type_response.json();

    let site_response = app
        .api
        .post("/api/admin/locations/sites")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Centro de Pesquisa",
            "city_id": city["id"],
            "site_type_id": site_type["id"]
        }))
        .await;
    let site: Value = site_response.json();

    let building_response = app
        .api
        .post("/api/admin/locations/buildings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Bloco Experimental",
            "site_id": site["id"],
            "building_type_id": building_type["id"]
        }))
        .await;
    let building: Value = building_response.json();

    let floor_response = app
        .api
        .post("/api/admin/locations/floors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "floor_number": 1,
            "building_id": building["id"]
        }))
        .await;
    let floor: Value = floor_response.json();

    let space_response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Lab 101",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"]
        }))
        .await;
    let space: Value = space_response.json();

    // Try all space endpoints with regular user (non-admin) - should fail with 403
    let list_response = app
        .api
        .get("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(list_response.status_code(), StatusCode::FORBIDDEN);

    let get_response = app
        .api
        .get(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(get_response.status_code(), StatusCode::FORBIDDEN);

    let create_response = app
        .api
        .post("/api/admin/locations/spaces")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "name": "Lab 102",
            "floor_id": floor["id"],
            "space_type_id": space_type["id"]
        }))
        .await;
    assert_eq!(create_response.status_code(), StatusCode::FORBIDDEN);

    let update_response = app
        .api
        .put(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "name": "Lab 103"
        }))
        .await;
    assert_eq!(update_response.status_code(), StatusCode::FORBIDDEN);

    let delete_response = app
        .api
        .delete(&format!(
            "/api/admin/locations/spaces/{}",
            space["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(delete_response.status_code(), StatusCode::FORBIDDEN);
}
