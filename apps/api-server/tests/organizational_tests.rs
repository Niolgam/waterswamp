mod common;

use common::TestApp;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// HELPERS
// ============================

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

fn random_code() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    uuid.chars().take(8).collect()
}

fn random_numeric_code() -> i32 {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let num = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    (num % 900000 + 100000) as i32  // Range 100000-999999
}

async fn create_system_setting(app: &TestApp, key: &str, value: Value) -> Value {
    let response = app
        .api
        .post("/api/admin/organizational/settings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "key": key,
            "value": value,
            "value_type": "string",
            "description": "Test setting",
            "category": "test",
            "is_sensitive": false
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create system setting. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_organization(app: &TestApp) -> Value {
    let cnpj = format!("{:014}", rand::random::<u64>() % 100000000000000);

    let response = app
        .api
        .post("/api/admin/organizational/organizations")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "acronym": random_code(),
            "name": random_name("Organization"),
            "cnpj": cnpj,
            "ug_code": rand::random::<u32>() % 1000000,
            "siorg_code": rand::random::<i32>() % 1000000,
            "is_main_organization": false,
            "is_active": true
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create organization. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_unit_category(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/organizational/unit-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Category"),
            "description": "Test category",
            "is_active": true,
            "is_siorg_managed": false
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create unit category. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_unit_type(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/organizational/unit-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": random_name("Type"),
            "description": "Test type",
            "is_active": true,
            "is_siorg_managed": false
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create unit type. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_organizational_unit(
    app: &TestApp,
    organization_id: &str,
    parent_id: Option<&str>,
    category_id: &str,
    unit_type_id: &str,
) -> Value {
    let mut payload = json!({
        "organization_id": organization_id,
        "category_id": category_id,
        "unit_type_id": unit_type_id,
        "internal_type": "Sector",
        "name": random_name("Unit"),
        "activity_area": "Support",
        "is_active": true
    });

    if let Some(pid) = parent_id {
        payload["parent_id"] = json!(pid);
    }

    let response = app
        .api
        .post("/api/admin/organizational/units")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create organizational unit. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

// ============================
// SYSTEM SETTINGS TESTS
// ============================

#[tokio::test]
async fn test_create_system_setting_success() {
    let app = common::spawn_app().await;

    let key = format!("test.setting.{}", Uuid::new_v4().simple());
    let response = app
        .api
        .post("/api/admin/organizational/settings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "key": key,
            "value": json!("test value"),
            "value_type": "string",
            "description": "Test setting",
            "category": "test",
            "is_sensitive": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["key"], key);
    assert_eq!(body["value"], "test value");
}

#[tokio::test]
async fn test_list_system_settings() {
    let app = common::spawn_app().await;

    // Create a setting first
    let key = format!("test.list.{}", Uuid::new_v4().simple());
    create_system_setting(&app, &key, json!("test value")).await;

    let response = app
        .api
        .get("/api/admin/organizational/settings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["settings"].is_array());
    assert!(body["total"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_get_system_setting() {
    let app = common::spawn_app().await;

    let key = format!("test.get.{}", Uuid::new_v4().simple());
    create_system_setting(&app, &key, json!("test value")).await;

    let response = app
        .api
        .get(&format!("/api/admin/organizational/settings/{}", key))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["key"], key);
    assert_eq!(body["value"], "test value");
}

#[tokio::test]
async fn test_update_system_setting() {
    let app = common::spawn_app().await;

    let key = format!("test.update.{}", Uuid::new_v4().simple());
    create_system_setting(&app, &key, json!("initial value")).await;

    let response = app
        .api
        .put(&format!("/api/admin/organizational/settings/{}", key))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "value": json!("updated value"),
            "description": "Updated description"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["value"], "updated value");
}

#[tokio::test]
async fn test_delete_system_setting() {
    let app = common::spawn_app().await;

    let key = format!("test.delete.{}", Uuid::new_v4().simple());
    create_system_setting(&app, &key, json!("test value")).await;

    let response = app
        .api
        .delete(&format!("/api/admin/organizational/settings/{}", key))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let response = app
        .api
        .get(&format!("/api/admin/organizational/settings/{}", key))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// ORGANIZATION TESTS
// ============================

#[tokio::test]
async fn test_create_organization_success() {
    let app = common::spawn_app().await;

    let cnpj = format!("{:014}", rand::random::<u64>() % 100000000000000);
    let ug_code = random_numeric_code();
    let siorg_code = random_numeric_code();
    let response = app
        .api
        .post("/api/admin/organizational/organizations")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "acronym": random_code(),
            "name": random_name("Organization"),
            "cnpj": cnpj,
            "ug_code": ug_code,
            "siorg_code": siorg_code,
            "is_main_organization": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["cnpj"], cnpj);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_organization_duplicate_cnpj() {
    let app = common::spawn_app().await;

    let cnpj = format!("{:014}", rand::random::<u64>() % 100000000000000);

    // First creation should succeed
    app.api
        .post("/api/admin/organizational/organizations")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "acronym": random_code(),
            "name": random_name("Organization"),
            "cnpj": cnpj,
            "ug_code": 123456,
            "siorg_code": 789012,
            "is_main_organization": false,
            "is_active": true
        }))
        .await;

    // Second creation with same CNPJ should fail
    let response = app
        .api
        .post("/api/admin/organizational/organizations")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "acronym": random_code(),
            "name": random_name("Organization"),
            "cnpj": cnpj,
            "ug_code": 654321,
            "siorg_code": 210987,
            "is_main_organization": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_organizations() {
    let app = common::spawn_app().await;

    create_organization(&app).await;

    let response = app
        .api
        .get("/api/admin/organizational/organizations")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["organizations"].is_array());
    assert!(body["total"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_get_organization() {
    let app = common::spawn_app().await;

    let created = create_organization(&app).await;
    let org_id = created["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/organizational/organizations/{}", org_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], org_id);
}

#[tokio::test]
async fn test_update_organization() {
    let app = common::spawn_app().await;

    let created = create_organization(&app).await;
    let org_id = created["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/organizational/organizations/{}", org_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Organization Name"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Organization Name");
}

#[tokio::test]
async fn test_delete_organization() {
    let app = common::spawn_app().await;

    let created = create_organization(&app).await;
    let org_id = created["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/organizational/organizations/{}", org_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

// ============================
// UNIT CATEGORY TESTS
// ============================

#[tokio::test]
async fn test_create_unit_category_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/organizational/unit-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Category"),
            "description": "Test category",
            "is_active": true,
            "is_siorg_managed": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_list_unit_categories() {
    let app = common::spawn_app().await;

    create_unit_category(&app).await;

    let response = app
        .api
        .get("/api/admin/organizational/unit-categories")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["categories"].is_array());
    assert!(body["total"].as_i64().unwrap() > 0);
}

// ============================
// UNIT TYPE TESTS
// ============================

#[tokio::test]
async fn test_create_unit_type_success() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/organizational/unit-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": random_name("Type"),
            "description": "Test type",
            "is_active": true,
            "is_siorg_managed": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_list_unit_types() {
    let app = common::spawn_app().await;

    create_unit_type(&app).await;

    let response = app
        .api
        .get("/api/admin/organizational/unit-types")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["types"].is_array());
    assert!(body["total"].as_i64().unwrap() > 0);
}

// ============================
// ORGANIZATIONAL UNIT TESTS
// ============================

#[tokio::test]
async fn test_create_organizational_unit_success() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    let response = app
        .api
        .post("/api/admin/organizational/units")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "organization_id": organization["id"].as_str().unwrap(),
            "category_id": category["id"].as_str().unwrap(),
            "unit_type_id": unit_type["id"].as_str().unwrap(),
            "internal_type": "Sector",
            "name": random_name("Unit"),
            "activity_area": "Support",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert_eq!(body["level"], 1); // Root unit should have level 1
}

#[tokio::test]
async fn test_create_organizational_unit_with_parent() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    // Create parent unit
    let parent = create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    // Create child unit
    let response = app
        .api
        .post("/api/admin/organizational/units")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "organization_id": organization["id"].as_str().unwrap(),
            "parent_id": parent["id"].as_str().unwrap(),
            "category_id": category["id"].as_str().unwrap(),
            "unit_type_id": unit_type["id"].as_str().unwrap(),
            "internal_type": "Sector",
            "name": random_name("ChildUnit"),
            "activity_area": "Support",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["parent_id"], parent["id"]);
    assert_eq!(body["level"], 2); // Child should have level 2
}

#[tokio::test]
async fn test_list_organizational_units() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .get("/api/admin/organizational/units")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["units"].is_array());
    assert!(body["total"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_get_organizational_unit_tree() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    // Create parent and child for tree structure
    let parent = create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        Some(parent["id"].as_str().unwrap()),
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .get("/api/admin/organizational/units/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_get_organizational_unit_children() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    let parent = create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        Some(parent["id"].as_str().unwrap()),
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/organizational/units/{}/children",
            parent["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_update_organizational_unit() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    let unit = create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .put(&format!(
            "/api/admin/organizational/units/{}",
            unit["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Unit Name"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Unit Name");
}

#[tokio::test]
async fn test_deactivate_organizational_unit() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    let unit = create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .post(&format!(
            "/api/admin/organizational/units/{}/deactivate",
            unit["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!("Deactivation reason"))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_organizational_unit() {
    let app = common::spawn_app().await;

    let organization = create_organization(&app).await;
    let category = create_unit_category(&app).await;
    let unit_type = create_unit_type(&app).await;

    let unit = create_organizational_unit(
        &app,
        organization["id"].as_str().unwrap(),
        None,
        category["id"].as_str().unwrap(),
        unit_type["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .delete(&format!(
            "/api/admin/organizational/units/{}",
            unit["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}
