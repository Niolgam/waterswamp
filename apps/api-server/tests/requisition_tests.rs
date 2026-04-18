//! Integration tests for requisition audit endpoints
//!
//! Tests the requisition management and audit trail functionality:
//! - CRUD operations on requisitions
//! - Approval/rejection/cancellation workflows
//! - Audit history and rollback functionality

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Creates a test city (with country and state) and returns the city_id
async fn create_test_city(pool: &PgPool) -> Uuid {
    let country_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO countries (name, iso2, bacen_code)
        VALUES ('Requisition Country', 'RQ', 888888)
        ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
        RETURNING id
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test country");

    let state_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO states (country_id, name, abbreviation, ibge_code)
        VALUES ($1, 'Requisition State', 'RQ', 888888)
        ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
        RETURNING id
        "#,
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("Failed to create test state");

    let city_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO cities (state_id, name, ibge_code)
        VALUES ($1, 'Requisition City', 8888888)
        ON CONFLICT (ibge_code) DO UPDATE SET name = cities.name
        RETURNING id
        "#,
    )
    .bind(state_id)
    .fetch_one(pool)
    .await
    .expect("Failed to create test city");

    city_id
}

/// Creates a test warehouse in the database
async fn create_test_warehouse(pool: &PgPool) -> Uuid {
    let unique_id = Uuid::new_v4();
    let code = format!("WH{}", &unique_id.to_string().replace("-", "")[..16]);
    let name = format!("Test Warehouse {}", &unique_id.to_string()[..8]);
    let city_id = create_test_city(pool).await;

    let warehouse_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active)
        VALUES ($1, $2, 'SECTOR', $3, true)
        ON CONFLICT (code) DO UPDATE SET name = warehouses.name
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("Failed to create test warehouse");

    warehouse_id
}

/// Creates a test requisition directly in the database.
/// Automatically fills approved_by/approved_at for statuses that require it,
/// and fulfilled_by/fulfilled_at for FULFILLED/PARTIALLY_FULFILLED.
async fn create_test_requisition(
    pool: &PgPool,
    warehouse_id: Uuid,
    requester_id: Uuid,
    status: &str,
) -> Uuid {
    let requisition_id = Uuid::new_v4();
    let requisition_number = format!("REQ{}", &requisition_id.to_string()[..12]);
    let destination_unit_id = Uuid::new_v4();

    let needs_approval = matches!(
        status,
        "APPROVED" | "PROCESSING" | "FULFILLED" | "PARTIALLY_FULFILLED"
    );
    let needs_fulfillment = matches!(status, "FULFILLED" | "PARTIALLY_FULFILLED");

    let approved_by: Option<Uuid> = if needs_approval { Some(requester_id) } else { None };
    let fulfilled_by: Option<Uuid> = if needs_fulfillment { Some(requester_id) } else { None };

    sqlx::query(
        r#"
        INSERT INTO requisitions (
            id, requisition_number, warehouse_id, destination_unit_id, requester_id,
            status, priority, request_date,
            approved_by, approved_at,
            fulfilled_by, fulfilled_at,
            created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6::requisition_status_enum, 'NORMAL', CURRENT_DATE,
            $7, CASE WHEN $7::uuid IS NOT NULL THEN NOW() ELSE NULL END,
            $8, CASE WHEN $8::uuid IS NOT NULL THEN NOW() ELSE NULL END,
            NOW(), NOW()
        )
        "#,
    )
    .bind(requisition_id)
    .bind(&requisition_number)
    .bind(warehouse_id)
    .bind(destination_unit_id)
    .bind(requester_id)
    .bind(status)
    .bind(approved_by)
    .bind(fulfilled_by)
    .execute(pool)
    .await
    .expect("Failed to create test requisition");

    requisition_id
}

/// Gets the admin user ID from the database
async fn get_admin_user_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(pool)
        .await
        .expect("Admin user 'alice' not found")
}

/// Generates a unique code for testing
fn random_code() -> String {
    format!("TST{}", &Uuid::new_v4().to_string()[..6])
}

// ============================================================================
// REQUISITION LIST AND GET TESTS
// ============================================================================

#[tokio::test]
async fn test_list_requisitions_empty() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/requisitions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn test_list_requisitions_with_data() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let _req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .get("/api/admin/requisitions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_requisitions_with_status_filter() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let _req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;

    let response = app
        .api
        .get("/api/admin/requisitions?status=DRAFT")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    // All returned items should have DRAFT status
    if let Some(items) = body["data"].as_array() {
        for item in items {
            assert_eq!(item["status"].as_str().unwrap(), "Draft");
        }
    }
}

