//! Integration tests for the invoice adjustments module (glosas)
//!
//! Covers:
//! - Listing adjustments for an invoice
//! - Creating adjustments (glosas) on POSTED invoices
//! - Guard rules (only POSTED invoices accept adjustments)
//! - Stock movement generation for STOCKABLE items (ADJUSTMENT_SUB)
//! - Validation: empty reason, empty items, qty and value both zero
//! - Authorization

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// DB HELPERS (mirrored from invoice_tests.rs)
// ============================================================================

async fn create_test_supplier(pool: &PgPool) -> Uuid {
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
        if r >= 10 { 0 } else { r }
    };
    d.push(c1);
    let s2: u32 = d.iter().enumerate().map(|(i, v)| v * (11 - i as u32)).sum();
    let c2 = {
        let r = (s2 * 10) % 11;
        if r >= 10 { 0 } else { r }
    };
    d.push(c2);
    let cpf: String = d.iter().map(|v| v.to_string()).collect();

    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO suppliers (legal_name, document_number) VALUES ($1, $2) RETURNING id",
    )
    .bind(format!("ADJ Supplier {}", &Uuid::new_v4().to_string()[..8]))
    .bind(cpf)
    .fetch_one(pool)
    .await
    .expect("Failed to create test supplier")
}

async fn create_test_city(pool: &PgPool) -> Uuid {
    let country_id: Uuid = sqlx::query_scalar(
        "INSERT INTO countries (name, iso2, bacen_code)
         VALUES ('Test Country ADJ', 'TA', 777801)
         ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("country");

    let state_id: Uuid = sqlx::query_scalar(
        "INSERT INTO states (country_id, name, abbreviation, ibge_code)
         VALUES ($1, 'Test State ADJ', 'TA', 777801)
         ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
         RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("state");

    sqlx::query_scalar(
        "INSERT INTO cities (state_id, name, ibge_code)
         VALUES ($1, 'Test City ADJ', 7778010)
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
    let code = format!("WA{}", &uid.to_string().replace('-', "")[..14]);
    let city_id = create_test_city(pool).await;

    sqlx::query_scalar(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active)
         VALUES ($1, $2, 'SECTOR', $3, true)
         ON CONFLICT (code) DO UPDATE SET name = warehouses.name
         RETURNING id",
    )
    .bind(format!("ADJ Warehouse {}", &uid.to_string()[..8]))
    .bind(code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("warehouse")
}

async fn get_unit_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM units_of_measure WHERE symbol = 'UNID' LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("Unit UNID not found")
}

async fn create_test_catmat_item(pool: &PgPool, unit_id: Uuid) -> Uuid {
    let uid = Uuid::new_v4().simple().to_string();
    let group_id: Uuid = sqlx::query_scalar(
        "INSERT INTO catmat_groups (code, name)
         VALUES ($1, $2)
         ON CONFLICT (code) DO UPDATE SET name = catmat_groups.name
         RETURNING id",
    )
    .bind(format!("AG{}", &uid[..5]))
    .bind(format!("ADJ Group {}", &uid[..5]))
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
    .bind(format!("AC{}", &uid[..5]))
    .bind(format!("ADJ Class {}", &uid[..5]))
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
    .bind(format!("AP{}", &uid[..5]))
    .bind(format!("ADJ PDM {}", &uid[..5]))
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
    .bind(format!("AI{}", &uid[..7]))
    .bind(format!("ADJ Item {}", &uid[..7]))
    .bind(unit_id)
    .fetch_one(pool)
    .await
    .expect("catmat_item")
}

/// Creates a POSTED invoice and returns (invoice_id, invoice_item_id)
async fn create_posted_invoice(
    app: &common::TestApp,
    supplier_id: Uuid,
    warehouse_id: Uuid,
    catalog_item_id: Uuid,
    unit_id: Uuid,
) -> (String, String) {
    let payload = json!({
        "invoice_number": format!("ADJ{}", &Uuid::new_v4().simple().to_string()[..8]),
        "series": "1",
        "issue_date": "2026-03-20T12:00:00Z",
        "supplier_id": supplier_id,
        "warehouse_id": warehouse_id,
        "items": [{
            "catalog_item_id": catalog_item_id,
            "unit_raw_id": unit_id,
            "quantity_raw": "10.0000",
            "unit_value_raw": "50.0000",
            "conversion_factor": "1.0"
        }]
    });

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await
        .json();
    let id = created["id"].as_str().unwrap().to_string();

    // PENDING → CHECKING
    app.api
        .post(&format!("/api/admin/invoices/{}/start-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    // CHECKING → CHECKED
    app.api
        .post(&format!("/api/admin/invoices/{}/finish-checking", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    // CHECKED → POSTED
    app.api
        .post(&format!("/api/admin/invoices/{}/post", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    // Fetch item id
    let items_resp: Value = app
        .api
        .get(&format!("/api/admin/invoices/{}/items", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await
        .json();
    let item_id = items_resp["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    (id, item_id)
}

// ============================================================================
// LIST ADJUSTMENTS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_adjustments_empty() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, _item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::OK,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert!(body.is_array(), "should return an array");
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_adjustments_not_found_invoice() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}/adjustments", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

// ============================================================================
// CREATE ADJUSTMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_create_adjustment_happy_path() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let payload = json!({
        "reason": "Quantidade divergente na conferência física",
        "items": [{
            "invoice_item_id": item_id,
            "adjusted_quantity": "2.0000",
            "adjusted_value": "100.0000",
            "notes": "Devolvido ao fornecedor"
        }]
    });

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await;

    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "body: {}",
        response.text()
    );
    let body: Value = response.json();
    assert_eq!(body["invoice_id"], invoice_id);
    assert_eq!(
        body["reason"],
        "Quantidade divergente na conferência física"
    );
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());

    let items = body["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["invoice_item_id"], item_id);
    assert_eq!(items[0]["adjusted_quantity"], "2.0000");
    assert_eq!(items[0]["adjusted_value"], "100.0000");
    assert_eq!(items[0]["notes"], "Devolvido ao fornecedor");
}

#[tokio::test]
async fn test_create_adjustment_appears_in_list() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    app.api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Glosa de teste",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "1.0000"}]
        }))
        .await;

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let adjustments = body.as_array().expect("array");
    assert_eq!(adjustments.len(), 1);
    assert_eq!(adjustments[0]["reason"], "Glosa de teste");
}

#[tokio::test]
async fn test_create_multiple_adjustments_on_same_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    // First glosa
    let r1 = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Glosa 1 — quantidade divergente",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "1.0000"}]
        }))
        .await;
    assert_eq!(r1.status_code(), StatusCode::CREATED);

    // Second glosa
    let r2 = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Glosa 2 — valor divergente",
            "items": [{"invoice_item_id": item_id, "adjusted_value": "50.0000"}]
        }))
        .await;
    assert_eq!(r2.status_code(), StatusCode::CREATED);

    let list: Value = app
        .api
        .get(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await
        .json();
    assert_eq!(list.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_create_adjustment_generates_stock_movement() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    // Stock after posting: should be 10
    let qty_before: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(warehouse_id)
    .bind(catalog_item_id)
    .fetch_optional(&app.db_auth)
    .await
    .expect("query")
    .unwrap_or(rust_decimal::Decimal::ZERO);

    assert_eq!(qty_before, rust_decimal::Decimal::from(10));

    // Create glosa for 3 units
    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Ajuste de estoque por avaria",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "3.0000"}]
        }))
        .await;
    assert_eq!(
        response.status_code(),
        StatusCode::CREATED,
        "body: {}",
        response.text()
    );

    // Stock should now be 7
    let qty_after: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT quantity FROM warehouse_stocks WHERE warehouse_id = $1 AND catalog_item_id = $2",
    )
    .bind(warehouse_id)
    .bind(catalog_item_id)
    .fetch_one(&app.db_auth)
    .await
    .expect("query");

    assert_eq!(qty_after, rust_decimal::Decimal::from(7));

    // Verify ADJUSTMENT_SUB movement was recorded
    let movement_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stock_movements
         WHERE invoice_id = $1 AND movement_type = 'ADJUSTMENT_SUB'",
    )
    .bind(uuid::Uuid::parse_str(&invoice_id).unwrap())
    .fetch_one(&app.db_auth)
    .await
    .expect("count");

    assert_eq!(movement_count, 1);
}

