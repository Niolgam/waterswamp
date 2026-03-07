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

async fn create_group(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": random_name("Group"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_group_with_code(app: &TestApp, code: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": random_name("Group"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_class(app: &TestApp, group_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/classes")
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

async fn create_pdm(app: &TestApp, class_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/pdms")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class_id,
            "code": random_code(),
            "description": random_name("PDM"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_item(app: &TestApp, pdm_id: &str, unit_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": pdm_id,
            "unit_of_measure_id": unit_id,
            "code": random_code(),
            "description": random_name("Item"),
            "is_sustainable": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

/// Creates a full CATMAT hierarchy: group → class → pdm, returns (group, class, pdm)
async fn create_hierarchy(app: &TestApp) -> (Value, Value, Value) {
    let group = create_group(app).await;
    let class = create_class(app, group["id"].as_str().unwrap()).await;
    let pdm = create_pdm(app, class["id"].as_str().unwrap()).await;
    (group, class, pdm)
}

// ============================
// CATMAT GROUP TESTS
// ============================

#[tokio::test]
async fn test_group_create_returns_all_fields() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;

    assert!(group["id"].is_string());
    assert!(group["code"].is_string());
    assert!(group["name"].is_string());
    assert_eq!(group["is_active"], true);
    assert!(group["verification_status"].is_string());
    assert!(group["created_at"].is_string());
    assert!(group["updated_at"].is_string());
}

#[tokio::test]
async fn test_group_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let code = random_code();
    create_group_with_code(&app, &code).await;

    let response = app
        .api
        .post("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": random_name("Dup"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_group_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/groups/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_group_update_name_and_code() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let id = group["id"].as_str().unwrap();
    let new_code = random_code();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Name",
            "code": new_code,
            "is_active": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Name");
    assert_eq!(body["code"], new_code);
    assert_eq!(body["is_active"], false);
}

#[tokio::test]
async fn test_group_update_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/groups/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": "X" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_group_delete_and_verify_gone() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let id = group["id"].as_str().unwrap();

    let del = app
        .api
        .delete(&format!("/api/admin/catalog/catmat/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    let get = app
        .api
        .get(&format!("/api/admin/catalog/catmat/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_group_list_with_search() {
    let app = common::spawn_app().await;
    let unique = random_name("SearchableGroup");

    app.api
        .post("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": &unique,
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/groups?search={}", &unique[..20]))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_group_list_filter_by_active() {
    let app = common::spawn_app().await;

    // Create active and inactive groups
    create_group(&app).await;

    let inactive = create_group(&app).await;
    app.api
        .put(&format!("/api/admin/catalog/catmat/groups/{}", inactive["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "is_active": false }))
        .await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/groups?is_active=true")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let groups = body["groups"].as_array().unwrap();
    for g in groups {
        assert_eq!(g["is_active"], true);
    }
}

#[tokio::test]
async fn test_group_tree_includes_classes() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    create_class(&app, group["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/groups/tree")
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
// CATMAT CLASS TESTS
// ============================

#[tokio::test]
async fn test_class_create_returns_details() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;

    assert!(class["id"].is_string());
    assert_eq!(class["group_id"], group["id"]);
    assert!(class["group_name"].is_string());
    assert!(class["group_code"].is_string());
    assert!(class["verification_status"].is_string());
    assert_eq!(class["pdm_count"], 0);
}

#[tokio::test]
async fn test_class_get_with_details() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], class["id"]);
    assert!(body["group_name"].is_string());
    assert!(body["group_code"].is_string());
    assert_eq!(body["pdm_count"], 0);
}

#[tokio::test]
async fn test_class_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/classes/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_class_update_success() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": "Updated Class Name" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Class Name");
}

#[tokio::test]
async fn test_class_update_move_to_different_group() {
    let app = common::spawn_app().await;
    let group1 = create_group(&app).await;
    let group2 = create_group(&app).await;
    let class = create_class(&app, group1["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "group_id": group2["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["group_id"], group2["id"]);
}

#[tokio::test]
async fn test_class_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let gid = group["id"].as_str().unwrap();
    let code = random_code();

    // Create first class with known code
    app.api
        .post("/api/admin/catalog/catmat/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": gid,
            "code": code,
            "name": random_name("C1"),
            "is_active": true
        }))
        .await;

    // Try duplicate
    let response = app
        .api
        .post("/api/admin/catalog/catmat/classes")
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
async fn test_class_delete_with_pdms_returns_409() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let class_id = class["id"].as_str().unwrap();

    // Create a PDM under this class
    create_pdm(&app, class_id).await;

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catmat/classes/{}", class_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_class_list_filter_by_group() {
    let app = common::spawn_app().await;
    let group1 = create_group(&app).await;
    let group2 = create_group(&app).await;
    let g1_id = group1["id"].as_str().unwrap();
    let g2_id = group2["id"].as_str().unwrap();

    create_class(&app, g1_id).await;
    create_class(&app, g1_id).await;
    create_class(&app, g2_id).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/classes?group_id={}", g1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let classes = body["classes"].as_array().unwrap();
    assert!(classes.len() >= 2);
    for c in classes {
        assert_eq!(c["group_id"].as_str().unwrap(), g1_id);
    }
}

#[tokio::test]
async fn test_class_pdm_count_increments() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let class_id = class["id"].as_str().unwrap();

    assert_eq!(class["pdm_count"], 0);

    create_pdm(&app, class_id).await;
    create_pdm(&app, class_id).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/classes/{}", class_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let body: Value = response.json();
    assert_eq!(body["pdm_count"], 2);
}

// ============================
// CATMAT PDM TESTS
// ============================

#[tokio::test]
async fn test_pdm_create_returns_details() {
    let app = common::spawn_app().await;
    let (group, class, pdm) = create_hierarchy(&app).await;

    assert!(pdm["id"].is_string());
    assert_eq!(pdm["class_id"], class["id"]);
    assert!(pdm["class_name"].is_string());
    assert!(pdm["class_code"].is_string());
    assert_eq!(pdm["group_id"], group["id"]);
    assert!(pdm["group_name"].is_string());
    assert!(pdm["verification_status"].is_string());
    assert_eq!(pdm["item_count"], 0);
}

#[tokio::test]
async fn test_pdm_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/pdms/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_pdm_update_description() {
    let app = common::spawn_app().await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let id = pdm["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/pdms/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "description": "Updated PDM Description" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["description"], "Updated PDM Description");
}

#[tokio::test]
async fn test_pdm_update_move_to_different_class() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let gid = group["id"].as_str().unwrap();
    let class1 = create_class(&app, gid).await;
    let class2 = create_class(&app, gid).await;
    let pdm = create_pdm(&app, class1["id"].as_str().unwrap()).await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/pdms/{}", pdm["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "class_id": class2["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["class_id"], class2["id"]);
}

#[tokio::test]
async fn test_pdm_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let cid = class["id"].as_str().unwrap();
    let code = random_code();

    app.api
        .post("/api/admin/catalog/catmat/pdms")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": cid,
            "code": code,
            "description": "First",
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .post("/api/admin/catalog/catmat/pdms")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": cid,
            "code": code,
            "description": "Second",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_pdm_delete_with_items_returns_409() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let pdm_id = pdm["id"].as_str().unwrap();

    create_item(&app, pdm_id, unit["id"].as_str().unwrap()).await;

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catmat/pdms/{}", pdm_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_pdm_list_filter_by_class() {
    let app = common::spawn_app().await;
    let group = create_group(&app).await;
    let gid = group["id"].as_str().unwrap();
    let class1 = create_class(&app, gid).await;
    let class2 = create_class(&app, gid).await;
    let c1_id = class1["id"].as_str().unwrap();

    create_pdm(&app, c1_id).await;
    create_pdm(&app, c1_id).await;
    create_pdm(&app, class2["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/pdms?class_id={}", c1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let pdms = body["pdms"].as_array().unwrap();
    assert!(pdms.len() >= 2);
    for p in pdms {
        assert_eq!(p["class_id"].as_str().unwrap(), c1_id);
    }
}

#[tokio::test]
async fn test_pdm_item_count_increments() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let pdm_id = pdm["id"].as_str().unwrap();
    let unit_id = unit["id"].as_str().unwrap();

    assert_eq!(pdm["item_count"], 0);

    create_item(&app, pdm_id, unit_id).await;
    create_item(&app, pdm_id, unit_id).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/pdms/{}", pdm_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    let body: Value = response.json();
    assert_eq!(body["item_count"], 2);
}

// ============================
// CATMAT ITEM TESTS
// ============================

#[tokio::test]
async fn test_item_create_returns_details() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (group, class, pdm) = create_hierarchy(&app).await;

    let item = create_item(&app, pdm["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;

    assert!(item["id"].is_string());
    assert_eq!(item["pdm_id"], pdm["id"]);
    assert!(item["pdm_description"].is_string());
    assert!(item["pdm_code"].is_string());
    assert_eq!(item["class_id"], class["id"]);
    assert!(item["class_name"].is_string());
    assert_eq!(item["group_id"], group["id"]);
    assert!(item["group_name"].is_string());
    assert_eq!(item["unit_of_measure_id"], unit["id"]);
    assert!(item["unit_name"].is_string());
    assert!(item["unit_symbol"].is_string());
    assert!(item["verification_status"].is_string());
    assert_eq!(item["is_sustainable"], false);
}

#[tokio::test]
async fn test_item_create_with_optional_fields() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": pdm["id"],
            "unit_of_measure_id": unit["id"],
            "code": random_code(),
            "description": "Item with NCM",
            "is_sustainable": true,
            "code_ncm": "12345678",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["is_sustainable"], true);
    assert_eq!(body["code_ncm"], "12345678");
}

#[tokio::test]
async fn test_item_get_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/items/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_item_update_description_and_sustainable() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let item = create_item(&app, pdm["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;
    let id = item["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/items/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "description": "Updated Description",
            "is_sustainable": true,
            "code_ncm": "99887766"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["description"], "Updated Description");
    assert_eq!(body["is_sustainable"], true);
    assert_eq!(body["code_ncm"], "99887766");
}

#[tokio::test]
async fn test_item_update_move_to_different_pdm() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let cid = class["id"].as_str().unwrap();
    let pdm1 = create_pdm(&app, cid).await;
    let pdm2 = create_pdm(&app, cid).await;
    let item = create_item(&app, pdm1["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/items/{}", item["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "pdm_id": pdm2["id"] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["pdm_id"], pdm2["id"]);
}

#[tokio::test]
async fn test_item_duplicate_code_returns_409() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let pdm_id = pdm["id"].as_str().unwrap();
    let unit_id = unit["id"].as_str().unwrap();
    let code = random_code();

    app.api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": pdm_id,
            "unit_of_measure_id": unit_id,
            "code": code,
            "description": "First",
            "is_sustainable": false,
            "is_active": true
        }))
        .await;

    let response = app
        .api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": pdm_id,
            "unit_of_measure_id": unit_id,
            "code": code,
            "description": "Second",
            "is_sustainable": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_item_delete_and_verify_gone() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let item = create_item(&app, pdm["id"].as_str().unwrap(), unit["id"].as_str().unwrap()).await;
    let id = item["id"].as_str().unwrap();

    let del = app
        .api
        .delete(&format!("/api/admin/catalog/catmat/items/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(del.status_code(), StatusCode::NO_CONTENT);

    let get = app
        .api
        .get(&format!("/api/admin/catalog/catmat/items/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_item_list_filter_by_pdm() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let group = create_group(&app).await;
    let class = create_class(&app, group["id"].as_str().unwrap()).await;
    let cid = class["id"].as_str().unwrap();
    let pdm1 = create_pdm(&app, cid).await;
    let pdm2 = create_pdm(&app, cid).await;
    let p1_id = pdm1["id"].as_str().unwrap();
    let uid = unit["id"].as_str().unwrap();

    create_item(&app, p1_id, uid).await;
    create_item(&app, p1_id, uid).await;
    create_item(&app, pdm2["id"].as_str().unwrap(), uid).await;

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/items?pdm_id={}", p1_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().unwrap();
    assert!(items.len() >= 2);
    for i in items {
        assert_eq!(i["pdm_id"].as_str().unwrap(), p1_id);
    }
}

#[tokio::test]
async fn test_item_list_filter_by_sustainable() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;
    let (_, _, pdm) = create_hierarchy(&app).await;
    let pdm_id = pdm["id"].as_str().unwrap();
    let uid = unit["id"].as_str().unwrap();

    // Create sustainable item
    app.api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": pdm_id,
            "unit_of_measure_id": uid,
            "code": random_code(),
            "description": "Sustainable",
            "is_sustainable": true,
            "is_active": true
        }))
        .await;

    // Create non-sustainable item
    create_item(&app, pdm_id, uid).await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/items?is_sustainable=true")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().unwrap();
    for i in items {
        assert_eq!(i["is_sustainable"], true);
    }
}

#[tokio::test]
async fn test_item_create_with_invalid_pdm_returns_404() {
    let app = common::spawn_app().await;
    let unit = create_unit(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": Uuid::new_v4(),
            "unit_of_measure_id": unit["id"],
            "code": random_code(),
            "description": "Orphan",
            "is_sustainable": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_item_create_with_invalid_unit_returns_404() {
    let app = common::spawn_app().await;
    let (_, _, pdm) = create_hierarchy(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "pdm_id": pdm["id"],
            "unit_of_measure_id": Uuid::new_v4(),
            "code": random_code(),
            "description": "Bad Unit",
            "is_sustainable": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// CATMAT PAGINATION TESTS
// ============================

#[tokio::test]
async fn test_group_list_pagination() {
    let app = common::spawn_app().await;

    for _ in 0..3 {
        create_group(&app).await;
    }

    let response = app
        .api
        .get("/api/admin/catalog/catmat/groups?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["groups"].as_array().unwrap().len() <= 2);
    assert!(body["total"].as_i64().unwrap() >= 3);
    assert_eq!(body["limit"], 2);
    assert_eq!(body["offset"], 0);
}

// ============================
// CATMAT AUTHORIZATION TESTS
// ============================

#[tokio::test]
async fn test_catmat_groups_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let post = app
        .api
        .post("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({ "code": "X", "name": "Y", "is_active": true }))
        .await;
    assert_eq!(post.status_code(), StatusCode::FORBIDDEN);

    let get = app
        .api
        .get("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(get.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catmat_classes_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/classes")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catmat_pdms_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/pdms")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catmat_items_forbidden_for_regular_user() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catmat_unauthenticated_returns_401() {
    let app = common::spawn_app().await;

    let response = app.api.get("/api/admin/catalog/catmat/groups").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
