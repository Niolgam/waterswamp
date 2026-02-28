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

async fn create_unit_of_measure(app: &TestApp, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/units-of-measure")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name(name),
            "symbol": random_code(),
            "is_base_unit": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_catmat_group(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": random_name("CatmatGroup"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_catmat_class(app: &TestApp, group_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": group_id,
            "code": random_code(),
            "name": random_name("CatmatClass"),
            "is_active": true
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create CATMAT class. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_catmat_item(app: &TestApp, class_id: &str, unit_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class_id,
            "unit_of_measure_id": unit_id,
            "code": random_code(),
            "description": random_name("PDM Item"),
            "is_sustainable": false,
            "estimated_value": 100.50,
            "is_permanent": false,
            "requires_batch_control": false,
            "is_active": true
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create CATMAT item. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_catser_group(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": random_code(),
            "name": random_name("CatserGroup"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    response.json()
}

async fn create_catser_class(app: &TestApp, group_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": group_id,
            "code": random_code(),
            "name": random_name("CatserClass"),
            "is_active": true
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create CATSER class. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_catser_item(app: &TestApp, class_id: &str, unit_id: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "class_id": class_id,
            "unit_of_measure_id": unit_id,
            "code": random_code(),
            "description": random_name("Service Item"),
            "estimated_value": 200.00,
            "is_active": true
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create CATSER item. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

// ============================
// UNITS OF MEASURE TESTS
// ============================

#[tokio::test]
async fn test_create_unit_of_measure_success() {
    let app = common::spawn_app().await;
    let symbol = random_code();

    let response = app
        .api
        .post("/api/admin/catalog/units-of-measure")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Kilogram"),
            "symbol": symbol,
            "is_base_unit": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert!(body["name"].as_str().unwrap().contains("Kilogram"));
    assert_eq!(body["symbol"], symbol);
    assert_eq!(body["is_base_unit"], true);
}

#[tokio::test]
async fn test_create_unit_missing_name_returns_422() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/catalog/units-of-measure")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "symbol": "X",
            "is_base_unit": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_unit_of_measure_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Liter").await;
    let id = unit["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/units-of-measure/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], unit["id"]);
    assert_eq!(body["name"], unit["name"]);
}

#[tokio::test]
async fn test_get_unit_not_found() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/units-of-measure/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_unit_of_measure_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Meter").await;
    let id = unit["id"].as_str().unwrap();
    let new_name = random_name("UpdatedMeter");
    let new_symbol = random_code();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/units-of-measure/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": new_name,
            "symbol": new_symbol
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], new_name);
    assert_eq!(body["symbol"], new_symbol);
}

#[tokio::test]
async fn test_delete_unit_of_measure_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Piece").await;
    let id = unit["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/units-of-measure/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    let get_response = app
        .api
        .get(&format!("/api/admin/catalog/units-of-measure/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_units_of_measure_success() {
    let app = common::spawn_app().await;

    create_unit_of_measure(&app, "Unit1").await;
    create_unit_of_measure(&app, "Unit2").await;

    let response = app
        .api
        .get("/api/admin/catalog/units-of-measure")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["units"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
}

// ============================
// CATMAT GROUP TESTS
// ============================

#[tokio::test]
async fn test_create_catmat_group_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;

    assert!(group["id"].is_string());
    assert!(group["code"].is_string());
    assert_eq!(group["is_active"], true);
}

#[tokio::test]
async fn test_get_catmat_group_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;
    let id = group["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/catmat/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], group["id"]);
}

#[tokio::test]
async fn test_list_catmat_groups_success() {
    let app = common::spawn_app().await;
    create_catmat_group(&app).await;
    create_catmat_group(&app).await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["groups"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
}

#[tokio::test]
async fn test_update_catmat_group_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;
    let id = group["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/catmat/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "name": "Updated CATMAT Group" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated CATMAT Group");
}

#[tokio::test]
async fn test_delete_catmat_group_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;
    let id = group["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catmat/groups/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_get_catmat_tree_success() {
    let app = common::spawn_app().await;
    create_catmat_group(&app).await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/groups/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let tree: Value = response.json();
    assert!(tree.is_array());
}

// ============================
// CATMAT CLASS TESTS
// ============================

#[tokio::test]
async fn test_create_catmat_class_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;
    let class = create_catmat_class(&app, group["id"].as_str().unwrap()).await;

    assert!(class["id"].is_string());
    assert_eq!(class["group_id"], group["id"]);
}

#[tokio::test]
async fn test_list_catmat_classes_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;
    create_catmat_class(&app, group["id"].as_str().unwrap()).await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/classes")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["classes"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_delete_catmat_class_success() {
    let app = common::spawn_app().await;
    let group = create_catmat_group(&app).await;
    let class = create_catmat_class(&app, group["id"].as_str().unwrap()).await;
    let id = class["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/catmat/classes/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

// ============================
// CATMAT ITEM (PDM) TESTS
// ============================

#[tokio::test]
async fn test_create_catmat_item_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catmat_group(&app).await;
    let class = create_catmat_class(&app, group["id"].as_str().unwrap()).await;

    let item = create_catmat_item(
        &app,
        class["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
    )
    .await;

    assert!(item["id"].is_string());
    assert_eq!(item["class_id"], class["id"]);
}

#[tokio::test]
async fn test_get_catmat_item_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catmat_group(&app).await;
    let class = create_catmat_class(&app, group["id"].as_str().unwrap()).await;
    let item = create_catmat_item(
        &app,
        class["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/catalog/catmat/items/{}",
            item["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], item["id"]);
    assert!(body["class_name"].is_string());
    assert!(body["unit_name"].is_string());
}

#[tokio::test]
async fn test_list_catmat_items_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catmat_group(&app).await;
    let class = create_catmat_class(&app, group["id"].as_str().unwrap()).await;

    create_catmat_item(
        &app,
        class["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["items"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_delete_catmat_item_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catmat_group(&app).await;
    let class = create_catmat_class(&app, group["id"].as_str().unwrap()).await;
    let item = create_catmat_item(
        &app,
        class["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .delete(&format!(
            "/api/admin/catalog/catmat/items/{}",
            item["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

// ============================
// CATSER GROUP TESTS
// ============================

#[tokio::test]
async fn test_create_catser_group_success() {
    let app = common::spawn_app().await;
    let group = create_catser_group(&app).await;

    assert!(group["id"].is_string());
    assert!(group["code"].is_string());
    assert_eq!(group["is_active"], true);
}

#[tokio::test]
async fn test_list_catser_groups_success() {
    let app = common::spawn_app().await;
    create_catser_group(&app).await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["groups"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_get_catser_tree_success() {
    let app = common::spawn_app().await;
    create_catser_group(&app).await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/groups/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let tree: Value = response.json();
    assert!(tree.is_array());
}

// ============================
// CATSER CLASS TESTS
// ============================

#[tokio::test]
async fn test_create_catser_class_success() {
    let app = common::spawn_app().await;
    let group = create_catser_group(&app).await;
    let class = create_catser_class(&app, group["id"].as_str().unwrap()).await;

    assert!(class["id"].is_string());
    assert_eq!(class["group_id"], group["id"]);
}

// ============================
// CATSER ITEM TESTS
// ============================

#[tokio::test]
async fn test_create_catser_item_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catser_group(&app).await;
    let class = create_catser_class(&app, group["id"].as_str().unwrap()).await;

    let item = create_catser_item(
        &app,
        class["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
    )
    .await;

    assert!(item["id"].is_string());
    assert_eq!(item["class_id"], class["id"]);
}

#[tokio::test]
async fn test_list_catser_items_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catser_group(&app).await;
    let class = create_catser_class(&app, group["id"].as_str().unwrap()).await;

    create_catser_item(
        &app,
        class["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
    )
    .await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["items"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 1);
}

// ============================
// UNIT CONVERSIONS TESTS
// ============================

#[tokio::test]
async fn test_create_unit_conversion_success() {
    let app = common::spawn_app().await;

    let from_unit = create_unit_of_measure(&app, "Meter").await;
    let to_unit = create_unit_of_measure(&app, "Centimeter").await;

    let response = app
        .api
        .post("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "from_unit_id": from_unit["id"],
            "to_unit_id": to_unit["id"],
            "conversion_factor": 100.0
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert_eq!(body["from_unit_id"], from_unit["id"]);
    assert_eq!(body["to_unit_id"], to_unit["id"]);
}

#[tokio::test]
async fn test_create_unit_conversion_same_units_returns_400() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Liter").await;

    let response = app
        .api
        .post("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "from_unit_id": unit["id"],
            "to_unit_id": unit["id"],
            "conversion_factor": 1.0
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_unit_conversion_success() {
    let app = common::spawn_app().await;

    let from_unit = create_unit_of_measure(&app, "Kilogram").await;
    let to_unit = create_unit_of_measure(&app, "Gram").await;

    let conversion_response = app
        .api
        .post("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "from_unit_id": from_unit["id"],
            "to_unit_id": to_unit["id"],
            "conversion_factor": 1000.0
        }))
        .await;

    let conversion: Value = conversion_response.json();
    let id = conversion["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/catalog/conversions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], conversion["id"]);
    assert!(body["from_unit_name"].is_string());
    assert!(body["to_unit_name"].is_string());
}

#[tokio::test]
async fn test_delete_unit_conversion_success() {
    let app = common::spawn_app().await;

    let from_unit = create_unit_of_measure(&app, "Day").await;
    let to_unit = create_unit_of_measure(&app, "Hour").await;

    let conversion_response = app
        .api
        .post("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "from_unit_id": from_unit["id"],
            "to_unit_id": to_unit["id"],
            "conversion_factor": 24.0
        }))
        .await;

    let conversion: Value = conversion_response.json();
    let id = conversion["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/conversions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_unit_conversions_success() {
    let app = common::spawn_app().await;

    let from_unit = create_unit_of_measure(&app, "Foot").await;
    let to_unit = create_unit_of_measure(&app, "Inch").await;

    app.api
        .post("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "from_unit_id": from_unit["id"],
            "to_unit_id": to_unit["id"],
            "conversion_factor": 12.0
        }))
        .await;

    let response = app
        .api
        .get("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["conversions"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 1);
}

// ============================
// AUTHORIZATION TESTS
// ============================

#[tokio::test]
async fn test_catalog_units_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/catalog/units-of-measure")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "name": "Unauthorized",
            "symbol": "UN",
            "is_base_unit": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catmat_groups_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catmat/groups")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catser_groups_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/catser/groups")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catalog_conversions_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}
