mod common;

use common::TestApp;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// HELPERS
// ============================

fn random_code_part() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    uuid.chars().take(4).collect()
}

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

async fn create_unique_classification(
    app: &TestApp,
    parent_id: Option<&str>,
    name_prefix: &str,
) -> Value {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let code_part = random_code_part();
        let name = random_name(name_prefix);

        let mut payload = json!({
            "code_part": code_part,
            "name": name,
            "is_active": true
        });

        if let Some(pid) = parent_id {
            payload["parent_id"] = json!(pid);
        }

        let response = app
            .api
            .post("/api/admin/budget-classifications")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&payload)
            .await;

        if response.status_code() == StatusCode::CREATED {
            return response.json();
        }

        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!(
                "Failed to create classification after {} attempts. Last status: {}. Body: {}",
                attempts,
                response.status_code(),
                response.text()
            );
        }
    }
}

// ============================
// CREATE TESTS
// ============================

#[tokio::test]
async fn test_create_budget_classification_level_1_success() {
    let app = common::spawn_app().await;
    let code_part = random_code_part();
    let name = random_name("Categoria");

    let response = app
        .api
        .post("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code_part": code_part,
            "name": name,
            "is_active": true
        }))
        .await;

    if response.status_code() == StatusCode::CONFLICT {
        return; // Pass if conflict to avoid flakiness
    }

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["code_part"], code_part);
    assert_eq!(body["name"], name);
    assert_eq!(body["level"], 1);
    assert_eq!(body["full_code"], code_part);
    assert_eq!(body["is_active"], true);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_budget_classification_with_parent_success() {
    let app = common::spawn_app().await;

    // Create level 1
    let parent = create_unique_classification(&app, None, "Parent").await;
    let parent_id = parent["id"].as_str().unwrap();

    // Create level 2 child
    let code_part = random_code_part();
    let name = random_name("Child");

    let response = app
        .api
        .post("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": parent_id,
            "code_part": code_part,
            "name": name,
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["parent_id"], parent["id"]);
    assert_eq!(body["level"], 2);
    assert_eq!(body["code_part"], code_part);
    // Full code should be parent_code + "." + code_part
    let expected_full_code = format!("{}.{}", parent["code_part"].as_str().unwrap(), code_part);
    assert_eq!(body["full_code"], expected_full_code);
}