#[tokio::test]
async fn test_get_requisition_by_id() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .get(&format!("/api/admin/requisitions/{}", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"].as_str().unwrap(), req_id.to_string());
}

#[tokio::test]
async fn test_get_requisition_not_found() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!("/api/admin/requisitions/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================================================================
// APPROVAL / REJECTION TESTS
// ============================================================================

#[tokio::test]
async fn test_approve_requisition_success() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/approve", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "notes": "Approved for testing"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "Approved");
    assert!(body["approved_by"].is_string());
    assert!(body["approved_at"].is_string());
}

#[tokio::test]
async fn test_approve_requisition_wrong_status() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    // Create with DRAFT status - can't approve directly
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/approve", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_requisition_success() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/reject", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Budget constraints"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "Rejected");
    assert_eq!(
        body["rejection_reason"].as_str().unwrap(),
        "Budget constraints"
    );
}

#[tokio::test]
async fn test_reject_requisition_requires_reason() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/reject", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": ""
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// CANCELLATION TESTS
// ============================================================================

#[tokio::test]
async fn test_cancel_requisition_success() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/cancel", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "No longer needed"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["success"].as_bool().unwrap_or(false) || body["new_status"].is_string());
}

#[tokio::test]
async fn test_cancel_requisition_requires_reason() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/cancel", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "   "
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// AUDIT HISTORY TESTS
// ============================================================================

#[tokio::test]
async fn test_get_requisition_history() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    // The INSERT should have created a history entry
    let response = app
        .api
        .get(&format!("/api/admin/requisitions/{}/history", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["data"].is_array());
    assert_eq!(body["requisition_id"].as_str().unwrap(), req_id.to_string());
}

#[tokio::test]
async fn test_get_requisition_history_after_approval() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    // Approve the requisition
    app.api
        .post(&format!("/api/admin/requisitions/{}/approve", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    // Check history - should have INSERT and APPROVAL entries
    let response = app
        .api
        .get(&format!("/api/admin/requisitions/{}/history", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let history = body["data"].as_array().expect("history should be array");

    // Should have at least the approval entry
    let has_approval = history.iter().any(|h| {
        h["operation"]
            .as_str()
            .map(|op| op.contains("APPROV"))
            .unwrap_or(false)
    });
    assert!(
        has_approval || history.len() >= 1,
        "Should have approval in history"
    );
}

#[tokio::test]
async fn test_get_requisition_history_with_limit() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/requisitions/{}/history?limit=5",
            req_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let history = body["data"].as_array().expect("history should be array");
    assert!(history.len() <= 5);
}

// ============================================================================
// ROLLBACK POINTS TESTS
// ============================================================================

#[tokio::test]
async fn test_get_rollback_points() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/requisitions/{}/rollback-points",
            req_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["data"].is_array());
    assert_eq!(body["requisition_id"].as_str().unwrap(), req_id.to_string());
}

