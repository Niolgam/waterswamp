//! Integration tests for stock transfer endpoints (RF-018)
//!
//! Covers the two-step transfer lifecycle (RN-011):
//! - POST /warehouses/{warehouse_id}/transfers — initiate (step 1)
//! - POST /transfers/{id}/confirm — confirm receipt (step 2a)
//! - POST /transfers/{id}/reject — reject at destination (step 2b)
//! - POST /transfers/{id}/cancel — cancel before confirmation
//! - GET /transfers — list transfers
//! - GET /transfers/{id} — get transfer with items
//!
//! Business rules verified:
//! - Same source/destination warehouse rejected
//! - Empty items list rejected
//! - Inactive/non-existent destination rejected
//! - Can only confirm/reject/cancel PENDING transfers
//! - Confirmed quantity cannot exceed requested
//! - Rejection/cancellation reason is mandatory

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
         VALUES ('TRF Country', 'DT', 555501)
         ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("country");

    let state_id: Uuid = sqlx::query_scalar(
        "INSERT INTO states (country_id, name, abbreviation, ibge_code)
         VALUES ($1, 'TRF State', 'DT', 555501)
         ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
         RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("state");

    sqlx::query_scalar(
        "INSERT INTO cities (state_id, name, ibge_code)
         VALUES ($1, 'TRF City', 5555010)
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
    let code = format!("DTR{}", &uid.simple().to_string()[..12]);
    let city_id = create_test_city(pool).await;

    sqlx::query_scalar(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active, allows_transfers)
         VALUES ($1, $2, 'SECTOR', $3, true, true)
         RETURNING id",
    )
    .bind(format!("TRF WH {}", &uid.to_string()[..8]))
    .bind(code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("warehouse")
}

/// Creates a catmat item chain with pre-loaded stock. Returns (catalog_item_id, unit_id).
async fn create_catalog_item_with_stock(pool: &PgPool, warehouse_id: Uuid) -> (Uuid, Uuid) {
    let uid = Uuid::new_v4().simple().to_string();

    let unit_id: Uuid =
        sqlx::query_scalar("SELECT id FROM units_of_measure WHERE symbol = 'UNID' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("Unit UNID");

    let group_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_groups (code, name) VALUES ($1, $2)
         ON CONFLICT (code) DO UPDATE SET name = catmat_groups.name RETURNING id",
    )
    .bind(format!("TG{}", &uid[..5]))
    .bind(format!("TRF Group {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("group");

    let class_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_classes (group_id, code, name) VALUES ($1, $2, $3)
         ON CONFLICT (code) DO UPDATE SET name = catmat_classes.name RETURNING id",
    )
    .bind(group_id)
    .bind(format!("TC{}", &uid[..5]))
    .bind(format!("TRF Class {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("class");

    let pdm_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_pdms (class_id, code, description, material_classification)
         VALUES ($1, $2, $3, 'STOCKABLE')
         ON CONFLICT (code) DO UPDATE SET description = catmat_pdms.description RETURNING id",
    )
    .bind(class_id)
    .bind(format!("TP{}", &uid[..5]))
    .bind(format!("TRF PDM {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("pdm");

    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_items (pdm_id, code, description, unit_of_measure_id, is_active)
         VALUES ($1, $2, $3, $4, true)
         ON CONFLICT (code) DO UPDATE SET description = catmat_items.description RETURNING id",
    )
    .bind(pdm_id)
    .bind(format!("TI{}", &uid[..7]))
    .bind(format!("TRF Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("item");

    sqlx::query(
        "INSERT INTO warehouse_stocks
         (warehouse_id, catalog_item_id, quantity, reserved_quantity, average_unit_value)
         VALUES ($1, $2, 100.0, 0.0, 25.00)
         ON CONFLICT (warehouse_id, catalog_item_id) DO UPDATE SET quantity = 100.0",
    )
    .bind(warehouse_id)
    .bind(item_id)
    .execute(pool)
    .await
    .expect("stock upsert");

    (item_id, unit_id)
}

/// Initiates a transfer and returns the full response body.
async fn initiate_transfer(
    app: &common::TestApp,
    source_id: Uuid,
    dest_id: Uuid,
    item_id: Uuid,
    unit_id: Uuid,
    qty: &str,
) -> Value {
    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", source_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "destination_warehouse_id": dest_id,
            "notes": "Transferência de teste",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": qty,
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "initiate_transfer failed: {}",
        response.text()
    );
    response.json()
}

// ============================================================================
// INITIATE TRANSFER TESTS
// ============================================================================

#[tokio::test]
async fn test_initiate_transfer_success() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", src))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "destination_warehouse_id": dst,
            "notes": "Redistribuição de materiais",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "10.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert!(body["transfer_number"]
        .as_str()
        .unwrap_or("")
        .starts_with("TRF-"));
    assert_eq!(body["status"].as_str().unwrap(), "PENDING");
    assert_eq!(
        body["source_warehouse_id"].as_str().unwrap(),
        src.to_string()
    );
    assert_eq!(
        body["destination_warehouse_id"].as_str().unwrap(),
        dst.to_string()
    );
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn test_initiate_transfer_same_warehouse_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "destination_warehouse_id": warehouse_id,
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_initiate_transfer_empty_items_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", src))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "destination_warehouse_id": dst,
            "items": []
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_initiate_transfer_destination_not_found() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", src))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "destination_warehouse_id": Uuid::new_v4(),
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_initiate_transfer_with_expiry() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", src))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "destination_warehouse_id": dst,
            "expires_in_hours": 72,
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["expires_at"].is_string());
}

// ============================================================================
// CONFIRM TRANSFER TESTS
// ============================================================================

#[tokio::test]
async fn test_confirm_transfer_full() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "10.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                { "transfer_item_id": transfer_item_id, "quantity_confirmed": "10.0000" }
            ],
            "notes": "Recebido sem divergências"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "CONFIRMED");
    assert!(body["confirmed_at"].is_string());
}

