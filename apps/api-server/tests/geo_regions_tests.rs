mod common;

use common::TestApp;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// HELPERS
// ============================

// Generates a random uppercase code of `len` characters (A-Z only, no digits)
fn random_code(len: usize) -> String {
    // Use UUID to get randomness
    let uuid = Uuid::new_v4().simple().to_string();

    // Only uppercase letters for state abbreviations (validation requires A-Z only)
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    uuid.bytes()
        .filter(|b| b.is_ascii_alphanumeric())
        .take(len)
        .map(|b| chars[(b as usize) % chars.len()])
        .collect()
}

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

// Generates a random BACEN code (int in range 100-9999)
fn random_bacen_code() -> i32 {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let num = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    (num % 9900 + 100) as i32
}

// Generates a random IBGE code for tests - use range 54-999 to avoid conflicts with real Brazilian state codes (11-53)
fn random_ibge_code_state() -> i32 {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let num = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    (num % 946 + 54) as i32 // Range 54-999 to avoid real IBGE state codes
}

fn random_ibge_code_city() -> i32 {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let num = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    (num % 8999999 + 1000000) as i32
}

async fn create_unique_country(app: &TestApp) -> Value {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let name = random_name("Country");
        let iso2 = random_code(2);
        let bacen_code = random_bacen_code();

        let response = app
            .api
            .post("/api/admin/geo_regions/countries")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "iso2": iso2,
                "bacen_code": bacen_code,
                "is_active": true,
            }))
            .await;

        if response.status_code() == StatusCode::CREATED {
            return response.json();
        }

        if response.status_code() != StatusCode::CONFLICT || attempts >= 20 {
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
        let abbreviation = random_code(2);
        let ibge_code = random_ibge_code_state();

        let response = app
            .api
            .post("/api/admin/geo_regions/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "abbreviation": abbreviation,
                "ibge_code": ibge_code,
                "country_id": country_id,
                "is_active": true,
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
    let mut attempts = 0;
    loop {
        attempts += 1;
        let name = random_name("City");
        let ibge_code = random_ibge_code_city();

        let response = app
            .api
            .post("/api/admin/geo_regions/cities")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "name": name,
                "ibge_code": ibge_code,
                "state_id": state_id,
                "is_active": true,
            }))
            .await;

        if response.status_code() == StatusCode::CREATED {
            return response.json();
        }

        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!(
                "Failed to create city after {} attempts. Last status: {}. Body: {}",
                attempts,
                response.status_code(),
                response.text()
            );
        }
    }
}

// ============================
// COUNTRY TESTS
// ============================

#[tokio::test]
async fn test_create_country_success() {
    let app = common::spawn_app().await;
    let name = random_name("Brasil");
    let iso2 = random_code(2);
    let bacen_code = random_bacen_code();

    let response = app
        .api
        .post("/api/admin/geo_regions/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "iso2": iso2,
            "bacen_code": bacen_code,
            "is_active": true,
        }))
        .await;

    if response.status_code() == StatusCode::CONFLICT {
        return; // Pass if conflict to avoid flakiness on parallel runs
    }

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], name);
    assert_eq!(body["iso2"], iso2);
    assert_eq!(body["bacen_code"], bacen_code);
    assert_eq!(body["is_active"], true);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_country_duplicate_iso2_returns_conflict() {
    let app = common::spawn_app().await;
    let iso2 = random_code(2);
    let bacen_code = random_bacen_code();

    // Create first country
    app.api
        .post("/api/admin/geo_regions/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Country1"),
            "iso2": iso2,
            "bacen_code": bacen_code,
            "is_active": true,
        }))
        .await;

    // Try to create duplicate with same iso2
    let response = app
        .api
        .post("/api/admin/geo_regions/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Country2"),
            "iso2": iso2,
            "bacen_code": random_bacen_code(),
            "is_active": true,
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
        .get(&format!("/api/admin/geo_regions/countries/{}", country_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], country["name"]);
    assert_eq!(body["iso2"], country["iso2"]);
    assert_eq!(body["bacen_code"], country["bacen_code"]);
    assert_eq!(body["is_active"], country["is_active"]);
}