// ============================================================================
// GUARD RULE TESTS
// ============================================================================

#[tokio::test]
async fn test_cannot_create_adjustment_on_pending_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let payload = json!({
        "invoice_number": format!("NF{}", &Uuid::new_v4().simple().to_string()[..8]),
        "issue_date": "2026-03-20T12:00:00Z",
        "supplier_id": supplier_id,
        "warehouse_id": warehouse_id,
        "items": [{"catalog_item_id": catalog_item_id, "unit_raw_id": unit_id,
                   "quantity_raw": "5.0", "unit_value_raw": "10.0", "conversion_factor": "1.0"}]
    });

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await
        .json();
    let invoice_id = created["id"].as_str().unwrap();
    let item_id = app
        .api
        .get(&format!("/api/admin/invoices/{}/items", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await
        .json::<Value>()["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Tentativa em NF PENDING",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "1.0"}]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_create_adjustment_on_cancelled_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let payload = json!({
        "invoice_number": format!("NF{}", &Uuid::new_v4().simple().to_string()[..8]),
        "issue_date": "2026-03-20T12:00:00Z",
        "supplier_id": supplier_id,
        "warehouse_id": warehouse_id,
        "items": [{"catalog_item_id": catalog_item_id, "unit_raw_id": unit_id,
                   "quantity_raw": "5.0", "unit_value_raw": "10.0", "conversion_factor": "1.0"}]
    });

    let created: Value = app
        .api
        .post("/api/admin/invoices")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&payload)
        .await
        .json();
    let invoice_id = created["id"].as_str().unwrap().to_string();
    let item_id = app
        .api
        .get(&format!("/api/admin/invoices/{}/items", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await
        .json::<Value>()["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Cancel
    app.api
        .post(&format!("/api/admin/invoices/{}/cancel", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({}))
        .await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Tentativa em NF CANCELLED",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "1.0"}]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_adjustment_item_must_belong_to_invoice() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, _item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Item de outra NF",
            "items": [{"invoice_item_id": Uuid::new_v4(), "adjusted_quantity": "1.0"}]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// VALIDATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adjustment_requires_reason() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "   ",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "1.0"}]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_adjustment_requires_at_least_one_item() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, _item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Glosa sem itens",
            "items": []
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_adjustment_item_must_have_positive_qty_or_value() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "reason": "Item sem quantidade nem valor",
            "items": [{
                "invoice_item_id": item_id,
                "adjusted_quantity": "0",
                "adjusted_value": "0"
            }]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// AUTHORIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adjustments_require_authentication() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!("/api/admin/invoices/{}/adjustments", fake_id))
        .await;

    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_adjustments_require_admin_role() {
    let app = common::spawn_app().await;
    let supplier_id = create_test_supplier(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let unit_id = get_unit_id(&app.db_auth).await;
    let catalog_item_id = create_test_catmat_item(&app.db_auth, unit_id).await;

    let (invoice_id, item_id) =
        create_posted_invoice(&app, supplier_id, warehouse_id, catalog_item_id, unit_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/invoices/{}/adjustments", invoice_id))
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .json(&json!({
            "reason": "Tentativa sem permissão",
            "items": [{"invoice_item_id": item_id, "adjusted_quantity": "1.0"}]
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}
