mod common;

use common::TestApp;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// HELPERS
// ============================

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, &Uuid::new_v4().simple().to_string()[..8])
}

async fn create_make(app: &TestApp, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/fleet/makes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_model(app: &TestApp, make_id: &str, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/fleet/models")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "make_id": make_id, "name": name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_color(app: &TestApp, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/fleet/colors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_category(app: &TestApp, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/fleet/categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_fuel_type(app: &TestApp, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/fleet/fuel-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

/// Helper to create a full vehicle with all lookup data
async fn create_vehicle_with_deps(app: &TestApp) -> (Value, String) {
    let make = create_make(app, &random_name("Make")).await;
    let make_id = make["id"].as_str().unwrap().to_string();
    let model = create_model(app, &make_id, &random_name("Model")).await;
    let color = create_color(app, &random_name("Color")).await;
    let category = create_category(app, &random_name("Cat")).await;
    let fuel_type = create_fuel_type(app, &random_name("Fuel")).await;

    let plate = format!(
        "{}{}{}",
        &"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[..3],
        rand_digit(),
        &"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[..1],
    );
    // Use a simple unique plate
    let unique = &Uuid::new_v4().simple().to_string()[..4];
    let plate = format!("TST{}", unique).chars().take(7).collect::<String>();

    let response = app
        .api
        .post("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "license_plate": format!("ABC{}", &Uuid::new_v4().simple().to_string()[..4].to_uppercase().replace(|c: char| !c.is_ascii_alphanumeric(), "")),
            "chassis_number": generate_chassis(),
            "renavam": "00891749802",
            "category_id": category["id"],
            "make_id": make["id"],
            "model_id": model["id"],
            "color_id": color["id"],
            "fuel_type_id": fuel_type["id"],
            "manufacture_year": 2024,
            "model_year": 2025,
            "acquisition_type": "PURCHASE",
            "passenger_capacity": 5,
        }))
        .await;

    let vehicle_id = if response.status_code() == StatusCode::CREATED {
        let v: Value = response.json();
        v["id"].as_str().unwrap().to_string()
    } else {
        panic!("Failed to create vehicle: {} - {}", response.status_code(), response.text());
    };

    let vehicle: Value = app
        .api
        .get(&format!("/api/admin/fleet/vehicles/{}", vehicle_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await
        .json();

    (vehicle, vehicle_id)
}

fn generate_chassis() -> String {
    use std::collections::HashSet;
    let valid_chars: Vec<char> = "ABCDEFGHJKLMNPRSTUVWXYZ0123456789".chars().collect();
    let mut result = String::new();
    let uuid = Uuid::new_v4().simple().to_string().to_uppercase();
    for c in uuid.chars() {
        if valid_chars.contains(&c) && result.len() < 17 {
            result.push(c);
        }
    }
    while result.len() < 17 {
        result.push('A');
    }
    result
}

fn rand_digit() -> char {
    let uuid = Uuid::new_v4().simple().to_string();
    uuid.chars().find(|c| c.is_ascii_digit()).unwrap_or('1')
}

// ============================
// VEHICLE CATEGORY TESTS
// ============================

#[tokio::test]
async fn test_crud_vehicle_category() {
    let app = common::spawn_app().await;
    let name = random_name("Categoria");

    // CREATE
    let response = app
        .api
        .post("/api/admin/fleet/categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": &name,
            "description": "Categoria de teste"
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let created: Value = response.json();
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["name"], name);

    // READ
    let response = app
        .api
        .get(&format!("/api/admin/fleet/categories/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // UPDATE
    let new_name = random_name("CatUpdated");
    let response = app
        .api
        .put(&format!("/api/admin/fleet/categories/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": &new_name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let updated: Value = response.json();
    assert_eq!(updated["name"], new_name);

    // LIST
    let response = app
        .api
        .get("/api/admin/fleet/categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    assert!(list["total"].as_i64().unwrap() > 0);

    // DELETE
    let response = app
        .api
        .delete(&format!("/api/admin/fleet/categories/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_duplicate_category_returns_conflict() {
    let app = common::spawn_app().await;
    let name = random_name("DupCat");

    // First creation should succeed
    let response = app
        .api
        .post("/api/admin/fleet/categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": &name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);

    // Second creation should conflict
    let response = app
        .api
        .post("/api/admin/fleet/categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": &name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

// ============================
// VEHICLE MAKE & MODEL TESTS
// ============================

#[tokio::test]
async fn test_crud_vehicle_make_and_model() {
    let app = common::spawn_app().await;

    // Create Make
    let make_name = random_name("Toyota");
    let make = create_make(&app, &make_name).await;
    let make_id = make["id"].as_str().unwrap();
    assert_eq!(make["name"], make_name);

    // Create Model under Make
    let model_name = random_name("Corolla");
    let model = create_model(&app, make_id, &model_name).await;
    assert_eq!(model["name"], model_name);
    assert_eq!(model["make_id"], make_id);

    // List models filtered by make
    let response = app
        .api
        .get(&format!("/api/admin/fleet/models?make_id={}", make_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    assert!(list["total"].as_i64().unwrap() >= 1);
}

// ============================
// VEHICLE CRUD TESTS
// ============================

#[tokio::test]
async fn test_create_vehicle_with_valid_data() {
    let app = common::spawn_app().await;
    let make = create_make(&app, &random_name("Honda")).await;
    let make_id = make["id"].as_str().unwrap();
    let model = create_model(&app, make_id, &random_name("Civic")).await;
    let color = create_color(&app, &random_name("Azul")).await;
    let category = create_category(&app, &random_name("Passeio")).await;
    let fuel_type = create_fuel_type(&app, &random_name("Flex")).await;

    let chassis = generate_chassis();

    let response = app
        .api
        .post("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "license_plate": "ABC1D23",
            "chassis_number": chassis,
            "renavam": "00891749802",
            "category_id": category["id"],
            "make_id": make["id"],
            "model_id": model["id"],
            "color_id": color["id"],
            "fuel_type_id": fuel_type["id"],
            "manufacture_year": 2024,
            "model_year": 2025,
            "acquisition_type": "PURCHASE",
            "passenger_capacity": 5,
            "purchase_value": 85000.00,
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let vehicle: Value = response.json();
    assert_eq!(vehicle["license_plate"], "ABC1D23");
    assert!(vehicle["category_name"].as_str().is_some());
    assert!(vehicle["make_name"].as_str().is_some());
}

#[tokio::test]
async fn test_create_vehicle_invalid_plate() {
    let app = common::spawn_app().await;
    let make = create_make(&app, &random_name("Ford")).await;
    let make_id = make["id"].as_str().unwrap();
    let model = create_model(&app, make_id, &random_name("Ka")).await;
    let color = create_color(&app, &random_name("Preto")).await;
    let category = create_category(&app, &random_name("PasseioInv")).await;
    let fuel_type = create_fuel_type(&app, &random_name("Gas")).await;

    let response = app
        .api
        .post("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "license_plate": "INVALID",
            "chassis_number": generate_chassis(),
            "renavam": "00891749802",
            "category_id": category["id"],
            "make_id": make["id"],
            "model_id": model["id"],
            "color_id": color["id"],
            "fuel_type_id": fuel_type["id"],
            "manufacture_year": 2024,
            "model_year": 2025,
            "acquisition_type": "PURCHASE",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_vehicle_invalid_chassis() {
    let app = common::spawn_app().await;
    let make = create_make(&app, &random_name("Fiat")).await;
    let make_id = make["id"].as_str().unwrap();
    let model = create_model(&app, make_id, &random_name("Uno")).await;
    let color = create_color(&app, &random_name("Vermelho")).await;
    let category = create_category(&app, &random_name("PassCh")).await;
    let fuel_type = create_fuel_type(&app, &random_name("Eta")).await;

    let response = app
        .api
        .post("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "license_plate": "XYZ5678",
            "chassis_number": "SHORT",
            "renavam": "00891749802",
            "category_id": category["id"],
            "make_id": make["id"],
            "model_id": model["id"],
            "color_id": color["id"],
            "fuel_type_id": fuel_type["id"],
            "manufacture_year": 2024,
            "model_year": 2025,
            "acquisition_type": "PURCHASE",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================
// VEHICLE STATUS CHANGE TESTS
// ============================

#[tokio::test]
async fn test_change_vehicle_status() {
    let app = common::spawn_app().await;
    let make = create_make(&app, &random_name("VW")).await;
    let make_id = make["id"].as_str().unwrap();
    let model = create_model(&app, make_id, &random_name("Gol")).await;
    let color = create_color(&app, &random_name("Prata")).await;
    let category = create_category(&app, &random_name("UtilSt")).await;
    let fuel_type = create_fuel_type(&app, &random_name("Diesel")).await;

    let chassis = generate_chassis();
    let response = app
        .api
        .post("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "license_plate": "DEF4G56",
            "chassis_number": chassis,
            "renavam": "00891749802",
            "category_id": category["id"],
            "make_id": make["id"],
            "model_id": model["id"],
            "color_id": color["id"],
            "fuel_type_id": fuel_type["id"],
            "manufacture_year": 2023,
            "model_year": 2024,
            "acquisition_type": "PURCHASE",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let vehicle: Value = response.json();
    let vehicle_id = vehicle["id"].as_str().unwrap();

    // Change status to IN_MAINTENANCE
    let response = app
        .api
        .put(&format!("/api/admin/fleet/vehicles/{}/status", vehicle_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "status": "IN_MAINTENANCE",
            "reason": "Manutenção preventiva"
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let updated: Value = response.json();
    assert_eq!(updated["status"], "IN_MAINTENANCE");

    // Check status history
    let response = app
        .api
        .get(&format!("/api/admin/fleet/vehicles/{}/history", vehicle_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let history: Vec<Value> = response.json();
    assert!(history.len() >= 2); // Initial + change
}

// ============================
// VEHICLE SOFT DELETE TEST
// ============================

#[tokio::test]
async fn test_soft_delete_vehicle() {
    let app = common::spawn_app().await;
    let make = create_make(&app, &random_name("Renault")).await;
    let make_id = make["id"].as_str().unwrap();
    let model = create_model(&app, make_id, &random_name("Sandero")).await;
    let color = create_color(&app, &random_name("Branco")).await;
    let category = create_category(&app, &random_name("UtilDel")).await;
    let fuel_type = create_fuel_type(&app, &random_name("FlexDel")).await;

    let chassis = generate_chassis();
    let response = app
        .api
        .post("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "license_plate": "GHI7J89",
            "chassis_number": chassis,
            "renavam": "00891749802",
            "category_id": category["id"],
            "make_id": make["id"],
            "model_id": model["id"],
            "color_id": color["id"],
            "fuel_type_id": fuel_type["id"],
            "manufacture_year": 2022,
            "model_year": 2023,
            "acquisition_type": "DONATION",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let vehicle: Value = response.json();
    let vehicle_id = vehicle["id"].as_str().unwrap();

    // Soft delete
    let response = app
        .api
        .delete(&format!("/api/admin/fleet/vehicles/{}", vehicle_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Should not be found by default
    let response = app
        .api
        .get(&format!("/api/admin/fleet/vehicles/{}", vehicle_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// VEHICLE LIST & FILTER TESTS
// ============================

#[tokio::test]
async fn test_list_vehicles_with_filters() {
    let app = common::spawn_app().await;

    // List all vehicles
    let response = app
        .api
        .get("/api/admin/fleet/vehicles")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    assert!(list["total"].as_i64().is_some());
    assert!(list["vehicles"].as_array().is_some());
}

// ============================
// VEHICLE SEARCH TEST
// ============================

#[tokio::test]
async fn test_search_vehicles() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/fleet/vehicles/search?q=ABC&limit=5")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let result: Value = response.json();
    assert!(result["vehicles"].as_array().is_some());
}

// ============================
// LOOKUP TABLE TESTS
// ============================

#[tokio::test]
async fn test_crud_fuel_types() {
    let app = common::spawn_app().await;
    let name = random_name("Biodiesel");

    let response = app
        .api
        .post("/api/admin/fleet/fuel-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": &name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let created: Value = response.json();

    let response = app
        .api
        .get("/api/admin/fleet/fuel-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    assert!(list["total"].as_i64().unwrap() > 0);

    // Delete
    let response = app
        .api
        .delete(&format!("/api/admin/fleet/fuel-types/{}", created["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_crud_transmission_types() {
    let app = common::spawn_app().await;
    let name = random_name("Semi-Auto");

    let response = app
        .api
        .post("/api/admin/fleet/transmission-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": &name }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);

    let response = app
        .api
        .get("/api/admin/fleet/transmission-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_crud_colors() {
    let app = common::spawn_app().await;
    let name = random_name("Roxo");

    let response = app
        .api
        .post("/api/admin/fleet/colors")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": &name, "hex_code": "#800080" }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let created: Value = response.json();
    assert_eq!(created["hex_code"], "#800080");

    // Update
    let response = app
        .api
        .put(&format!("/api/admin/fleet/colors/{}", created["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "hex_code": "#9B30FF" }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let updated: Value = response.json();
    assert_eq!(updated["hex_code"], "#9B30FF");
}