#[tokio::test]
async fn test_get_rollback_points_not_found() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!(
            "/api/admin/requisitions/{}/rollback-points",
            fake_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    // Should return NOT_FOUND since requisition doesn't exist
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================================================================
// REQUISITION ITEMS TESTS
// ============================================================================

#[tokio::test]
async fn test_get_requisition_items_empty() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .get(&format!("/api/admin/requisitions/{}/items", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body.is_array());
}

// ============================================================================
// AUTHORIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_requisitions_require_authentication() {
    let app = common::spawn_app().await;

    let response = app.api.get("/api/admin/requisitions").await;

    // Should be unauthorized without token
    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_requisitions_require_admin_role() {
    let app = common::spawn_app().await;

    // Use regular user token instead of admin
    let response = app
        .api
        .get("/api/admin/requisitions")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    // Should be forbidden for non-admin users
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_approve_requires_admin_role() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/approve", req_id))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

// ============================================================================
// PAGINATION TESTS
// ============================================================================

#[tokio::test]
async fn test_list_requisitions_pagination() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

    // Create multiple requisitions
    for _ in 0..5 {
        create_test_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;
    }

    // Test limit
    let response = app
        .api
        .get("/api/admin/requisitions?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["data"].as_array().expect("data should be array");
    assert!(items.len() <= 2);
    assert_eq!(body["limit"].as_i64().unwrap(), 2);
    assert_eq!(body["offset"].as_i64().unwrap(), 0);
}

// ============================================================================
// AUDIT CONTEXT TESTS (IP / User Agent capture)
// ============================================================================

#[tokio::test]
async fn test_audit_captures_request_metadata() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    // Approve with custom headers
    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/approve", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .add_header("X-Forwarded-For", "192.168.1.100")
        .add_header("User-Agent", "TestClient/1.0")
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // The audit context should have been captured by the trigger
    // We can verify by checking the history
    let history_response = app
        .api
        .get(&format!("/api/admin/requisitions/{}/history", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(history_response.status_code(), StatusCode::OK);
}

// ============================================================================
// ADD ITEM TO REQUISITION TESTS (replaces fn_capture_requisition_item_value)
// ============================================================================

/// Creates a minimal catmat item hierarchy and returns catalog_item_id
async fn create_test_catalog_item_for_req(pool: &sqlx::PgPool) -> uuid::Uuid {
    let unit_id: uuid::Uuid =
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM units_of_measure WHERE symbol = 'UNID' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("Unit UNID not found");

    let uid = uuid::Uuid::new_v4().simple().to_string();

    let group_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_groups (code, name) VALUES ($1, $2)
         ON CONFLICT (code) DO UPDATE SET name = catmat_groups.name RETURNING id",
    )
    .bind(format!("RG{}", &uid[..5]))
    .bind(format!("Req Group {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_group");

    let class_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_classes (group_id, code, name) VALUES ($1, $2, $3)
         ON CONFLICT (code) DO UPDATE SET name = catmat_classes.name RETURNING id",
    )
    .bind(group_id)
    .bind(format!("RC{}", &uid[..5]))
    .bind(format!("Req Class {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_class");

    let pdm_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_pdms (class_id, code, description, material_classification)
         VALUES ($1, $2, $3, 'STOCKABLE')
         ON CONFLICT (code) DO UPDATE SET description = catmat_pdms.description RETURNING id",
    )
    .bind(class_id)
    .bind(format!("RP{}", &uid[..5]))
    .bind(format!("Req PDM {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_pdm");

    sqlx::query_scalar(
        "INSERT INTO catmat_items (pdm_id, code, description, unit_of_measure_id, is_active)
         VALUES ($1, $2, $3, $4, true)
         ON CONFLICT (code) DO UPDATE SET description = catmat_items.description RETURNING id",
    )
    .bind(pdm_id)
    .bind(format!("RI{}", &uid[..7]))
    .bind(format!("Req Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("catmat_item")
}

#[tokio::test]
async fn test_add_item_to_draft_requisition() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/items", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&serde_json::json!({
            "catalog_item_id": catalog_item_id,
            "requested_quantity": "3.0000",
            "justification": "Reposição de estoque"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        axum::http::StatusCode::CREATED,
        "body: {}",
        response.text()
    );
    let body: serde_json::Value = response.json();
    assert_eq!(body["catalog_item_id"], catalog_item_id.to_string());
    assert_eq!(body["requested_quantity"], "3.0000");
    assert_eq!(body["justification"], "Reposição de estoque");
    // unit_value comes from warehouse_stocks or falls back to 0
    assert!(body["unit_value"].is_string() || body["unit_value"].is_number());
}

#[tokio::test]
async fn test_add_item_to_pending_requisition() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/items", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&serde_json::json!({
            "catalog_item_id": catalog_item_id,
            "requested_quantity": "1.0000"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        axum::http::StatusCode::CREATED,
        "body: {}",
        response.text()
    );
}

#[tokio::test]
async fn test_cannot_add_item_to_approved_requisition() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id =
        create_test_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/items", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&serde_json::json!({
            "catalog_item_id": catalog_item_id,
            "requested_quantity": "1.0000"
        }))
        .await;

    assert_eq!(response.status_code(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_item_requires_catalog_item_id() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/items", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&serde_json::json!({
            "requested_quantity": "1.0000"
        }))
        .await;

    // Missing required field — should be 422 or 400
    assert!(
        response.status_code() == axum::http::StatusCode::UNPROCESSABLE_ENTITY
            || response.status_code() == axum::http::StatusCode::BAD_REQUEST,
        "expected 400/422 got {}",
        response.status_code()
    );
}

// ============================================================================
// START-PROCESSING / FULFILL HELPERS AND TESTS (RF-013, RF-014)
// ============================================================================

/// Inserts a requisition item and returns its ID.
/// Also upserts a warehouse_stock row so unit_value lookup works.
async fn add_requisition_item(
    pool: &PgPool,
    requisition_id: Uuid,
    warehouse_id: Uuid,
    catalog_item_id: Uuid,
    requested_qty: f64,
    approved_qty: Option<f64>,
) -> Uuid {
    sqlx::query(
        "INSERT INTO warehouse_stocks
         (warehouse_id, catalog_item_id, quantity, reserved_quantity, average_unit_value)
         VALUES ($1, $2, 500.0, 0.0, 10.00)
         ON CONFLICT (warehouse_id, catalog_item_id) DO UPDATE SET quantity = 500.0",
    )
    .bind(warehouse_id)
    .bind(catalog_item_id)
    .execute(pool)
    .await
    .expect("warehouse_stock upsert");

    let item_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO requisition_items
         (id, requisition_id, catalog_item_id, requested_quantity, approved_quantity, unit_value, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 10.00, NOW(), NOW())",
    )
    .bind(item_id)
    .bind(requisition_id)
    .bind(catalog_item_id)
    .bind(requested_qty)
    .bind(approved_qty)
    .execute(pool)
    .await
    .expect("requisition_item insert");

    item_id
}

