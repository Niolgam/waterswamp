//! Integration tests for the invoice module
//!
//! Covers:
//! - CRUD (create, get, update, delete, list)
//! - State machine workflow (PENDING → CHECKING → CHECKED → POSTED)
//! - Alternative paths (REJECTED, CANCELLED)
//! - Guard rules (cannot edit non-PENDING, cannot post non-CHECKED, etc.)
//! - Deduplication (duplicate access_key)
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

async fn get_admin_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'vinicius'")
        .fetch_one(pool)
        .await
        .expect("Usuário 'vinicius' não encontrado")
}

/// Creates a minimal supplier and returns its id
async fn create_test_supplier(pool: &PgPool) -> Uuid {
    // Build a valid CPF-like doc using uuid digits
    let raw = Uuid::new_v4().simple().to_string();
    let mut d: Vec<u32> = raw
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(9)
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    while d.len() < 9 {
        d.push(1);
    }
    if d.iter().all(|&x| x == d[0]) {
        d[8] = (d[0] + 1) % 10;
    }
    let s1: u32 = d.iter().enumerate().map(|(i, v)| v * (10 - i as u32)).sum();
    let c1 = {
        let r = (s1 * 10) % 11;
        if r >= 10 {
            0
        } else {
            r
        }
    };
    d.push(c1);
    let s2: u32 = d.iter().enumerate().map(|(i, v)| v * (11 - i as u32)).sum();
    let c2 = {
        let r = (s2 * 10) % 11;
        if r >= 10 {
            0
        } else {
            r
        }
    };
    d.push(c2);
    let cpf: String = d.iter().map(|v| v.to_string()).collect();

    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO suppliers (legal_name, document_number) VALUES ($1, $2) RETURNING id",
    )
    .bind(format!(
        "Test Supplier {}",
        &Uuid::new_v4().to_string()[..8]
    ))
    .bind(cpf)
    .fetch_one(pool)
    .await
    .expect("Failed to create test supplier")
}

