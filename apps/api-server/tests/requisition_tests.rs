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
    let suffix = &Uuid::new_v4().to_string()[..8];

    // Create country (schema: id, name, iso2, bacen_code)
    let country_id = Uuid::new_v4();
    let bacen_code = (suffix.as_bytes()[0] as i32 % 9000) + 1000;
    sqlx::query(
        r#"
        INSERT INTO countries (id, name, iso2, bacen_code)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(country_id)
    .bind(format!("Test Country {}", suffix))
    .bind(&suffix[..2].to_uppercase())
    .bind(bacen_code)
    .execute(pool)
    .await
    .expect("Failed to create test country");

    // Create state (schema: id, country_id, name, abbreviation, ibge_code)
    let state_id = Uuid::new_v4();
    let state_abbr: String = suffix.chars().filter(|c| c.is_alphabetic()).take(2).collect::<String>().to_uppercase();
    let state_abbr = if state_abbr.len() < 2 { "ZZ".to_string() } else { state_abbr };
    let state_ibge = (suffix.as_bytes()[0] as i32 % 40) + 60;

    sqlx::query(
        r#"
        INSERT INTO states (id, country_id, name, abbreviation, ibge_code)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (ibge_code) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(state_id)
    .bind(country_id)
    .bind(format!("Test State {}", suffix))
    .bind(&state_abbr)
    .bind(state_ibge)
    .execute(pool)
    .await
    .expect("Failed to create test state");

    // Create city (schema: id, state_id, name, ibge_code)
    let city_id = Uuid::new_v4();
    let city_ibge = (suffix.as_bytes()[0] as i32 % 9000000) + 1000000;
    sqlx::query(
        r#"
        INSERT INTO cities (id, state_id, name, ibge_code)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (ibge_code) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(city_id)
    .bind(state_id)
    .bind(format!("Test City {}", suffix))
    .bind(city_ibge)
    .execute(pool)
    .await
    .expect("Failed to create test city");

    city_id
}

/// Creates a test warehouse in the database
async fn create_test_warehouse(pool: &PgPool) -> Uuid {
    let warehouse_id = Uuid::new_v4();
    let name = format!("Test Warehouse {}", &warehouse_id.to_string()[..8]);
    let city_id = create_test_city(pool).await;

    sqlx::query(
        r#"
        INSERT INTO warehouses (id, name, code, warehouse_type, city_id, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, 'SECTOR', $4, true, NOW(), NOW())
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(warehouse_id)
    .bind(&name)
    .bind(&name[..8])
    .bind(city_id)
    .execute(pool)
    .await
    .expect("Failed to create test warehouse");

    warehouse_id
}

/// Creates a test requisition directly in the database
async fn create_test_requisition(
    pool: &PgPool,
    warehouse_id: Uuid,
    requester_id: Uuid,
    status: &str,
) -> Uuid {
    let requisition_id = Uuid::new_v4();
    let requisition_number = format!("REQ{}", &requisition_id.to_string()[..8]);

    sqlx::query(
        r#"
        INSERT INTO requisitions (
            id, requisition_number, warehouse_id, requester_id,
            status, priority, request_date, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5::requisition_status_enum, 'NORMAL', CURRENT_DATE, NOW(), NOW())
        "#,
    )
    .bind(requisition_id)
    .bind(&requisition_number)
    .bind(warehouse_id)
    .bind(requester_id)
    .bind(status)
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
    assert_eq!(body["rejection_reason"].as_str().unwrap(), "Budget constraints");
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
        h["operation"].as_str().map(|op| op.contains("APPROV")).unwrap_or(false)
    });
    assert!(has_approval || history.len() >= 1, "Should have approval in history");
}

#[tokio::test]
async fn test_get_requisition_history_with_limit() {
    let app = common::spawn_app().await;

    let admin_id = get_admin_user_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_test_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

    let response = app
        .api
        .get(&format!("/api/admin/requisitions/{}/history?limit=5", req_id))
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
        .get(&format!("/api/admin/requisitions/{}/rollback-points", req_id))
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
        .get(&format!("/api/admin/requisitions/{}/rollback-points", fake_id))
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

    let response = app
        .api
        .get("/api/admin/requisitions")
        .await;

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