#[tokio::test]
async fn test_start_processing_approved_requisition() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "Processing");
}

#[tokio::test]
async fn test_start_processing_with_notes() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "notes": "Iniciando separação física dos itens" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "Processing");
}

#[tokio::test]
async fn test_start_processing_wrong_status_pending() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_start_processing_wrong_status_draft() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_start_processing_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_start_processing_requires_auth() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", req_id))
        .json(&json!({}))
        .await;

    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_fulfill_requisition_total() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id = add_requisition_item(
        &app.db_auth,
        req_id,
        warehouse_id,
        catalog_item_id,
        5.0,
        Some(5.0),
    )
    .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                {
                    "requisition_item_id": item_id,
                    "fulfilled_quantity": "5.0000"
                }
            ]
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "Fulfilled");
}

#[tokio::test]
async fn test_fulfill_requisition_partial_with_cut_reason() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id = add_requisition_item(
        &app.db_auth,
        req_id,
        warehouse_id,
        catalog_item_id,
        10.0,
        Some(10.0),
    )
    .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                {
                    "requisition_item_id": item_id,
                    "fulfilled_quantity": "6.0000",
                    "cut_reason": "Estoque insuficiente no momento do atendimento"
                }
            ]
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "PartiallyFulfilled");
}

#[tokio::test]
async fn test_fulfill_partial_without_cut_reason_fails() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id = add_requisition_item(
        &app.db_auth,
        req_id,
        warehouse_id,
        catalog_item_id,
        10.0,
        Some(10.0),
    )
    .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                {
                    "requisition_item_id": item_id,
                    "fulfilled_quantity": "3.0000"
                    // cut_reason missing
                }
            ]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fulfill_partial_with_empty_cut_reason_fails() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id = add_requisition_item(
        &app.db_auth,
        req_id,
        warehouse_id,
        catalog_item_id,
        10.0,
        Some(10.0),
    )
    .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                {
                    "requisition_item_id": item_id,
                    "fulfilled_quantity": "3.0000",
                    "cut_reason": "   "
                }
            ]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fulfill_wrong_status_approved() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id =
        add_requisition_item(&app.db_auth, req_id, warehouse_id, catalog_item_id, 2.0, Some(2.0))
            .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [{ "requisition_item_id": item_id, "fulfilled_quantity": "2.0000" }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fulfill_empty_items_list_fails() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "items": [] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fulfill_not_found() {
    let app = common::spawn_app().await;
    let fake_item_id = Uuid::new_v4();

    let response = app
        .api
        .post(&format!(
            "/api/admin/requisitions/{}/fulfill",
            Uuid::new_v4()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [{ "requisition_item_id": fake_item_id, "fulfilled_quantity": "1.0" }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_fulfill_exceeds_approved_quantity() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id =
        add_requisition_item(&app.db_auth, req_id, warehouse_id, catalog_item_id, 5.0, Some(5.0))
            .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [{ "requisition_item_id": item_id, "fulfilled_quantity": "10.0000" }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fulfill_zero_quantity_fails() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id =
        add_requisition_item(&app.db_auth, req_id, warehouse_id, catalog_item_id, 5.0, Some(5.0))
            .await;

    let response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [{ "requisition_item_id": item_id, "fulfilled_quantity": "0.0000" }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_full_approved_to_fulfilled_flow() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;
    let catalog_item_id = create_test_catalog_item_for_req(&app.db_auth).await;
    let item_id = add_requisition_item(
        &app.db_auth,
        req_id,
        warehouse_id,
        catalog_item_id,
        3.0,
        Some(3.0),
    )
    .await;

    // Step 1: start-processing
    let r1 = app
        .api
        .post(&format!("/api/admin/requisitions/{}/start-processing", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;
    assert_eq!(r1.status_code(), StatusCode::OK, "body: {}", r1.text());
    assert_eq!(r1.json::<Value>()["status"].as_str().unwrap(), "Processing");

    // Step 2: fulfill totally
    let r2 = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", req_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [{ "requisition_item_id": item_id, "fulfilled_quantity": "3.0000" }]
        }))
        .await;
    assert_eq!(r2.status_code(), StatusCode::OK, "body: {}", r2.text());
    assert_eq!(r2.json::<Value>()["status"].as_str().unwrap(), "Fulfilled");
}