#[tokio::test]
async fn test_confirm_transfer_partial() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "10.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                { "transfer_item_id": transfer_item_id, "quantity_confirmed": "7.0000" }
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
    assert_eq!(body["status"].as_str().unwrap(), "CONFIRMED");
    let item = &body["items"][0];
    assert_eq!(item["quantity_confirmed"].as_str().unwrap_or("7"), "7");
}

#[tokio::test]
async fn test_confirm_transfer_exceeds_requested_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                { "transfer_item_id": transfer_item_id, "quantity_confirmed": "10.0000" }
            ]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_confirm_transfer_already_confirmed_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "items": [{ "transfer_item_id": transfer_item_id, "quantity_confirmed": "5.0" }] }))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "items": [{ "transfer_item_id": transfer_item_id, "quantity_confirmed": "5.0" }] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// REJECT TRANSFER TESTS
// ============================================================================

#[tokio::test]
async fn test_reject_transfer_success() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "8.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/reject", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "rejection_reason": "Materiais não correspondem ao pedido"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "REJECTED");
    assert_eq!(
        body["rejection_reason"].as_str().unwrap(),
        "Materiais não correspondem ao pedido"
    );
    assert!(body["rejected_at"].is_string());
}

#[tokio::test]
async fn test_reject_transfer_empty_reason_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/reject", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "rejection_reason": "   " }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_already_confirmed_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "items": [{ "transfer_item_id": transfer_item_id, "quantity_confirmed": "5.0" }] }))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/reject", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "rejection_reason": "Tentativa de rejeitar após confirmação" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_transfer_restores_source_stock() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let initial_qty: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(src)
    .bind(item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("initial qty");

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "10.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/transfers/{}/reject", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "rejection_reason": "Materiais incorretos" }))
        .await;

    let final_qty: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(src)
    .bind(item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("final qty");

    assert_eq!(
        initial_qty, final_qty,
        "stock should be restored after rejection"
    );
}

