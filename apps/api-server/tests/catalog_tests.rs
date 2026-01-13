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

async fn create_budget_classification(app: &TestApp) -> Value {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let uuid = Uuid::new_v4().simple().to_string();
        let code_part: String = uuid.chars().take(4).collect();

        let response = app
            .api
            .post("/api/admin/budget-classifications")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "code_part": code_part,
                "name": format!("BudgetClass-{}", code_part),
                "is_active": true
            }))
            .await;

        if response.status_code() == StatusCode::CREATED {
            return response.json();
        }

        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!("Failed to create budget classification: {}", response.text());
        }
    }
}

async fn create_unit_of_measure(app: &TestApp, name: &str) -> Value {
    let response = app
        .api
        .post("/api/admin/catalog/units")
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

async fn create_catalog_group(
    app: &TestApp,
    parent_id: Option<&str>,
    budget_classification_id: &str,
    item_type: &str,
) -> Value {
    let mut payload = json!({
        "name": random_name("Group"),
        "code": random_code(),
        "item_type": item_type,
        "budget_classification_id": budget_classification_id,
        "is_active": true
    });

    if let Some(pid) = parent_id {
        payload["parent_id"] = json!(pid);
    }

    let response = app
        .api
        .post("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create catalog group. Status: {}, Body: {}",
            response.status_code(),
            response.text()
        );
    }

    response.json()
}

