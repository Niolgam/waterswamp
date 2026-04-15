//! Integration tests for DRS requisition workflow extensions
//!
//! Covers the new status transitions introduced by the DRS implementation:
//! - RF-013: APPROVED → PROCESSING (start-processing)
//! - RF-014: PROCESSING → FULFILLED / PARTIALLY_FULFILLED (fulfill)
//! - Business rules: cut_reason mandatory for partial fulfillment
//! - Error paths: wrong status, missing fields

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// DB HELPERS
// ============================================================================

async fn create_test_city(pool: &PgPool) -> Uuid {
    let country_id: Uuid = sqlx::query_scalar(
        "INSERT INTO countries (name, iso2, bacen_code)
         VALUES ('DRS Req Country', 'DR', 777701)
         ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("country");

    let state_id: Uuid = sqlx::query_scalar(
        "INSERT INTO states (country_id, name, abbreviation, ibge_code)
         VALUES ($1, 'DRS Req State', 'DR', 777701)
         ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
         RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("state");

    sqlx::query_scalar(
        "INSERT INTO cities (state_id, name, ibge_code)
         VALUES ($1, 'DRS Req City', 7777010)
         ON CONFLICT (ibge_code) DO UPDATE SET name = cities.name
         RETURNING id",
    )
    .bind(state_id)
    .fetch_one(pool)
    .await
    .expect("city")
}

async fn create_test_warehouse(pool: &PgPool) -> Uuid {
    let uid = Uuid::new_v4();
    let code = format!("DRQ{}", &uid.simple().to_string()[..12]);
    let city_id = create_test_city(pool).await;

    sqlx::query_scalar(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active)
         VALUES ($1, $2, 'SECTOR', $3, true)
         RETURNING id",
    )
    .bind(format!("DRS Req WH {}", &uid.to_string()[..8]))
    .bind(code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("warehouse")
}

/// Creates a catmat item chain and returns (catalog_item_id, unit_id)
async fn create_test_catalog_item(pool: &PgPool) -> (Uuid, Uuid) {
    let uid = Uuid::new_v4().simple().to_string();

    let unit_id: Uuid =
        sqlx::query_scalar("SELECT id FROM units_of_measure WHERE symbol = 'UNID' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("Unit UNID not found");

    let group_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_groups (code, name) VALUES ($1, $2)
         ON CONFLICT (code) DO UPDATE SET name = catmat_groups.name RETURNING id",
    )
    .bind(format!("DG{}", &uid[..5]))
    .bind(format!("DRS Group {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_group");

    let class_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_classes (group_id, code, name) VALUES ($1, $2, $3)
         ON CONFLICT (code) DO UPDATE SET name = catmat_classes.name RETURNING id",
    )
    .bind(group_id)
    .bind(format!("DC{}", &uid[..5]))
    .bind(format!("DRS Class {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_class");

    let pdm_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_pdms (class_id, code, description, material_classification)
         VALUES ($1, $2, $3, 'STOCKABLE')
         ON CONFLICT (code) DO UPDATE SET description = catmat_pdms.description RETURNING id",
    )
    .bind(class_id)
    .bind(format!("DP{}", &uid[..5]))
    .bind(format!("DRS PDM {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_pdm");

    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_items (pdm_id, code, description, unit_of_measure_id, base_unit_id, is_active)
         VALUES ($1, $2, $3, $4, $4, true)
         ON CONFLICT (code) DO UPDATE SET description = catmat_items.description RETURNING id",
    )
    .bind(pdm_id)
    .bind(format!("DI{}", &uid[..7]))
    .bind(format!("DRS Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("catmat_item");

    (item_id, unit_id)
}

/// Creates a requisition at the given status and returns its ID
async fn create_requisition(pool: &PgPool, warehouse_id: Uuid, requester_id: Uuid, status: &str) -> Uuid {
    let id = Uuid::new_v4();
    let number = format!("DRSREQ{}", &id.simple().to_string()[..10]);
    let dest_unit_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO requisitions
         (id, requisition_number, warehouse_id, destination_unit_id, requester_id,
          status, priority, request_date, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6::requisition_status_enum, 'NORMAL', CURRENT_DATE, NOW(), NOW())",
    )
    .bind(id)
    .bind(&number)
    .bind(warehouse_id)
    .bind(dest_unit_id)
    .bind(requester_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("requisition insert");

    id
}

/// Inserts a requisition item and returns its ID.
/// Creates a stock entry so `unit_value` lookup works.
async fn add_requisition_item(
    pool: &PgPool,
    requisition_id: Uuid,
    warehouse_id: Uuid,
    catalog_item_id: Uuid,
    requested_qty: f64,
    approved_qty: Option<f64>,
) -> Uuid {
    // Ensure there is a warehouse_stock row so unit_value can be populated
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

async fn get_admin_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE username = 'vinicius'")
        .fetch_one(pool)
        .await
        .expect("vinicius not found")
}

// ============================================================================
// START-PROCESSING TESTS (RF-013)
// ============================================================================

#[tokio::test]
async fn test_start_processing_approved_requisition() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;

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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;

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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PENDING").await;

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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "DRAFT").await;

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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;

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

// ============================================================================
// FULFILL TESTS (RF-014)
// ============================================================================

#[tokio::test]
async fn test_fulfill_requisition_total() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
    let item_id = add_requisition_item(
        &app.db_auth,
        req_id,
        warehouse_id,
        catalog_item_id,
        10.0,
        Some(10.0),
    )
    .await;

    // Partial quantity without cut_reason — must fail (RF-014)
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;

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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
    let item_id =
        add_requisition_item(&app.db_auth, req_id, warehouse_id, catalog_item_id, 5.0, Some(5.0))
            .await;

    // Try to fulfill 10 when only 5 were approved
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "PROCESSING").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
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

// ============================================================================
// START-PROCESSING → FULFILL FLOW (RF-013 + RF-014 combined)
// ============================================================================

#[tokio::test]
async fn test_full_approved_to_fulfilled_flow() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let req_id = create_requisition(&app.db_auth, warehouse_id, admin_id, "APPROVED").await;
    let (catalog_item_id, _) = create_test_catalog_item(&app.db_auth).await;
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