/// Creates a city hierarchy and returns city_id
async fn create_test_city(pool: &PgPool) -> Uuid {
    let country_id: Uuid = sqlx::query_scalar(
        "INSERT INTO countries (name, iso2, bacen_code)
         VALUES ('Test Country INV', 'TI', 777701)
         ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("country");

    let state_id: Uuid = sqlx::query_scalar(
        "INSERT INTO states (country_id, name, abbreviation, ibge_code)
         VALUES ($1, 'Test State INV', 'TI', 777701)
         ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
         RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("state");

    sqlx::query_scalar(
        "INSERT INTO cities (state_id, name, ibge_code)
         VALUES ($1, 'Test City INV', 7777010)
         ON CONFLICT (ibge_code) DO UPDATE SET name = cities.name
         RETURNING id",
    )
    .bind(state_id)
    .fetch_one(pool)
    .await
    .expect("city")
}

/// Creates a warehouse and returns its id
async fn create_test_warehouse(pool: &PgPool) -> Uuid {
    let uid = Uuid::new_v4();
    let code = format!("WI{}", &uid.to_string().replace('-', "")[..14]);
    let city_id = create_test_city(pool).await;

    sqlx::query_scalar(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active)
         VALUES ($1, $2, 'SECTOR', $3, true)
         ON CONFLICT (code) DO UPDATE SET name = warehouses.name
         RETURNING id",
    )
    .bind(format!("Test Warehouse {}", &uid.to_string()[..8]))
    .bind(code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("Failed to create test warehouse")
}

/// Returns the id of the seeded 'UNIDADE' unit of measure
async fn get_unit_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM units_of_measure WHERE symbol = 'UNID' LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("Unit UNID not found — ensure migrations_main have been applied")
}

/// Creates a minimal catmat item (group → class → pdm → item) and returns item id
async fn create_test_catmat_item(pool: &PgPool, unit_id: Uuid) -> Uuid {
    let uid = Uuid::new_v4().simple().to_string();
    let group_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_groups (code, name)
         VALUES ($1, $2)
         ON CONFLICT (code) DO UPDATE SET name = catmat_groups.name
         RETURNING id",
    )
    .bind(format!("G{}", &uid[..5]))
    .bind(format!("Test Group {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_group");

    let class_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_classes (group_id, code, name)
         VALUES ($1, $2, $3)
         ON CONFLICT (code) DO UPDATE SET name = catmat_classes.name
         RETURNING id",
    )
    .bind(group_id)
    .bind(format!("C{}", &uid[..5]))
    .bind(format!("Test Class {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_class");

    let pdm_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_pdms (class_id, code, name, material_classification)
         VALUES ($1, $2, $3, 'STOCKABLE')
         ON CONFLICT (code) DO UPDATE SET name = catmat_pdms.name
         RETURNING id",
    )
    .bind(class_id)
    .bind(format!("P{}", &uid[..5]))
    .bind(format!("Test PDM {}", &uid[..5]))
    .fetch_one(pool)
    .await
    .expect("catmat_pdm");

    sqlx::query_scalar(
        "INSERT INTO catmat_items (pdm_id, code, description, unit_of_measure_id, is_active)
         VALUES ($1, $2, $3, $4, true)
         ON CONFLICT (code) DO UPDATE SET description = catmat_items.description
         RETURNING id",
    )
    .bind(pdm_id)
    .bind(format!("I{}", &uid[..7]))
    .bind(format!("Test Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("catmat_item")
}

/// Builds the invoice creation JSON payload
fn invoice_payload(
    supplier_id: Uuid,
    warehouse_id: Uuid,
    catalog_item_id: Uuid,
    unit_id: Uuid,
) -> Value {
    json!({
        "invoice_number": format!("NF{}", &Uuid::new_v4().simple().to_string()[..8]),
        "series": "1",
        "issue_date": "2026-03-16T12:00:00Z",
        "supplier_id": supplier_id,
        "warehouse_id": warehouse_id,
        "total_freight": "10.00",
        "total_discount": "0.00",
        "items": [
            {
                "catalog_item_id": catalog_item_id,
                "unit_raw_id": unit_id,
                "quantity_raw": "5.0000",
                "unit_value_raw": "20.0000",
                "conversion_factor": "1.0"
            }
        ]
    })
}

// ============================================================================
// CRUD TESTS
// ============================================================================

#[tokio::test]
async fn test_create_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let response = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"], "PENDING");
    assert!(body["id"].as_str().is_some());
    assert!(body["supplier_name"].is_string() || body["supplier_name"].is_null());
}

#[tokio::test]
async fn test_get_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["id"], id);
    assert_eq!(body["status"], "PENDING");
}

#[tokio::test]
async fn test_get_invoice_items() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}/items", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let items = body["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["catalog_item_id"], catalog_item_id.to_string());
}

#[tokio::test]
async fn test_update_invoice_pending() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .put(&format!("/api/admin/invoices/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "notes": "Nota atualizada nos testes",
            "commitment_number": "2026NE000001"
        }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["notes"], "Nota atualizada nos testes");
    assert_eq!(body["commitment_number"], "2026NE000001");
}

#[tokio::test]
async fn test_delete_pending_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/invoices/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_invoices() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    for _ in 0..3 {
        app.api
            .post("/api/admin/invoices")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&invoice_payload(
                supplier_id,
                warehouse_id,
                catalog_item_id,
                unit_id,
            ))
            .await;
    }

    let response = app
        .api
        .get("/api/admin/invoices?limit=50&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["total"].as_i64().unwrap() >= 3);
    assert!(body["invoices"].as_array().is_some());
}

#[tokio::test]
async fn test_list_invoices_filter_by_status() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    app.api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await;

    let response = app
        .api
        .get("/api/admin/invoices?status=PENDING")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let invoices = body["invoices"].as_array().unwrap();
    for inv in invoices {
        assert_eq!(inv["status"], "PENDING");
    }
}

// ============================================================================
// WORKFLOW STATE MACHINE TESTS
// ============================================================================

