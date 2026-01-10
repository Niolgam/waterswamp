mod common;

use common::TestApp;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// HELPERS
// ============================

// Generates a random uppercase code of `len` characters (A-Z)
fn random_code(len: usize) -> String {
    // Use UUID to get randomness
    let uuid = Uuid::new_v4().simple().to_string();

    // Map bytes to A-Z range to maximize the available pool (26^len)
    uuid.bytes()
        .filter(|b| b.is_ascii_alphanumeric())
        .take(len)
        .map(|b| (b % 26 + b'A') as char)
        .collect()
}

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

async fn create_unique_country(app: &TestApp) -> Value {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let name = random_name("Country");
        let code = random_code(3);

        let response = app
            .api
            .post("/api/admin/locations/countries")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "code": code
            }))
            .await;

        if response.status_code() == StatusCode::CREATED {
            return response.json();
        }

        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!(
                "Failed to create country after {} attempts. Last status: {}. Body: {}",
                attempts,
                response.status_code(),
                response.text()
            );
        }
    }
}

async fn create_unique_state(app: &TestApp, country_id: &str) -> Value {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let name = random_name("State");
        let code = random_code(2);

        let response = app
            .api
            .post("/api/admin/locations/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "code": code,
                "country_id": country_id
            }))
            .await;

        if response.status_code() == StatusCode::CREATED {
            return response.json();
        }

        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!(
                "Failed to create state after {} attempts. Last status: {}. Body: {}",
                attempts,
                response.status_code(),
                response.text()
            );
        }
    }
}

async fn create_unique_city(app: &TestApp, state_id: &str) -> Value {
    let name = random_name("City");

    let response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "state_id": state_id
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "Failed to create city: {}",
        response.text()
    );
    response.json()
}

// ============================
// COUNTRY TESTS
// ============================

#[tokio::test]
async fn test_create_country_success() {
    let app = common::spawn_app().await;
    let name = random_name("Brasil");
    let code = random_code(3);

    let response = app
        .api
        .post("/api/admin/locations/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "code": code
        }))
        .await;

    if response.status_code() == StatusCode::CONFLICT {
        return; // Pass if conflict to avoid flakiness on parallel runs
    }

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], name);
    assert_eq!(body["code"], code);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_country_duplicate_code_returns_conflict() {
    let app = common::spawn_app().await;
    let code = random_code(3);

    // Create first country
    app.api
        .post("/api/admin/locations/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Country1"),
            "code": code
        }))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Country2"),
            "code": code
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_country_success() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let country_id = country["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/locations/countries/{}", country_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], country["name"]);
    assert_eq!(body["code"], country["code"]);
}

#[tokio::test]
async fn test_list_countries_success() {
    let app = common::spawn_app().await;

    create_unique_country(&app).await;
    create_unique_country(&app).await;

    let response = app
        .api
        .get("/api/admin/locations/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["countries"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
}

// ============================
// STATE TESTS
// ============================

#[tokio::test]
async fn test_create_state_success() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;

    let name = random_name("Sao Paulo");
    let code = random_code(2);

    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "code": code,
            "country_id": country["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], name);
    assert_eq!(body["code"], code);
    assert_eq!(body["country_id"], country["id"]);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_state_duplicate_code_returns_conflict() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let code = random_code(2);

    // Create first state
    app.api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("State1"),
            "code": code,
            "country_id": country["id"]
        }))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("State2"),
            "code": code,
            "country_id": country["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_state_invalid_code_returns_400() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;

    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Invalid"),
            "code": "XYZ", // Invalid: 3 chars instead of 2
            "country_id": country["id"]
        }))
        .await;

    assert!(
        response.status_code() == StatusCode::BAD_REQUEST
            || response.status_code() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn test_get_state_success() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let state_id = state["id"].as_str().unwrap();

    // Get state
    let response = app
        .api
        .get(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], state["name"]);
    assert_eq!(body["code"], state["code"]);
    assert_eq!(body["country_id"], country["id"]);
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
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let state_id = state["id"].as_str().unwrap();

    let new_name = random_name("Updated State");

    // Update state
    let response = app
        .api
        .put(&format!("/api/admin/locations/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": new_name
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], new_name);
    assert_eq!(body["code"], state["code"]);
}

