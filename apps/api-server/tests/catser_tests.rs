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

async fn create_unit(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/units-of-measure")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Unit"),
            "symbol": random_code(),
            "is_base_unit": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_section(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/sections")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Section"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_division(app: &TestApp, section_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/divisions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "section_id": section_id,
            "name": random_name("Division"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_group(app: &TestApp, division_id: Option<&str>) -> Value {
    let mut payload = json!({
        "code": random_code(),
        "name": random_name("Group"),
        "is_active": true
    });
    if let Some(did) = division_id {
        payload["division_id"] = json!(did);
    }

    let response = app
        .api
        .post("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_class(app: &TestApp, group_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": group_id,
            "code": random_code(),
            "name": random_name("Class"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_item(app: &TestApp, class_id: &str, unit_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class_id,
            "unit_of_measure_id": unit_id,
            "code": random_code(),
            "description": random_name("Item"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

/// Creates a full CATSER hierarchy: section → division → group → class
async fn create_hierarchy(app: &TestApp) -> (Value, Value, Value, Value) {
    let section = create_section(app).await;
    let division = create_division(app, section["id"].as_str().unwrap()).await;
    let group = create_group(app, Some(division["id"].as_str().unwrap())).await;
    let class = create_class(app, group["id"].as_str().unwrap()).await;
    (section, division, group, class)
}

// ============================
// CATSER SECTION TESTS
// ============================

#[tokio::test]
async fn test_section_create_returns_all_fields() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;

    assert!(section["id"].is_string());
    assert!(section["name"].is_string());
    assert_eq!(section["is_active"], true);
    assert!(section["verification_status"].is_string());
    assert!(section["created_at"].is_string());
    assert!(section["updated_at"].is_string());
}

#[tokio::test]
async fn test_section_get_with_details() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let id = section["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/sections/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], section["id"]);
    assert_eq!(body["division_count"], 0);
}

#[tokio::test]
async fn test_section_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/sections/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_section_update_name() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let id = section["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/sections/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Section",
            "is_active": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Section");
    assert_eq!(body["is_active"], false);
}

#[tokio::test]
async fn test_section_update_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/sections/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": "X" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_section_delete_and_verify_gone() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let id = section["id"].as_str().unwrap();

    let del = app
        .api
        .delete(&format!("/api/admin/catalog/catser/sections/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    let get = app
        .api
        .get(&format!("/api/admin/catalog/catser/sections/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_section_delete_with_divisions_returns_409() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let section_id = section["id"].as_str().unwrap();

    create_division(&app, section_id).await;

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catser/sections/{}", section_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_section_list_with_search() {
    let app = common::spawn_app().await;
    let unique = random_name("SearchableSection");

    app.api
        .post("/api/admin/catalog/catser/sections")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": &unique,
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/sections?search={}", &unique[..20]))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_section_list_filter_by_active() {
    let app = common::spawn_app().await;

    create_section(&app).await;

    let inactive = create_section(&app).await;
    app.api
        .put(&format!("/api/admin/catalog/catser/sections/{}", inactive["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "is_active": false }))
        .await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/sections?is_active=true")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let sections = body["data"].as_array().unwrap();
    for s in sections {
        assert_eq!(s["is_active"], true);
    }
}

#[tokio::test]
async fn test_section_division_count_increments() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let section_id = section["id"].as_str().unwrap();

    create_division(&app, section_id).await;
    create_division(&app, section_id).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/sections/{}", section_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let body: Value = response.json();
    assert_eq!(body["division_count"], 2);
}

#[tokio::test]
async fn test_section_tree_includes_divisions() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    create_division(&app, section["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/sections/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let tree: Value = response.json();
    assert!(tree.is_array());

    let our_section = tree.as_array().unwrap().iter().find(|s| s["id"] == section["id"]);
    assert!(our_section.is_some());
    let divisions = our_section.unwrap()["divisions"].as_array().unwrap();
    assert!(!divisions.is_empty());
}

// ============================
// CATSER DIVISION TESTS
// ============================

#[tokio::test]
async fn test_division_create_returns_details() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;

    assert!(division["id"].is_string());
    assert_eq!(division["section_id"], section["id"]);
    assert!(division["section_name"].is_string());
    assert!(division["verification_status"].is_string());
    assert_eq!(division["group_count"], 0);
}

#[tokio::test]
async fn test_division_get_with_details() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;
    let id = division["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/divisions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], division["id"]);
    assert!(body["section_name"].is_string());
    assert_eq!(body["group_count"], 0);
}

#[tokio::test]
async fn test_division_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/divisions/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_division_update_name() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;
    let id = division["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/divisions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": "Updated Division" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Division");
}

#[tokio::test]
async fn test_division_move_to_different_section() {
    let app = common::spawn_app().await;
    let section1 = create_section(&app).await;
    let section2 = create_section(&app).await;
    let division = create_division(&app, section1["id"].as_str().unwrap()).await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/divisions/{}", division["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "section_id": section2["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["section_id"], section2["id"]);
}

#[tokio::test]
async fn test_division_delete_with_groups_returns_409() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;
    let division_id = division["id"].as_str().unwrap();

    create_group(&app, Some(division_id)).await;

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catser/divisions/{}", division_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_division_delete_and_verify_gone() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;
    let id = division["id"].as_str().unwrap();

    let del = app
        .api
        .delete(&format!("/api/admin/catalog/catser/divisions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    let get = app
        .api
        .get(&format!("/api/admin/catalog/catser/divisions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_division_list_filter_by_section() {
    let app = common::spawn_app().await;
    let section1 = create_section(&app).await;
    let section2 = create_section(&app).await;
    let s1_id = section1["id"].as_str().unwrap();

    create_division(&app, s1_id).await;
    create_division(&app, s1_id).await;
    create_division(&app, section2["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/divisions?section_id={}", s1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let divisions = body["data"].as_array().unwrap();
    assert!(divisions.len() >= 2);
    for d in divisions {
        assert_eq!(d["section_id"].as_str().unwrap(), s1_id);
    }
}

#[tokio::test]
async fn test_division_group_count_increments() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;
    let division_id = division["id"].as_str().unwrap();

    assert_eq!(division["group_count"], 0);

    create_group(&app, Some(division_id)).await;
    create_group(&app, Some(division_id)).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/divisions/{}", division_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let body: Value = response.json();
    assert_eq!(body["group_count"], 2);
}

// ============================
// CATSER GROUP TESTS
// ============================

#[tokio::test]
async fn test_catser_group_create_returns_all_fields() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;
    let group = create_group(&app, Some(division["id"].as_str().unwrap())).await;

    assert!(group["id"].is_string());
    assert!(group["code"].is_string());
    assert!(group["name"].is_string());
    assert_eq!(group["is_active"], true);
    assert_eq!(group["division_id"], division["id"]);
    assert!(group["verification_status"].is_string());
    assert!(group["created_at"].is_string());
    assert!(group["updated_at"].is_string());
}

#[tokio::test]
async fn test_catser_group_create_without_division() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;

    assert!(group["id"].is_string());
    assert!(group["division_id"].is_null());
}

#[tokio::test]
async fn test_catser_group_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let code = random_code();

    // Create first group with known code
    app.api
        .post("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": random_name("G1"),
            "is_active": true
        }))
        .await;

    // Try duplicate
    let response = app
        .api
        .post("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": random_name("G2"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_catser_group_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/groups/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_catser_group_update_name_and_code() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let id = group["id"].as_str().unwrap();
    let new_code = random_code();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Group",
            "code": new_code,
            "is_active": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Group");
    assert_eq!(body["code"], new_code);
    assert_eq!(body["is_active"], false);
}

#[tokio::test]
async fn test_catser_group_update_assign_division() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let section = create_section(&app).await;
    let division = create_division(&app, section["id"].as_str().unwrap()).await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/groups/{}", group["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "division_id": division["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["division_id"], division["id"]);
}

#[tokio::test]
async fn test_catser_group_delete_and_verify_gone() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let id = group["id"].as_str().unwrap();

    let del = app
        .api
        .delete(&format!("/api/admin/catalog/catser/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    let get = app
        .api
        .get(&format!("/api/admin/catalog/catser/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_catser_group_list_with_search() {
    let app = common::spawn_app().await;
    let unique = random_name("SearchableGroup");

    app.api
        .post("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": &unique,
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/groups?search={}", &unique[..20]))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_catser_group_list_filter_by_division() {
    let app = common::spawn_app().await;
    let section = create_section(&app).await;
    let division1 = create_division(&app, section["id"].as_str().unwrap()).await;
    let division2 = create_division(&app, section["id"].as_str().unwrap()).await;
    let d1_id = division1["id"].as_str().unwrap();

    create_group(&app, Some(d1_id)).await;
    create_group(&app, Some(d1_id)).await;
    create_group(&app, Some(division2["id"].as_str().unwrap())).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/groups?division_id={}", d1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let groups = body["data"].as_array().unwrap();
    assert!(groups.len() >= 2);
    for g in groups {
        assert_eq!(g["division_id"].as_str().unwrap(), d1_id);
    }
}

#[tokio::test]
async fn test_catser_group_list_filter_by_active() {
    let app = common::spawn_app().await;

    create_group(&app, None).await;

    let inactive = create_group(&app, None).await;
    app.api
        .put(&format!("/api/admin/catalog/catser/groups/{}", inactive["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "is_active": false }))
        .await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/groups?is_active=true")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let groups = body["data"].as_array().unwrap();
    for g in groups {
        assert_eq!(g["is_active"], true);
    }
}

#[tokio::test]
async fn test_catser_group_tree_includes_classes() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    create_class(&app, group["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/groups/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let tree: Value = response.json();
    assert!(tree.is_array());

    let our_group = tree.as_array().unwrap().iter().find(|g| g["id"] == group["id"]);
    assert!(our_group.is_some());
    let classes = our_group.unwrap()["classes"].as_array().unwrap();
    assert!(!classes.is_empty());
}

// ============================
// CATSER CLASS TESTS
// ============================

#[tokio::test]
async fn test_catser_class_create_returns_details() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;

    assert!(class["id"].is_string());
    assert_eq!(class["group_id"], group["id"]);
    assert!(class["group_name"].is_string());
    assert!(class["group_code"].is_string());
    assert!(class["verification_status"].is_string());
    assert_eq!(class["item_count"], 0);
}

#[tokio::test]
async fn test_catser_class_get_with_details() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], class["id"]);
    assert!(body["group_name"].is_string());
    assert!(body["group_code"].is_string());
    assert_eq!(body["item_count"], 0);
}

#[tokio::test]
async fn test_catser_class_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/classes/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_catser_class_update_success() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": "Updated Class Name" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Class Name");
}

#[tokio::test]
async fn test_catser_class_move_to_different_group() {
    let app = common::spawn_app().await;
    let group1 = create_group(&app, None).await;
    let group2 = create_group(&app, None).await;
    let class = create_class(&app, group1["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "group_id": group2["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["group_id"], group2["id"]);
}

#[tokio::test]
async fn test_catser_class_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let group = create_group(&app, None).await;
    let gid = group["id"].as_str().unwrap();
    let code = random_code();

    app.api
        .post("/api/admin/catalog/catser/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": gid,
            "code": code,
            "name": random_name("C1"),
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .post("/api/admin/catalog/catser/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": gid,
            "code": code,
            "name": random_name("C2"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_catser_class_delete_with_items_returns_409() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app, None).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let class_id = class["id"].as_str().unwrap();

    create_item(&app, class_id, unit["id"].as_str().unwrap()).await;

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catser/classes/{}", class_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_catser_class_list_filter_by_group() {
    let app = common::spawn_app().await;
    let group1 = create_group(&app, None).await;
    let group2 = create_group(&app, None).await;
    let g1_id = group1["id"].as_str().unwrap();

    create_class(&app, g1_id).await;
    create_class(&app, g1_id).await;
    create_class(&app, group2["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/classes?group_id={}", g1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let classes = body["data"].as_array().unwrap();
    assert!(classes.len() >= 2);
    for c in classes {
        assert_eq!(c["group_id"].as_str().unwrap(), g1_id);
    }
}

#[tokio::test]
async fn test_catser_class_item_count_increments() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app, None).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let class_id = class["id"].as_str().unwrap();
    let unit_id = unit["id"].as_str().unwrap();

    assert_eq!(class["item_count"], 0);

    create_item(&app, class_id, unit_id).await;
    create_item(&app, class_id, unit_id).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/classes/{}", class_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let body: Value = response.json();
    assert_eq!(body["item_count"], 2);
}

// ============================
// CATSER ITEM TESTS
// ============================

#[tokio::test]
async fn test_catser_item_create_returns_details() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, group, class) = create_hierarchy(&app).await;

    let item = create_item(&app, class["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;

    assert!(item["id"].is_string());
    assert_eq!(item["class_id"], class["id"]);
    assert!(item["class_name"].is_string());
    assert!(item["class_code"].is_string());
    assert_eq!(item["group_id"], group["id"]);
    assert!(item["group_name"].is_string());
    assert_eq!(item["unit_of_measure_id"], unit["id"]);
    assert!(item["unit_name"].is_string());
    assert!(item["unit_symbol"].is_string());
    assert!(item["verification_status"].is_string());
}

#[tokio::test]
async fn test_catser_item_create_with_optional_fields() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, _, class) = create_hierarchy(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class["id"],
            "unit_of_measure_id": unit["id"],
            "code": random_code(),
            "description": "Item with extras",
            "code_cpc": "CPC12345",
            "supplementary_description": "Extra details",
            "specification": "Spec v1",
            "search_links": "link1;link2",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["code_cpc"], "CPC12345");
    assert_eq!(body["supplementary_description"], "Extra details");
    assert_eq!(body["specification"], "Spec v1");
    assert_eq!(body["search_links"], "link1;link2");
}

#[tokio::test]
async fn test_catser_item_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/items/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_catser_item_update_description() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, _, class) = create_hierarchy(&app).await;
    let item = create_item(&app, class["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;
    let id = item["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/items/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "description": "Updated Description",
            "code_cpc": "CPC99999",
            "specification": "New Spec"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["description"], "Updated Description");
    assert_eq!(body["code_cpc"], "CPC99999");
    assert_eq!(body["specification"], "New Spec");
}

#[tokio::test]
async fn test_catser_item_move_to_different_class() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app, None).await;
    let gid = group["id"].as_str().unwrap();
    let class1 = create_class(&app, gid).await;
    let class2 = create_class(&app, gid).await;
    let item = create_item(&app, class1["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catser/items/{}", item["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "class_id": class2["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["class_id"], class2["id"]);
}

#[tokio::test]
async fn test_catser_item_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, _, class) = create_hierarchy(&app).await;
    let class_id = class["id"].as_str().unwrap();
    let unit_id = unit["id"].as_str().unwrap();
    let code = random_code();

    app.api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class_id,
            "unit_of_measure_id": unit_id,
            "code": code,
            "description": "First",
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class_id,
            "unit_of_measure_id": unit_id,
            "code": code,
            "description": "Second",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_catser_item_delete_and_verify_gone() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, _, class) = create_hierarchy(&app).await;
    let item = create_item(&app, class["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;
    let id = item["id"].as_str().unwrap();

    let del = app
        .api
        .delete(&format!("/api/admin/catalog/catser/items/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    let get = app
        .api
        .get(&format!("/api/admin/catalog/catser/items/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_catser_item_list_filter_by_class() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app, None).await;
    let gid = group["id"].as_str().unwrap();
    let class1 = create_class(&app, gid).await;
    let class2 = create_class(&app, gid).await;
    let c1_id = class1["id"].as_str().unwrap();
    let uid = unit["id"].as_str().unwrap();

    create_item(&app, c1_id, uid).await;
    create_item(&app, c1_id, uid).await;
    create_item(&app, class2["id"].as_str().unwrap(), uid).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catser/items?class_id={}", c1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["data"].as_array().unwrap();
    assert!(items.len() >= 2);
    for i in items {
        assert_eq!(i["class_id"].as_str().unwrap(), c1_id);
    }
}

#[tokio::test]
async fn test_catser_item_create_with_invalid_class_returns_404() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": Uuid::new_v4(),
            "unit_of_measure_id": unit["id"],
            "code": random_code(),
            "description": "Orphan",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_catser_item_create_with_invalid_unit_returns_404() {
    let app = common::spawn_app().await;
    let (_, _, _, class) = create_hierarchy(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class["id"],
            "unit_of_measure_id": Uuid::new_v4(),
            "code": random_code(),
            "description": "Bad Unit",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// CATSER PAGINATION TESTS
// ============================

#[tokio::test]
async fn test_section_list_pagination() {
    let app = common::spawn_app().await;

    for _ in 0..3 {
        create_section(&app).await;
    }

    let response = app
        .api
        .get("/api/admin/catalog/catser/sections?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["data"].as_array().unwrap().len() <= 2);
    assert!(body["total"].as_i64().unwrap() >= 3);
    assert_eq!(body["limit"], 2);
    assert_eq!(body["offset"], 0);
}

#[tokio::test]
async fn test_catser_item_list_pagination() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app, None).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let class_id = class["id"].as_str().unwrap();
    let unit_id = unit["id"].as_str().unwrap();

    for _ in 0..3 {
        create_item(&app, class_id, unit_id).await;
    }

    let response = app
        .api
        .get("/api/admin/catalog/catser/items?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["data"].as_array().unwrap().len() <= 2);
    assert!(body["total"].as_i64().unwrap() >= 3);
    assert_eq!(body["limit"], 2);
    assert_eq!(body["offset"], 0);
}

// ============================
// CATSER AUTHORIZATION TESTS
// ============================

#[tokio::test]
async fn test_catser_sections_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let post = app
        .api
        .post("/api/admin/catalog/catser/sections")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({ "name": "Y", "is_active": true }))
        .await;
    assert_eq!(post.status_code(), StatusCode::FORBIDDEN);

    let get = app
        .api
        .get("/api/admin/catalog/catser/sections")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catser_divisions_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/divisions")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catser_groups_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catser_classes_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/classes")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catser_items_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catser_unauthenticated_returns_401() {
    let app = common::spawn_app().await;

    let response = app.api.get("/api/admin/catalog/catser/sections").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