async fn create_catalog_item(
    app: &TestApp,
    group_id: &str,
    unit_id: &str,
    catmat_code: Option<&str>,
) -> Value {
    let mut payload = json!({
        "group_id": group_id,
        "unit_of_measure_id": unit_id,
        "name": random_name("Item"),
        "specification": "Test specification",
        "estimated_value": 100.50,
        "is_stockable": true,
        "is_permanent": false,
        "requires_batch_control": false,
        "is_active": true
    });

    if let Some(code) = catmat_code {
        payload["catmat_code"] = json!(code);
    }

    let response = app
        .api
        .post("/api/admin/catalog/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;

    if response.status_code() != StatusCode::CREATED {
        panic!(
            "Failed to create catalog item. Status: {}, Body: {}",
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

    let response = app
        .api
        .post("/api/admin/catalog/units")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("Kilogram"),
            "symbol": "KGTEST",
            "is_base_unit": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert!(body["name"].as_str().unwrap().contains("Kilogram"));
    assert_eq!(body["symbol"], "KGTEST");
    assert_eq!(body["is_base_unit"], true);
}

#[tokio::test]
async fn test_create_unit_missing_name_returns_422() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/catalog/units")
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
        .get(&format!("/api/admin/catalog/units/{}", id))
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
        .get(&format!("/api/admin/catalog/units/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_unit_of_measure_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Meter").await;
    let id = unit["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/units/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Meter",
            "symbol": "MTEST"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Meter");
    assert_eq!(body["symbol"], "MTEST");
}

#[tokio::test]
async fn test_delete_unit_of_measure_success() {
    let app = common::spawn_app().await;
    let unit = create_unit_of_measure(&app, "Piece").await;
    let id = unit["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/catalog/units/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify deletion
    let get_response = app
        .api
        .get(&format!("/api/admin/catalog/units/{}", id))
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
        .get("/api/admin/catalog/units")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["units"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
}

// ============================
// CATALOG GROUPS TESTS
// ============================

#[tokio::test]
async fn test_create_catalog_group_root_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;

    let response = app
        .api
        .post("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("RootGroup"),
            "code": random_code(),
            "item_type": "MATERIAL",
            "budget_classification_id": budget_class["id"],
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert_eq!(body["item_type"], "MATERIAL");
    assert!(body["parent_id"].is_null());
}

#[tokio::test]
async fn test_create_catalog_group_with_parent_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;

    let parent = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "SERVICE",
    )
    .await;

    let response = app
        .api
        .post("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": parent["id"],
            "name": random_name("SubGroup"),
            "code": random_code(),
            "item_type": "SERVICE",
            "budget_classification_id": budget_class["id"],
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["parent_id"], parent["id"]);
    assert_eq!(body["item_type"], "SERVICE");
}

#[tokio::test]
async fn test_create_catalog_group_type_mismatch_returns_400() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;

    let parent = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    // Try to create SERVICE subgroup under MATERIAL parent
    let response = app
        .api
        .post("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": parent["id"],
            "name": random_name("Mismatch"),
            "code": random_code(),
            "item_type": "SERVICE",
            "budget_classification_id": budget_class["id"],
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let error_text = response.text();
    assert!(error_text.contains("Conflito") || error_text.contains("tipo"));
}

#[tokio::test]
async fn test_create_subgroup_under_parent_with_items_returns_400() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;

    let parent = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    // Create an item in the parent group (makes it a leaf node)
    create_catalog_item(
        &app,
        parent["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        None,
    )
    .await;

    // Try to create subgroup under parent that has items
    let response = app
        .api
        .post("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": parent["id"],
            "name": random_name("FailSubgroup"),
            "code": random_code(),
            "item_type": "MATERIAL",
            "budget_classification_id": budget_class["id"],
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let error_text = response.text();
    assert!(error_text.contains("item") || error_text.contains("folha"));
}

#[tokio::test]
async fn test_create_catalog_group_invalid_parent_returns_404() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let fake_parent_id = Uuid::new_v4();

    let response = app
        .api
        .post("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": fake_parent_id.to_string(),
            "name": random_name("Invalid"),
            "code": random_code(),
            "item_type": "MATERIAL",
            "budget_classification_id": budget_class["id"],
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_catalog_group_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/catalog/groups/{}",
            group["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], group["id"]);
    assert!(body["budget_classification_name"].is_string());
}

#[tokio::test]
async fn test_update_catalog_group_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "SERVICE",
    )
    .await;

    let response = app
        .api
        .put(&format!(
            "/api/admin/catalog/groups/{}",
            group["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Group Name"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Group Name");
}

#[tokio::test]
async fn test_delete_catalog_group_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let response = app
        .api
        .delete(&format!(
            "/api/admin/catalog/groups/{}",
            group["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_catalog_group_with_children_returns_409() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;

    let parent = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let _child = create_catalog_group(
        &app,
        Some(parent["id"].as_str().unwrap()),
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let response = app
        .api
        .delete(&format!(
            "/api/admin/catalog/groups/{}",
            parent["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_catalog_groups_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;

    create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;
    create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "SERVICE",
    )
    .await;

    let response = app
        .api
        .get("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["groups"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
}

#[tokio::test]
async fn test_get_catalog_groups_tree_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;

    let parent = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let _child = create_catalog_group(
        &app,
        Some(parent["id"].as_str().unwrap()),
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let response = app
        .api
        .get("/api/admin/catalog/groups/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let tree: Value = response.json();
    assert!(tree.is_array());

    // Find our hierarchy in the tree
    let tree_array = tree.as_array().unwrap();
    let our_root = tree_array.iter().find(|node| node["id"] == parent["id"]);

    if let Some(root) = our_root {
        assert!(root["children"].is_array());
        let children = root["children"].as_array().unwrap();
        assert!(!children.is_empty());
    }
}

// ============================
// CATALOG ITEMS TESTS
// ============================

#[tokio::test]
async fn test_create_catalog_item_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let response = app
        .api
        .post("/api/admin/catalog/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": group["id"],
            "unit_of_measure_id": unit["id"],
            "name": random_name("TestItem"),
            "specification": "Detailed specification",
            "estimated_value": 250.75,
            "is_stockable": true,
            "is_permanent": false,
            "requires_batch_control": true,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert_eq!(body["group_id"], group["id"]);
    assert_eq!(body["is_stockable"], true);
}

#[tokio::test]
async fn test_create_catalog_item_in_synthetic_group_returns_400() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;

    let parent = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    // Create a child group (makes parent synthetic)
    let _child = create_catalog_group(
        &app,
        Some(parent["id"].as_str().unwrap()),
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    // Try to create item in synthetic group
    let response = app
        .api
        .post("/api/admin/catalog/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": parent["id"],
            "unit_of_measure_id": unit["id"],
            "name": random_name("FailItem"),
            "specification": "Should fail",
            "estimated_value": 100.0,
            "is_stockable": true,
            "is_permanent": false,
            "requires_batch_control": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    let error_text = response.text();
    assert!(error_text.contains("subgrupo") || error_text.contains("folha"));
}

#[tokio::test]
async fn test_create_catalog_item_with_catmat_code() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let catmat_code = format!("CATMAT-{}", random_code());

    let response = app
        .api
        .post("/api/admin/catalog/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": group["id"],
            "unit_of_measure_id": unit["id"],
            "name": random_name("CatmatItem"),
            "catmat_code": catmat_code,
            "specification": "Item with CATMAT",
            "estimated_value": 150.0,
            "is_stockable": true,
            "is_permanent": false,
            "requires_batch_control": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["catmat_code"], catmat_code);
}

#[tokio::test]
async fn test_create_catalog_item_duplicate_catmat_returns_409() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let catmat_code = format!("CATMAT-{}", random_code());

    // Create first item
    create_catalog_item(
        &app,
        group["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        Some(&catmat_code),
    )
    .await;

    // Try to create second item with same CATMAT code
    let response = app
        .api
        .post("/api/admin/catalog/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "group_id": group["id"],
            "unit_of_measure_id": unit["id"],
            "name": random_name("Duplicate"),
            "catmat_code": catmat_code,
            "specification": "Duplicate CATMAT",
            "estimated_value": 100.0,
            "is_stockable": true,
            "is_permanent": false,
            "requires_batch_control": false,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_catalog_item_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let item = create_catalog_item(
        &app,
        group["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        None,
    )
    .await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/catalog/items/{}",
            item["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], item["id"]);
    assert!(body["group_name"].is_string());
    assert!(body["unit_name"].is_string());
}

#[tokio::test]
async fn test_update_catalog_item_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "SERVICE",
    )
    .await;

    let item = create_catalog_item(
        &app,
        group["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        None,
    )
    .await;

    let response = app
        .api
        .put(&format!(
            "/api/admin/catalog/items/{}",
            item["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Item Name",
            "estimated_value": 500.00
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], "Updated Item Name");
    assert_eq!(body["estimated_value"], "500.00");
}

#[tokio::test]
async fn test_delete_catalog_item_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    let item = create_catalog_item(
        &app,
        group["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        None,
    )
    .await;

    let response = app
        .api
        .delete(&format!(
            "/api/admin/catalog/items/{}",
            item["id"].as_str().unwrap()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_catalog_items_success() {
    let app = common::spawn_app().await;
    let budget_class = create_budget_classification(&app).await;
    let unit = create_unit_of_measure(&app, "Unit").await;
    let group = create_catalog_group(
        &app,
        None,
        budget_class["id"].as_str().unwrap(),
        "MATERIAL",
    )
    .await;

    create_catalog_item(
        &app,
        group["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        None,
    )
    .await;
    create_catalog_item(
        &app,
        group["id"].as_str().unwrap(),
        unit["id"].as_str().unwrap(),
        None,
    )
    .await;

    let response = app
        .api
        .get("/api/admin/catalog/items")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["items"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
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
    // Decimal serialization may include trailing zeros (e.g., "100.0000")
    let factor: f64 = body["conversion_factor"].as_str().unwrap().parse().unwrap();
    assert!((factor - 100.0).abs() < 0.001);
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
async fn test_update_unit_conversion_success() {
    let app = common::spawn_app().await;

    let from_unit = create_unit_of_measure(&app, "Hour").await;
    let to_unit = create_unit_of_measure(&app, "Minute").await;

    let conversion_response = app
        .api
        .post("/api/admin/catalog/conversions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "from_unit_id": from_unit["id"],
            "to_unit_id": to_unit["id"],
            "conversion_factor": 60.0
        }))
        .await;

    let conversion: Value = conversion_response.json();
    let id = conversion["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/catalog/conversions/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "conversion_factor": 59.5
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    // Decimal serialization may include trailing zeros (e.g., "59.5000")
    let factor: f64 = body["conversion_factor"].as_str().unwrap().parse().unwrap();
    assert!((factor - 59.5).abs() < 0.001);
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
        .post("/api/admin/catalog/units")
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
async fn test_catalog_groups_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/groups")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_catalog_items_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/catalog/items")
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