// ============================================================================
// CANCEL TRANSFER TESTS
// ============================================================================

#[tokio::test]
async fn test_cancel_transfer_success() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "6.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/cancel", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "cancellation_reason": "Necessidade cancelada antes do envio"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"].as_str().unwrap(), "CANCELLED");
    assert_eq!(
        body["cancellation_reason"].as_str().unwrap(),
        "Necessidade cancelada antes do envio"
    );
    assert!(body["cancelled_at"].is_string());
}

#[tokio::test]
async fn test_cancel_transfer_empty_reason_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/cancel", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "cancellation_reason": "" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_already_confirmed_fails() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "items": [{ "transfer_item_id": transfer_item_id, "quantity_confirmed": "5.0" }] }))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/transfers/{}/cancel", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "cancellation_reason": "Tentativa de cancelar confirmada" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_transfer_restores_source_stock() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let initial_qty: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(src)
    .bind(item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("initial qty");

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "10.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/transfers/{}/cancel", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "cancellation_reason": "Cancelado por mudança de planos" }))
        .await;

    let final_qty: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(src)
    .bind(item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("final qty");

    assert_eq!(
        initial_qty, final_qty,
        "stock should be restored after cancellation"
    );
}

// ============================================================================
// LIST AND GET TRANSFERS
// ============================================================================

#[tokio::test]
async fn test_list_transfers_empty() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/transfers?limit=10&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn test_list_transfers_with_data() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;

    let response = app
        .api
        .get("/api/admin/transfers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_transfers_filter_by_source() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;

    let response = app
        .api
        .get(&format!("/api/admin/transfers?source_warehouse_id={}", src))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let transfers = body["data"].as_array().unwrap();
    for t in transfers {
        assert_eq!(t["source_warehouse_id"].as_str().unwrap(), src.to_string());
    }
}

#[tokio::test]
async fn test_get_transfer_by_id() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "5.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/transfers/{}", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"].as_str().unwrap(), transfer_id);
    assert!(body["items"].is_array());
    assert!(!body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_transfer_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/transfers/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================================================================
// FULL TRANSFER LIFECYCLE TEST
// ============================================================================

#[tokio::test]
async fn test_full_transfer_lifecycle_initiate_confirm() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;
    let dst = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, src).await;

    let src_initial: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(src)
    .bind(item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("src initial qty");

    let transfer = initiate_transfer(&app, src, dst, item_id, unit_id, "15.0000").await;
    let transfer_id = transfer["id"].as_str().unwrap();
    let transfer_item_id = transfer["items"][0]["id"].as_str().unwrap();
    assert_eq!(transfer["status"].as_str().unwrap(), "PENDING");

    let src_after_initiate: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(src)
    .bind(item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("src after initiate");
    assert!(
        src_after_initiate < src_initial,
        "source qty must decrease after initiate"
    );

    let confirm_response = app
        .api
        .post(&format!("/api/admin/transfers/{}/confirm", transfer_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [{ "transfer_item_id": transfer_item_id, "quantity_confirmed": "15.0000" }]
        }))
        .await;
    assert_eq!(confirm_response.status_code(), StatusCode::OK);
    assert_eq!(
        confirm_response.json::<Value>()["status"].as_str().unwrap(),
        "CONFIRMED"
    );

    let dst_qty: Option<rust_decimal::Decimal> = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(dst)
    .bind(item_id)
    .fetch_optional(&app.db_auth)
    .await
    .expect("dst qty");
    assert!(
        dst_qty.is_some(),
        "destination should have a stock entry after confirm"
    );
    assert!(
        dst_qty.unwrap() > rust_decimal::Decimal::ZERO,
        "destination qty must be positive"
    );
}

#[tokio::test]
async fn test_transfers_require_auth() {
    let app = common::spawn_app().await;
    let src = create_test_warehouse(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/transfers", src))
        .json(&json!({
            "destination_warehouse_id": Uuid::new_v4(),
            "items": []
        }))
        .await;

    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}