#[tokio::test]
async fn test_full_workflow_pending_to_posted() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    // 1. Create
    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "PENDING");

    // 2. Start checking
    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "start-checking: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"], "CHECKING");
    assert!(body["received_at"].as_str().is_some());

    // 3. Finish checking
    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/finish-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "finish-checking: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"], "CHECKED");

    // 4. Post (triggers stock movement via DB trigger)
    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/post", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;
    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "post: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"], "POSTED");
    assert!(body["posted_at"].as_str().is_some());
    assert!(body["posted_by"].as_str().is_some());
}

#[tokio::test]
async fn test_workflow_reject_from_checking() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/reject", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "rejection_reason": "Divergência nos itens recebidos" }))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "reject: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"], "REJECTED");
    assert_eq!(body["rejection_reason"], "Divergência nos itens recebidos");
}

#[tokio::test]
async fn test_workflow_cancel_pending_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/cancel", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "cancel: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["status"], "CANCELLED");
}

// ============================================================================
// GUARD RULE TESTS (invalid state transitions)
// ============================================================================

#[tokio::test]
async fn test_cannot_edit_non_pending_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    // Move to CHECKING
    app.api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    // Attempt edit — should fail
    let response = app
        .api
        .put(&format!("/api/admin/invoices/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "notes": "should fail" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_post_non_checked_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    // Attempt to post from PENDING (skip CHECKING and CHECKED steps)
    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/post", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_start_checking_from_checking() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_delete_posted_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    // Full flow to POSTED
    app.api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;
    app.api
        .post(&format!("/api/admin/invoices/{}/finish-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;
    app.api
        .post(&format!("/api/admin/invoices/{}/post", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    let response = app
        .api
        .delete(&format!("/api/admin/invoices/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_cancel_already_cancelled_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/invoices/{}/cancel", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/cancel", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_requires_reason() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&invoice_payload(
            supplier_id,
            warehouse_id,
            catalog_item_id,
            unit_id,
        ))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    app.api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    // Empty rejection_reason
    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/reject", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({ "rejection_reason": "" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// DEDUPLICATION TESTS
// ============================================================================

#[tokio::test]
async fn test_duplicate_access_key_is_rejected() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    // 44-digit valid access key
    let access_key = "35260300000000000000550010000000011000000019";

    let payload = json!({
        "invoice_number": "NF0001",
        "access_key": access_key,
        "issue_date": "2026-03-16T12:00:00Z",
        "supplier_id": supplier_id,
        "warehouse_id": warehouse_id,
        "items": [{
            "catalog_item_id": catalog_item_id,
            "unit_raw_id": unit_id,
            "quantity_raw": "1.0000",
            "unit_value_raw": "10.0000",
            "conversion_factor": "1.0"
        }]
    });

    let r1 = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;
    assert_eq!(r1.status_code(), StatusCode::CREATED);

    let mut payload2 = payload.clone();
    payload2["invoice_number"] = json!("NF0002");

    let r2 = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload2)
        .await;
    assert_eq!(r2.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_invoice_empty_items_rejected() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

    let response = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "invoice_number": "NF_EMPTY",
            "issue_date": "2026-03-16T12:00:00Z",
            "supplier_id": supplier_id,
            "warehouse_id": warehouse_id,
            "items": []
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// NOT FOUND TESTS
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_invoice_returns_404() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_start_checking_nonexistent_invoice_returns_404() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/start-checking", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================================================================
// AUTHORIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_invoices_require_authentication() {
    let app = common::spawn_app().await;

    let response = app.api.get("/api/admin/invoices").await;
    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_invoices_require_admin_role() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

// ============================================================================
// PAGINATION TESTS
// ============================================================================

#[tokio::test]
async fn test_list_invoices_pagination() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    for _ in 0..4 {
        app.api
            .post("/api/admin/invoices")
            .add_header("Authorization", format!("Bearer {}", app.admin_token))
            .json(&invoice_payload(
                supplier_id,
                warehouse_id,
                catalog_item_id,
                unit_id,
            ))
            .await;
    }

    let response = app
        .api
        .get("/api/admin/invoices?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["invoices"].as_array().unwrap().len() <= 2);
    assert_eq!(body["limit"].as_i64().unwrap(), 2);
    assert_eq!(body["offset"].as_i64().unwrap(), 0);
}