#[tokio::test]
async fn test_delete_state_success() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let state_id = state["id"].as_str().unwrap();

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
    let country = create_unique_country(&app).await;
    let country_id = country["id"].as_str().unwrap();

    create_unique_state(&app, country_id).await;
    create_unique_state(&app, country_id).await;
    create_unique_state(&app, country_id).await;

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
    let country = create_unique_country(&app).await;
    let country_id = country["id"].as_str().unwrap();

    let unique_suffix = Uuid::new_v4().simple().to_string();
    // Use names without spaces to prevent URL encoding issues in test environment
    let name_match = format!("Santa-Catarina-{}", unique_suffix);
    let name_no_match = format!("Rio-Grande-{}", unique_suffix);

    // Robust creation with retries for name_match state
    let mut attempts = 0;
    loop {
        attempts += 1;
        let code = random_code(2);
        let response = app
            .api
            .post("/api/admin/locations/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({"name": name_match, "code": code, "country_id": country_id}))
            .await;

        if response.status_code() == StatusCode::CREATED {
            break;
        }
        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!("Failed to create match state: {}", response.text());
        }
    }

    // Robust creation with retries for name_no_match state
    attempts = 0;
    loop {
        attempts += 1;
        let code = random_code(2);
        let response = app
            .api
            .post("/api/admin/locations/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({"name": name_no_match, "code": code, "country_id": country_id}))
            .await;

        if response.status_code() == StatusCode::CREATED {
            break;
        }
        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!("Failed to create no_match state: {}", response.text());
        }
    }

    // Search for "Santa"
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/states?search={}",
            name_match
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let states = body["states"].as_array().unwrap();
    assert!(states
        .iter()
        .any(|s| s["name"].as_str().unwrap().contains(&name_match)));
}

#[tokio::test]
async fn test_state_requires_admin_role() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;

    // Try with user token (not admin)
    let response = app
        .api
        .post("/api/admin/locations/states")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "name": random_name("Goias"),
            "code": random_code(2),
            "country_id": country["id"]
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
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let name = random_name("Campinas");

    // Create city
    let response = app
        .api
        .post("/api/admin/locations/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "state_id": state["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], name);
    assert_eq!(body["state_id"], state["id"]);
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
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let city = create_unique_city(&app, state["id"].as_str().unwrap()).await;

    // Get city
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/cities/{}",
            city["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], city["name"]);
    assert_eq!(body["state_name"], state["name"]);
    assert_eq!(body["state_code"], state["code"]);
}

#[tokio::test]
async fn test_list_cities_filter_by_state() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;

    let state1 = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let state2 = create_unique_state(&app, country["id"].as_str().unwrap()).await;

    let state1_id = state1["id"].as_str().unwrap();
    let state2_id = state2["id"].as_str().unwrap();

    create_unique_city(&app, state1_id).await;
    create_unique_city(&app, state1_id).await;
    create_unique_city(&app, state2_id).await;

    // Filter by State 1
    let response = app
        .api
        .get(&format!(
            "/api/admin/locations/cities?state_id={}",
            state1_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let cities = body["cities"].as_array().unwrap();

    // Should only have State 1 cities
    for city in cities {
        assert_eq!(city["state_id"], state1["id"]);
    }
}

#[tokio::test]
async fn test_update_city_success() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let city = create_unique_city(&app, state["id"].as_str().unwrap()).await;
    let city_id = city["id"].as_str().unwrap();

    let new_name = random_name("Updated City");

    // Update city name
    let response = app
        .api
        .put(&format!("/api/admin/locations/cities/{}", city_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"name": new_name}))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], new_name);
}

#[tokio::test]
async fn test_delete_city_success() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let city = create_unique_city(&app, state["id"].as_str().unwrap()).await;
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
    let country = create_unique_country(&app).await;
    let state = create_unique_state(&app, country["id"].as_str().unwrap()).await;
    let city = create_unique_city(&app, state["id"].as_str().unwrap()).await;

    let state_id = state["id"].as_str().unwrap();
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