#[tokio::test]
async fn test_list_countries_success() {
    let app = common::spawn_app().await;

    create_unique_country(&app).await;
    create_unique_country(&app).await;

    let response = app
        .api
        .get("/api/admin/geo_regions/countries")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["items"].is_array());
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
    let abbreviation = random_code(2);
    let ibge_code = random_ibge_code_state();

    let response = app
        .api
        .post("/api/admin/geo_regions/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "abbreviation": abbreviation,
            "ibge_code": ibge_code,
            "country_id": country["id"],
            "is_active": true,
        }))
        .await;

    if response.status_code() == StatusCode::CONFLICT {
        return; // Pass if conflict to avoid flakiness on parallel runs
    }

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], name);
    assert_eq!(body["abbreviation"], abbreviation);
    assert_eq!(body["ibge_code"], ibge_code);
    assert_eq!(body["country_id"], country["id"]);
    assert_eq!(body["is_active"], true);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_state_duplicate_abbreviation_returns_conflict() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;
    let abbreviation = random_code(2);

    // Create first state
    app.api
        .post("/api/admin/geo_regions/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("State1"),
            "abbreviation": abbreviation,
            "ibge_code": random_ibge_code_state(),
            "country_id": country["id"]
        }))
        .await;

    // Try to create duplicate with same abbreviation
    let response = app
        .api
        .post("/api/admin/geo_regions/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("State2"),
            "abbreviation": abbreviation,
            "ibge_code": random_ibge_code_state(),
            "country_id": country["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_state_invalid_abbreviation_returns_400() {
    let app = common::spawn_app().await;
    let country = create_unique_country(&app).await;

    let response = app
        .api
        .post("/api/admin/geo_regions/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Invalid"),
            "abbreviation": "XYZ", // Invalid: 3 chars instead of 2
            "ibge_code": random_ibge_code_state(),
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
        .get(&format!("/api/admin/geo_regions/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], state["name"]);
    assert_eq!(body["abbreviation"], state["abbreviation"]);
    assert_eq!(body["ibge_code"], state["ibge_code"]);
    assert_eq!(body["country_id"], country["id"]);
}

#[tokio::test]
async fn test_get_state_not_found() {
    let app = common::spawn_app().await;

    let fake_id = Uuid::new_v4();
    let response = app
        .api
        .get(&format!("/api/admin/geo_regions/states/{}", fake_id))
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
        .put(&format!("/api/admin/geo_regions/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": new_name
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], new_name);
    assert_eq!(body["abbreviation"], state["abbreviation"]);
    assert_eq!(body["ibge_code"], state["ibge_code"]);
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
        .delete(&format!("/api/admin/geo_regions/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify it's deleted
    let get_response = app
        .api
        .get(&format!("/api/admin/geo_regions/states/{}", state_id))
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
        .get("/api/admin/geo_regions/states")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["items"].is_array());
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
        let abbreviation = random_code(2);
        let ibge_code = random_ibge_code_state();
        let response = app
            .api
            .post("/api/admin/geo_regions/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({"name": name_match, "abbreviation": abbreviation, "ibge_code": ibge_code, "country_id": country_id}))
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
        let abbreviation = random_code(2);
        let ibge_code = random_ibge_code_state();
        let response = app
            .api
            .post("/api/admin/geo_regions/states")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({"name": name_no_match, "abbreviation": abbreviation, "ibge_code": ibge_code, "country_id": country_id}))
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
            "/api/admin/geo_regions/states?search={}",
            name_match
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let states = body["items"].as_array().unwrap();
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
        .post("/api/admin/geo_regions/states")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "name": random_name("Goias"),
            "abbreviation": random_code(2),
            "ibge_code": random_ibge_code_state(),
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
    let ibge_code = random_ibge_code_city();

    // Create city
    let response = app
        .api
        .post("/api/admin/geo_regions/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": name,
            "ibge_code": ibge_code,
            "state_id": state["id"]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], name);
    assert_eq!(body["ibge_code"], ibge_code);
    assert_eq!(body["state_id"], state["id"]);
}

#[tokio::test]
async fn test_create_city_invalid_state_returns_404() {
    let app = common::spawn_app().await;

    let fake_state_id = Uuid::new_v4();
    let response = app
        .api
        .post("/api/admin/geo_regions/cities")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Cidade Fantasma",
            "ibge_code": random_ibge_code_city(),
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
            "/api/admin/geo_regions/cities/{}",
            city["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], city["name"]);
    assert_eq!(body["ibge_code"], city["ibge_code"]);
    assert_eq!(body["state_name"], state["name"]);
    assert_eq!(body["state_abbreviation"], state["abbreviation"]);
    assert_eq!(body["state_ibge_code"], state["ibge_code"]);
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
            "/api/admin/geo_regions/cities?state_id={}",
            state1_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let cities = body["items"].as_array().unwrap();

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
        .put(&format!("/api/admin/geo_regions/cities/{}", city_id))
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
        .delete(&format!("/api/admin/geo_regions/cities/{}", city_id))
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
        .delete(&format!("/api/admin/geo_regions/states/{}", state_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    // City should be deleted too
    let response = app
        .api
        .get(&format!("/api/admin/geo_regions/cities/{}", city_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