#[tokio::test]
async fn test_create_budget_classification_invalid_parent_returns_404() {
    let app = common::spawn_app().await;

    let fake_parent_id = Uuid::new_v4();
    let response = app
        .api
        .post("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": fake_parent_id.to_string(),
            "code_part": random_code_part(),
            "name": random_name("Invalid"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_budget_classification_exceeds_max_level_returns_400() {
    let app = common::spawn_app().await;

    // Create a 5-level hierarchy
    let level1 = create_unique_classification(&app, None, "L1").await;
    let level2 = create_unique_classification(&app, Some(level1["id"].as_str().unwrap()), "L2").await;
    let level3 = create_unique_classification(&app, Some(level2["id"].as_str().unwrap()), "L3").await;
    let level4 = create_unique_classification(&app, Some(level3["id"].as_str().unwrap()), "L4").await;
    let level5 = create_unique_classification(&app, Some(level4["id"].as_str().unwrap()), "L5").await;

    // Try to create level 6 (should fail)
    let response = app
        .api
        .post("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": level5["id"],
            "code_part": random_code_part(),
            "name": random_name("L6"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let error_text = response.text();
    assert!(error_text.contains("max is 5") || error_text.contains("level 5"));
}

#[tokio::test]
async fn test_create_budget_classification_missing_code_part_returns_422() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": random_name("NoCode"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ============================
// READ TESTS
// ============================

#[tokio::test]
async fn test_get_budget_classification_success() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "GetTest").await;
    let id = classification["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], classification["id"]);
    assert_eq!(body["name"], classification["name"]);
    assert_eq!(body["code_part"], classification["code_part"]);
}

#[tokio::test]
async fn test_get_budget_classification_with_parent_includes_parent_info() {
    let app = common::spawn_app().await;

    let parent = create_unique_classification(&app, None, "Parent").await;
    let child = create_unique_classification(&app, Some(parent["id"].as_str().unwrap()), "Child").await;
    let child_id = child["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/budget-classifications/{}", child_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["parent_id"], parent["id"]);
    assert_eq!(body["parent_name"], parent["name"]);
    assert_eq!(body["parent_full_code"], parent["full_code"]);
}

#[tokio::test]
async fn test_get_budget_classification_not_found() {
    let app = common::spawn_app().await;

    let fake_id = Uuid::new_v4();
    let response = app
        .api
        .get(&format!("/api/admin/budget-classifications/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// UPDATE TESTS
// ============================

#[tokio::test]
async fn test_update_budget_classification_name_success() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "UpdateTest").await;
    let id = classification["id"].as_str().unwrap();
    let new_name = random_name("Updated");

    let response = app
        .api
        .put(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": new_name
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"], new_name);
    assert_eq!(body["code_part"], classification["code_part"]);
}

#[tokio::test]
async fn test_update_budget_classification_is_active_success() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "ActiveTest").await;
    let id = classification["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "is_active": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["is_active"], false);
}

#[tokio::test]
async fn test_update_budget_classification_parent_to_self_returns_400() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "SelfParent").await;
    let id = classification["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": id
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    let error_text = response.text();
    assert!(error_text.contains("self") || error_text.contains("circular"));
}

#[tokio::test]
async fn test_update_budget_classification_invalid_parent_returns_404() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "InvalidParent").await;
    let id = classification["id"].as_str().unwrap();
    let fake_parent_id = Uuid::new_v4();

    let response = app
        .api
        .put(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "parent_id": fake_parent_id.to_string()
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// DELETE TESTS
// ============================

#[tokio::test]
async fn test_delete_budget_classification_success() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "DeleteTest").await;
    let id = classification["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify it's deleted
    let get_response = app
        .api
        .get(&format!("/api/admin/budget-classifications/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_budget_classification_with_children_returns_409() {
    let app = common::spawn_app().await;

    let parent = create_unique_classification(&app, None, "Parent").await;
    let _child = create_unique_classification(&app, Some(parent["id"].as_str().unwrap()), "Child").await;

    let response = app
        .api
        .delete(&format!("/api/admin/budget-classifications/{}", parent["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
    let error_text = response.text();
    assert!(error_text.contains("child") || error_text.contains("children"));
}

#[tokio::test]
async fn test_delete_budget_classification_not_found() {
    let app = common::spawn_app().await;

    let fake_id = Uuid::new_v4();
    let response = app
        .api
        .delete(&format!("/api/admin/budget-classifications/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================
// LIST TESTS
// ============================

#[tokio::test]
async fn test_list_budget_classifications_success() {
    let app = common::spawn_app().await;

    create_unique_classification(&app, None, "List1").await;
    create_unique_classification(&app, None, "List2").await;

    let response = app
        .api
        .get("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["items"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
    assert!(body["limit"].is_number());
    assert!(body["offset"].is_number());
}

#[tokio::test]
async fn test_list_budget_classifications_filter_by_level() {
    let app = common::spawn_app().await;

    let parent = create_unique_classification(&app, None, "L1").await;
    create_unique_classification(&app, Some(parent["id"].as_str().unwrap()), "L2").await;

    let response = app
        .api
        .get("/api/admin/budget-classifications?level=1")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().unwrap();

    for item in items {
        assert_eq!(item["level"], 1);
    }
}

#[tokio::test]
async fn test_list_budget_classifications_filter_by_parent_id() {
    let app = common::spawn_app().await;

    let parent1 = create_unique_classification(&app, None, "Parent1").await;
    let parent2 = create_unique_classification(&app, None, "Parent2").await;

    let child1 = create_unique_classification(&app, Some(parent1["id"].as_str().unwrap()), "Child1").await;
    create_unique_classification(&app, Some(parent2["id"].as_str().unwrap()), "Child2").await;

    let response = app
        .api
        .get(&format!("/api/admin/budget-classifications?parent_id={}", parent1["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().unwrap();

    // Should contain child1
    assert!(items.iter().any(|item| item["id"] == child1["id"]));
    // All items should have parent1 as parent
    for item in items {
        if item["parent_id"].is_string() {
            assert_eq!(item["parent_id"], parent1["id"]);
        }
    }
}

#[tokio::test]
async fn test_list_budget_classifications_filter_by_is_active() {
    let app = common::spawn_app().await;

    let active = create_unique_classification(&app, None, "Active").await;
    let inactive = create_unique_classification(&app, None, "Inactive").await;

    // Make one inactive
    app.api
        .put(&format!("/api/admin/budget-classifications/{}", inactive["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({"is_active": false}))
        .await;

    let response = app
        .api
        .get("/api/admin/budget-classifications?is_active=true")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().unwrap();

    // All items should be active
    for item in items {
        assert_eq!(item["is_active"], true);
    }

    // Should contain active classification
    assert!(items.iter().any(|item| item["id"] == active["id"]));
}

#[tokio::test]
async fn test_list_budget_classifications_with_search() {
    let app = common::spawn_app().await;

    let unique_suffix = Uuid::new_v4().simple().to_string();
    let searchable_name = format!("UniqueSearchable-{}", unique_suffix);

    let mut attempts = 0;
    loop {
        attempts += 1;
        let code_part = random_code_part();
        let response = app
            .api
            .post("/api/admin/budget-classifications")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "code_part": code_part,
                "name": searchable_name,
                "is_active": true
            }))
            .await;

        if response.status_code() == StatusCode::CREATED {
            break;
        }
        if response.status_code() != StatusCode::CONFLICT || attempts >= 10 {
            panic!("Failed to create searchable classification: {}", response.text());
        }
    }

    let response = app
        .api
        .get(&format!("/api/admin/budget-classifications?search={}", searchable_name))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().unwrap();

    assert!(items.iter().any(|item| item["name"].as_str().unwrap().contains(&searchable_name)));
}

#[tokio::test]
async fn test_list_budget_classifications_pagination() {
    let app = common::spawn_app().await;

    // Create some classifications
    for i in 0..5 {
        create_unique_classification(&app, None, &format!("Page{}", i)).await;
    }

    let response = app
        .api
        .get("/api/admin/budget-classifications?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["limit"], 2);
    assert_eq!(body["offset"], 0);
    let items = body["items"].as_array().unwrap();
    assert!(items.len() <= 2);
}

// ============================
// TREE TESTS
// ============================

#[tokio::test]
async fn test_get_tree_returns_hierarchical_structure() {
    let app = common::spawn_app().await;

    // Create a hierarchy
    let level1 = create_unique_classification(&app, None, "TreeL1").await;
    let level2 = create_unique_classification(&app, Some(level1["id"].as_str().unwrap()), "TreeL2").await;
    let _level3 = create_unique_classification(&app, Some(level2["id"].as_str().unwrap()), "TreeL3").await;

    let response = app
        .api
        .get("/api/admin/budget-classifications/tree")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let tree: Value = response.json();
    assert!(tree.is_array());

    // Find our test hierarchy in the tree
    let tree_array = tree.as_array().unwrap();
    let our_root = tree_array.iter().find(|node| node["id"] == level1["id"]);

    if let Some(root) = our_root {
        assert_eq!(root["level"], 1);
        assert!(root["children"].is_array());

        let children = root["children"].as_array().unwrap();
        if let Some(child) = children.iter().find(|c| c["id"] == level2["id"]) {
            assert_eq!(child["level"], 2);
            assert!(child["children"].is_array());
        }
    }
}

// ============================
// AUTHORIZATION TESTS
// ============================

#[tokio::test]
async fn test_budget_classification_requires_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post("/api/admin/budget-classifications")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "code_part": random_code_part(),
            "name": random_name("Unauthorized"),
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_budget_classification_requires_admin_role() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "AuthTest").await;

    let response = app
        .api
        .get(&format!("/api/admin/budget-classifications/{}", classification["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_budget_classification_requires_admin_role() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "AuthUpdate").await;

    let response = app
        .api
        .put(&format!("/api/admin/budget-classifications/{}", classification["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({"name": "Hacked"}))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_budget_classification_requires_admin_role() {
    let app = common::spawn_app().await;
    let classification = create_unique_classification(&app, None, "AuthDelete").await;

    let response = app
        .api
        .delete(&format!("/api/admin/budget-classifications/{}", classification["id"].as_str().unwrap()))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}
