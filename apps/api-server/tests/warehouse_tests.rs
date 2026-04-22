//! Integration tests for the warehouse module
//!
//! Covers:
//! - CRUD (create, get, update, delete, list)
//! - Code uniqueness constraint
//! - Warehouse stocks listing and detail
//! - Stock params update
//! - Stock block / unblock workflow
//! - Guard rules (duplicate code, block already-blocked, unblock non-blocked)
//! - Pagination and filters
//! - Authorization (no token, non-admin token)

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// DB HELPERS
// ============================================================================

/// Creates a city hierarchy and returns city_id (shared with invoice tests)
async fn create_test_city(pool: &PgPool) -> Uuid {
    let country_id: Uuid = sqlx::query_scalar(
        "INSERT INTO countries (name, iso2, bacen_code)
         VALUES ('Test Country WH', 'TW', 888801)
         ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("country");

    let state_id: Uuid = sqlx::query_scalar(
        "INSERT INTO states (country_id, name, abbreviation, ibge_code)
         VALUES ($1, 'Test State WH', 'TW', 888801)
         ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
         RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("state");

    sqlx::query_scalar(
        "INSERT INTO cities (state_id, name, ibge_code)
         VALUES ($1, 'Test City WH', 8888010)
         ON CONFLICT (ibge_code) DO UPDATE SET name = cities.name
         RETURNING id",
    )
    .bind(state_id)
    .fetch_one(pool)
    .await
    .expect("city")
}

/// Creates a warehouse and returns its id
async fn create_test_warehouse_db(pool: &PgPool) -> Uuid {
    let uid = Uuid::new_v4();
    let code = format!("WH{}", &uid.simple().to_string()[..14]);
    let city_id = create_test_city(pool).await;

    sqlx::query_scalar(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active)
         VALUES ($1, $2, 'SECTOR', $3, true)
         RETURNING id",
    )
    .bind(format!("Test WH {}", &uid.to_string()[..8]))
    .bind(code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("Failed to create test warehouse")
}

/// Creates a catmat item chain and a warehouse stock entry, returns (warehouse_id, stock_id)
async fn create_warehouse_with_stock(pool: &PgPool) -> (Uuid, Uuid) {
    let uid = Uuid::new_v4().simple().to_string();

    // Unit of measure
    let unit_id: Uuid = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM units_of_measure WHERE symbol = 'UNID' LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .expect("Unit UNID not found");

    // Catmat chain
    let group_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_groups (code, name) VALUES ($1, $2)
         ON CONFLICT (code) DO UPDATE SET name = catmat_groups.name
         RETURNING id",
    )
    .bind(format!("GW{}", &uid[..5]))
    .bind(format!("WH Group {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_group");

    let class_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_classes (group_id, code, name) VALUES ($1, $2, $3)
         ON CONFLICT (code) DO UPDATE SET name = catmat_classes.name
         RETURNING id",
    )
    .bind(group_id)
    .bind(format!("CW{}", &uid[..5]))
    .bind(format!("WH Class {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_class");

    let pdm_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_pdms (class_id, code, description, material_classification) 
         VALUES ($1, $2, $3, 'STOCKABLE')
         ON CONFLICT (code) DO UPDATE SET description = catmat_pdms.description
         RETURNING id",
    )
    .bind(class_id)
    .bind(format!("PW{}", &uid[..5]))
    .bind(format!("WH PDM {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_pdm");

    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_items (pdm_id, code, description, unit_of_measure_id, is_active)
         VALUES ($1, $2, $3, $4, true)
         ON CONFLICT (code) DO UPDATE SET description = catmat_items.description
         RETURNING id",
    )
    .bind(pdm_id)
    .bind(format!("IW{}", &uid[..7]))
    .bind(format!("WH Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("catmat_item");

    // Create warehouse
    let warehouse_id = create_test_warehouse_db(pool).await;

    // Insert a stock record directly (simulating what a stock movement trigger would do)
    let stock_id: Uuid = sqlx::query_scalar(
        "INSERT INTO warehouse_stocks (warehouse_id, catalog_item_id, quantity, reserved_quantity, average_unit_value)
         VALUES ($1, $2, 100.0, 0.0, 25.50)
         ON CONFLICT (warehouse_id, catalog_item_id) DO UPDATE SET quantity = 100.0
         RETURNING id",
    )
    .bind(warehouse_id)
    .bind(item_id)
    .fetch_one(pool)
    .await
    .expect("warehouse_stock");

    (warehouse_id, stock_id)
}

/// Builds warehouse creation payload
fn warehouse_payload(city_id: Uuid) -> Value {
    let code = format!("API{}", &Uuid::new_v4().simple().to_string()[..8]);
    json!({
        "name": format!("API Warehouse {}", &code),
        "code": code,
        "warehouse_type": "SECTOR",
        "city_id": city_id,
        "allows_transfers": true,
        "is_budgetary": false
    })
}

// ============================================================================
// WAREHOUSE CRUD TESTS
// ============================================================================

#[tokio::test]
async fn test_create_warehouse() {
    let app = common::spawn_app().await;
    let city_id = create_test_city(&app.db_auth).await;

    let response = app
        .api
        .post("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&warehouse_payload(city_id))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert!(body["id"].is_string());
    assert_eq!(body["warehouse_type"].as_str().unwrap(), "SECTOR");
    assert_eq!(body["is_active"].as_bool().unwrap(), true);
}

#[tokio::test]
async fn test_get_warehouse() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"].as_str().unwrap(), warehouse_id.to_string());
}

#[tokio::test]
async fn test_get_warehouse_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_warehouse() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .put(&format!("/api/admin/warehouses/{}", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "name": "Updated Warehouse Name",
            "allows_transfers": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["name"].as_str().unwrap(), "Updated Warehouse Name");
    assert_eq!(body["allows_transfers"].as_bool().unwrap(), false);
}

#[tokio::test]
async fn test_delete_warehouse() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .delete(&format!("/api/admin/warehouses/{}", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_warehouses() {
    let app = common::spawn_app().await;
    let _ = create_test_warehouse_db(&app.db_auth).await;
    let _ = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .get("/api/admin/warehouses?limit=50&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 2);
    assert!(body["warehouses"].is_array());
}

#[tokio::test]
async fn test_list_warehouses_filter_type() {
    let app = common::spawn_app().await;
    let city_id = create_test_city(&app.db_auth).await;

    // Create a CENTRAL warehouse
    let code = format!("CTR{}", &Uuid::new_v4().simple().to_string()[..8]);
    sqlx::query(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id) VALUES ($1, $2, 'CENTRAL', $3)",
    )
    .bind(format!("Central WH {}", &code))
    .bind(code)
    .bind(city_id)
    .execute(&app.db_auth)
    .await
    .unwrap();

    let response = app
        .api
        .get("/api/admin/warehouses?warehouse_type=CENTRAL&limit=50")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let warehouses = body["warehouses"].as_array().unwrap();
    assert!(warehouses.iter().all(|w| w["warehouse_type"] == "CENTRAL"));
}

// ============================================================================
// CODE UNIQUENESS TESTS
// ============================================================================

#[tokio::test]
async fn test_create_warehouse_duplicate_code() {
    let app = common::spawn_app().await;
    let city_id = create_test_city(&app.db_auth).await;
    let payload = warehouse_payload(city_id);

    // First creation
    let r1 = app
        .api
        .post("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;
    assert_eq!(r1.status_code(), StatusCode::CREATED);

    // Second creation with same code → 409
    let r2 = app
        .api
        .post("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;
    assert_eq!(r2.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_warehouse_duplicate_code() {
    let app = common::spawn_app().await;
    let city_id = create_test_city(&app.db_auth).await;

    // Create two warehouses
    let p1 = warehouse_payload(city_id);
    let p2 = warehouse_payload(city_id);
    let code_of_first = p1["code"].as_str().unwrap().to_string();

    app.api
        .post("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&p1)
        .await;

    let r2 = app
        .api
        .post("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&p2)
        .await;

    let id2: Value = r2.json();
    let id2 = id2["id"].as_str().unwrap().to_string();

    // Try to update warehouse2 with code of warehouse1 → 409
    let response = app
        .api
        .put(&format!("/api/admin/warehouses/{}", id2))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "code": code_of_first }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

// ============================================================================
// WAREHOUSE STOCKS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_warehouse_stocks() {
    let app = common::spawn_app().await;
    let (warehouse_id, _stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/warehouses/{}/stocks?limit=50",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["total"].as_i64().unwrap(), 1);
    let stocks = body["stocks"].as_array().unwrap();
    assert_eq!(stocks.len(), 1);
    assert_eq!(
        stocks[0]["warehouse_id"].as_str().unwrap(),
        warehouse_id.to_string()
    );
}

#[tokio::test]
async fn test_get_stock_by_id() {
    let app = common::spawn_app().await;
    let (_warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/stocks/{}", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"].as_str().unwrap(), stock_id.to_string());
    assert!(body["quantity"].is_string());
    assert!(body["available_quantity"].is_string());
    assert!(body["total_value"].is_string());
}

#[tokio::test]
async fn test_get_stock_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/stocks/{}", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_stock_params() {
    let app = common::spawn_app().await;
    let (_warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    let response = app
        .api
        .patch(&format!("/api/admin/warehouses/stocks/{}", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "min_stock": "10.0",
            "max_stock": "200.0",
            "reorder_point": "20.0",
            "resupply_days": 7,
            "location": "Corredor A",
            "secondary_location": "Prateleira 3"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["location"].as_str().unwrap(), "Corredor A");
    assert_eq!(body["resupply_days"].as_i64().unwrap(), 7);
}

// ============================================================================
// BLOCK / UNBLOCK TESTS
// ============================================================================

#[tokio::test]
async fn test_block_and_unblock_stock() {
    let app = common::spawn_app().await;
    let (_warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    // Block
    let response = app
        .api
        .post(&format!("/api/admin/warehouses/stocks/{}/block", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "block_reason": "Produto em quarentena" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["is_blocked"].as_bool().unwrap(), true);
    assert_eq!(
        body["block_reason"].as_str().unwrap(),
        "Produto em quarentena"
    );

    // Unblock
    let response = app
        .api
        .post(&format!(
            "/api/admin/warehouses/stocks/{}/unblock",
            stock_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["is_blocked"].as_bool().unwrap(), false);
    assert!(body["block_reason"].is_null());
}

#[tokio::test]
async fn test_block_already_blocked_stock() {
    let app = common::spawn_app().await;
    let (_warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    // Block first time
    app.api
        .post(&format!("/api/admin/warehouses/stocks/{}/block", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "block_reason": "First block" }))
        .await;

    // Block again → 400
    let response = app
        .api
        .post(&format!("/api/admin/warehouses/stocks/{}/block", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "block_reason": "Second block attempt" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_unblock_non_blocked_stock() {
    let app = common::spawn_app().await;
    let (_warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    // Stock is not blocked, unblock should fail
    let response = app
        .api
        .post(&format!(
            "/api/admin/warehouses/stocks/{}/unblock",
            stock_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_block_requires_reason() {
    let app = common::spawn_app().await;
    let (_warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/stocks/{}/block", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "block_reason": "   " }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// WAREHOUSE STOCKS LIST - FILTER TESTS
// ============================================================================

#[tokio::test]
async fn test_list_stocks_filter_blocked() {
    let app = common::spawn_app().await;
    let (warehouse_id, stock_id) = create_warehouse_with_stock(&app.db_auth).await;

    // Block the stock
    app.api
        .post(&format!("/api/admin/warehouses/stocks/{}/block", stock_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "block_reason": "Testing filter" }))
        .await;

    // Filter by is_blocked=true
    let response = app
        .api
        .get(&format!(
            "/api/admin/warehouses/{}/stocks?is_blocked=true",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["total"].as_i64().unwrap(), 1);

    // Filter by is_blocked=false — should be zero
    let response = app
        .api
        .get(&format!(
            "/api/admin/warehouses/{}/stocks?is_blocked=false",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["total"].as_i64().unwrap(), 0);
}

// ============================================================================
// AUTHORIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_warehouse_requires_auth() {
    let app = common::spawn_app().await;

    // AQUI NÃO VAI TOKEN. Tem que dar 401.
    let response = app.api.get("/api/admin/warehouses").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_warehouse_requires_admin_role() {
    let app = common::spawn_app().await;

    // Aqui vai token de USUÁRIO COMUM (user_token). Tem que dar 403.
    let response = app
        .api
        .get("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_warehouse_requires_admin_role() {
    let app = common::spawn_app().await;
    let city_id = create_test_city(&app.db_auth).await;

    // Aqui vai token de USUÁRIO COMUM (user_token). Tem que dar 403.
    let response = app
        .api
        .post("/api/admin/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&warehouse_payload(city_id))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

// ============================================================================
// STOCK MOVEMENT HELPERS
// ============================================================================

/// Creates a full catmat chain with pre-loaded stock. Returns (catalog_item_id, unit_id).
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
    .bind(format!("MG{}", &uid[..5]))
    .bind(format!("Mvmt Group {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("group");

    let class_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_classes (group_id, code, name) VALUES ($1, $2, $3)
         ON CONFLICT (code) DO UPDATE SET name = catmat_classes.name RETURNING id",
    )
    .bind(group_id)
    .bind(format!("MC{}", &uid[..5]))
    .bind(format!("Mvmt Class {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("class");

    let pdm_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_pdms (class_id, code, description, material_classification)
         VALUES ($1, $2, $3, 'STOCKABLE')
         ON CONFLICT (code) DO UPDATE SET description = catmat_pdms.description RETURNING id",
    )
    .bind(class_id)
    .bind(format!("MP{}", &uid[..5]))
    .bind(format!("Mvmt PDM {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("pdm");

    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_items (pdm_id, code, description, unit_of_measure_id, base_unit_id, is_active)
         VALUES ($1, $2, $3, $4, $4, true)
         ON CONFLICT (code) DO UPDATE SET description = catmat_items.description RETURNING id",
    )
    .bind(pdm_id)
    .bind(format!("MI{}", &uid[..7]))
    .bind(format!("Mvmt Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("item");

    sqlx::query(
        "INSERT INTO warehouse_stocks
         (warehouse_id, catalog_item_id, quantity, reserved_quantity, average_unit_value)
         VALUES ($1, $2, 200.0, 0.0, 15.00)
         ON CONFLICT (warehouse_id, catalog_item_id) DO UPDATE SET quantity = 200.0",
    )
    .bind(warehouse_id)
    .bind(item_id)
    .execute(pool)
    .await
    .expect("stock upsert");

    (item_id, unit_id)
}

/// Creates a FULFILLED requisition. Fills approved_by/fulfilled_by to satisfy CHECK constraints.
async fn create_fulfilled_requisition(pool: &PgPool, warehouse_id: Uuid, user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let number = format!("MVMTFULFILL{}", &id.simple().to_string()[..8]);
    let dest_unit = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO requisitions
         (id, requisition_number, warehouse_id, destination_unit_id, requester_id,
          status, priority, request_date,
          approved_by, approved_at,
          fulfilled_by, fulfilled_at,
          created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'FULFILLED', 'NORMAL', CURRENT_DATE,
                 $5, NOW(), $5, NOW(), NOW(), NOW())",
    )
    .bind(id)
    .bind(&number)
    .bind(warehouse_id)
    .bind(dest_unit)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("requisition insert");

    id
}

async fn get_admin_user_id_mvmt(pool: &PgPool) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(pool)
        .await
        .expect("alice not found")
}

// ============================================================================
// STANDALONE ENTRY TESTS (RF-009)
// ============================================================================

#[tokio::test]
async fn test_standalone_entry_donation() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "123.456.789-00",
            "document_number": "DOA-2026-001",
            "notes": "Doação de equipamentos",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "10.0000",
                "conversion_factor": "1.0000",
                "unit_price_base": "20.00"
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
    assert_eq!(body["movements_created"].as_i64().unwrap(), 1);
    assert_eq!(body["entry_type"].as_str().unwrap(), "Donation");
    assert_eq!(
        body["warehouse_id"].as_str().unwrap(),
        warehouse_id.to_string()
    );
}

#[tokio::test]
async fn test_standalone_entry_inventory_adjustment() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "INVENTORY_ADJUSTMENT",
            "origin_description": "Ajuste de inventário — contagem física",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "conversion_factor": "1.0000",
                "unit_price_base": "0.00"
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
    assert_eq!(body["movements_created"].as_i64().unwrap(), 1);
    assert_eq!(body["entry_type"].as_str().unwrap(), "InventoryAdjustment");
}

#[tokio::test]
async fn test_standalone_entry_missing_origin_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "   ",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "conversion_factor": "1.0000",
                "unit_price_base": "10.00"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_standalone_entry_empty_items_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "Doador XYZ",
            "items": []
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_standalone_entry_warehouse_not_found() {
    let app = common::spawn_app().await;
    let fake_wh = Uuid::new_v4();
    let fake_item = Uuid::new_v4();
    let fake_unit = Uuid::new_v4();

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/entries", fake_wh))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "Teste",
            "items": [{
                "catalog_item_id": fake_item,
                "unit_raw_id": fake_unit,
                "quantity_raw": "1.0",
                "conversion_factor": "1.0",
                "unit_price_base": "10.00"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_standalone_entry_multi_item() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item1, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;
    let (item2, _) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "Doador Multi",
            "items": [
                {
                    "catalog_item_id": item1,
                    "unit_raw_id": unit_id,
                    "quantity_raw": "5.0000",
                    "conversion_factor": "1.0000",
                    "unit_price_base": "10.00"
                },
                {
                    "catalog_item_id": item2,
                    "unit_raw_id": unit_id,
                    "quantity_raw": "3.0000",
                    "conversion_factor": "1.0000",
                    "unit_price_base": "20.00"
                }
            ]
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["movements_created"].as_i64().unwrap(), 2);
}

// ============================================================================
// RETURN ENTRY TESTS (RF-011)
// ============================================================================

#[tokio::test]
async fn test_return_entry_success() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id_mvmt(&app.db_auth).await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let req_id = create_fulfilled_requisition(&app.db_auth, warehouse_id, admin_id).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/returns", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "requisition_id": req_id,
            "notes": "Devolução de itens não utilizados",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "2.0000",
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
    assert_eq!(body["movements_created"].as_i64().unwrap(), 1);
    assert_eq!(body["requisition_id"].as_str().unwrap(), req_id.to_string());
    assert_eq!(
        body["warehouse_id"].as_str().unwrap(),
        warehouse_id.to_string()
    );
}

#[tokio::test]
async fn test_return_entry_invalid_requisition_status() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id_mvmt(&app.db_auth).await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let pending_req_id = {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO requisitions
             (id, requisition_number, warehouse_id, destination_unit_id, requester_id,
              status, priority, request_date, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'PENDING', 'NORMAL', CURRENT_DATE, NOW(), NOW())",
        )
        .bind(id)
        .bind(format!("MVMTPEND{}", &id.simple().to_string()[..8]))
        .bind(warehouse_id)
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .execute(&app.db_auth)
        .await
        .expect("pending req");
        id
    };

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/returns", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "requisition_id": pending_req_id,
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "1.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_return_entry_requisition_not_found() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/returns", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "requisition_id": Uuid::new_v4(),
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "1.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_return_entry_empty_items_fails() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_user_id_mvmt(&app.db_auth).await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let req_id = create_fulfilled_requisition(&app.db_auth, warehouse_id, admin_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/returns", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "requisition_id": req_id,
            "items": []
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// DISPOSAL EXIT TESTS (RF-016)
// ============================================================================

#[tokio::test]
async fn test_disposal_exit_success() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/disposals", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "justification": "Materiais danificados — laudo técnico emitido",
            "sei_process_number": "23108.012345/2026-07",
            "technical_opinion_url": "https://sei.ufmt.br/docs/parecer-tecnico-2026-07.pdf",
            "notes": "Baixa por obsolescência",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "3.0000",
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
    assert_eq!(body["movements_created"].as_i64().unwrap(), 1);
    assert_eq!(
        body["sei_process_number"].as_str().unwrap(),
        "23108.012345/2026-07"
    );
    assert_eq!(
        body["warehouse_id"].as_str().unwrap(),
        warehouse_id.to_string()
    );
}

#[tokio::test]
async fn test_disposal_exit_invalid_sei_format() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/disposals", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "justification": "Materiais danificados",
            "sei_process_number": "INVALIDO-2026",
            "technical_opinion_url": "https://sei.ufmt.br/docs/parecer.pdf",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "1.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_disposal_exit_empty_justification_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/disposals", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "justification": "   ",
            "sei_process_number": "23108.012345/2026-07",
            "technical_opinion_url": "https://sei.ufmt.br/docs/parecer.pdf",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "1.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_disposal_exit_sei_various_valid_formats() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let valid_seis = [
        "23108.000001/2026-01",
        "00001.999999/2000-99",
        "99999.000001/2099-00",
    ];

    for sei in valid_seis {
        let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

        let response = app
            .api
            .post(&format!("/api/admin/warehouses/{}/disposals", warehouse_id))
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "justification": format!("Teste SEI format: {}", sei),
                "sei_process_number": sei,
                "technical_opinion_url": "https://sei.ufmt.br/docs/p.pdf",
                "items": [{
                    "catalog_item_id": item_id,
                    "unit_raw_id": unit_id,
                    "quantity_raw": "1.0000",
                    "conversion_factor": "1.0000"
                }]
            }))
            .await;

        assert_eq!(
            response.status_code(),
            StatusCode::CREATED,
            "SEI '{}' should be valid. body: {}",
            sei,
            response.text()
        );
    }
}

#[tokio::test]
async fn test_disposal_exit_sei_invalid_formats() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let invalid_seis = [
        "2310.012345/2026-07",
        "23108.12345/2026-07",
        "23108.012345/26-07",
        "23108.012345-2026/07",
        "23108.012345/2026",
        "",
    ];

    for sei in invalid_seis {
        let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

        let response = app
            .api
            .post(&format!("/api/admin/warehouses/{}/disposals", warehouse_id))
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "justification": "Teste formato SEI inválido",
                "sei_process_number": sei,
                "technical_opinion_url": "https://sei.ufmt.br/docs/p.pdf",
                "items": [{
                    "catalog_item_id": item_id,
                    "unit_raw_id": unit_id,
                    "quantity_raw": "1.0000",
                    "conversion_factor": "1.0000"
                }]
            }))
            .await;

        assert_eq!(
            response.status_code(),
            StatusCode::BAD_REQUEST,
            "SEI '{}' should be invalid but got {}. body: {}",
            sei,
            response.status_code(),
            response.text()
        );
    }
}

// ============================================================================
// MANUAL EXIT TESTS (RF-017)
// ============================================================================

#[tokio::test]
async fn test_manual_exit_success() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!(
            "/api/admin/warehouses/{}/manual-exits",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "document_number": "OS-2026-00123",
            "justification": "Manutenção preventiva do laboratório",
            "notes": "Saída autorizada pelo responsável técnico",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "4.0000",
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
    assert_eq!(body["movements_created"].as_i64().unwrap(), 1);
    assert_eq!(body["document_number"].as_str().unwrap(), "OS-2026-00123");
    assert_eq!(
        body["warehouse_id"].as_str().unwrap(),
        warehouse_id.to_string()
    );
}

#[tokio::test]
async fn test_manual_exit_empty_document_number_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!(
            "/api/admin/warehouses/{}/manual-exits",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "document_number": "   ",
            "justification": "Justificativa válida",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "1.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_manual_exit_empty_justification_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!(
            "/api/admin/warehouses/{}/manual-exits",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "document_number": "OS-2026-001",
            "justification": "",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "1.0000",
                "conversion_factor": "1.0000"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_manual_exit_empty_items_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .post(&format!(
            "/api/admin/warehouses/{}/manual-exits",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "document_number": "OS-2026-001",
            "justification": "Justificativa válida",
            "items": []
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// LIST MOVEMENTS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_movements_empty_warehouse() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}/movements", warehouse_id))
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
    assert_eq!(body["total"].as_i64().unwrap(), 0);
    assert_eq!(
        body["warehouse_id"].as_str().unwrap(),
        warehouse_id.to_string()
    );
}

#[tokio::test]
async fn test_list_movements_after_entry() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    app.api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "Doador para teste de listagem",
            "items": [{
                "catalog_item_id": item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "conversion_factor": "1.0000",
                "unit_price_base": "10.00"
            }]
        }))
        .await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}/movements", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 1);
    let movements = body["data"].as_array().unwrap();
    assert!(!movements.is_empty());
    assert!(movements.iter().any(|m| m["movement_type"]
        .as_str()
        .map(|t| t.contains("DONATION") || t.contains("Donation"))
        .unwrap_or(false)));
}

#[tokio::test]
async fn test_list_movements_with_limit_and_offset() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    for i in 1..=3 {
        app.api
            .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&json!({
                "entry_type": "DONATION",
                "origin_description": format!("Doador {}", i),
                "items": [{
                    "catalog_item_id": item_id,
                    "unit_raw_id": unit_id,
                    "quantity_raw": "1.0000",
                    "conversion_factor": "1.0000",
                    "unit_price_base": "5.00"
                }]
            }))
            .await;
    }

    let response = app
        .api
        .get(&format!(
            "/api/admin/warehouses/{}/movements?limit=2&offset=0",
            warehouse_id
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let movements = body["data"].as_array().unwrap();
    assert!(movements.len() <= 2);
    assert_eq!(body["limit"].as_i64().unwrap(), 2);
    assert_eq!(body["offset"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_list_movements_filter_by_catalog_item() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;
    let (item1, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;
    let (item2, _) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    app.api
        .post(&format!("/api/admin/warehouses/{}/entries", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "entry_type": "DONATION",
            "origin_description": "Doador filter test",
            "items": [
                { "catalog_item_id": item1, "unit_raw_id": unit_id, "quantity_raw": "2.0", "conversion_factor": "1.0", "unit_price_base": "5.0" },
                { "catalog_item_id": item2, "unit_raw_id": unit_id, "quantity_raw": "3.0", "conversion_factor": "1.0", "unit_price_base": "5.0" }
            ]
        }))
        .await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/warehouses/{}/movements?catalog_item_id={}",
            warehouse_id, item1
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let movements = body["data"].as_array().unwrap();
    for m in movements {
        assert_eq!(m["catalog_item_id"].as_str().unwrap(), item1.to_string());
    }
}

#[tokio::test]
async fn test_list_movements_warehouse_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!(
            "/api/admin/warehouses/{}/movements",
            Uuid::new_v4()
        ))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert!(
        response.status_code() == StatusCode::OK || response.status_code() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_movements_require_auth() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse_db(&app.db_auth).await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}/movements", warehouse_id))
        .await;

    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}
