//! Integration tests for DRS warehouse movement endpoints
//!
//! Covers the new stock movement routes:
//! - RF-009: POST /warehouses/{id}/entries (entrada avulsa — DONATION_IN / ADJUSTMENT_ADD)
//! - RF-011: POST /warehouses/{id}/returns (devolução de requisição — RETURN)
//! - RF-016: POST /warehouses/{id}/disposals (saída por desfazimento — LOSS)
//! - RF-017: POST /warehouses/{id}/manual-exits (saída manual / OS — EXIT)
//! - GET /warehouses/{id}/movements (listagem do histórico de movimentações)

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
         VALUES ('DRS Mvmt Country', 'DM', 666601)
         ON CONFLICT (bacen_code) DO UPDATE SET name = countries.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("country");

    let state_id: Uuid = sqlx::query_scalar(
        "INSERT INTO states (country_id, name, abbreviation, ibge_code)
         VALUES ($1, 'DRS Mvmt State', 'DM', 666601)
         ON CONFLICT (ibge_code) DO UPDATE SET name = states.name
         RETURNING id",
    )
    .bind(country_id)
    .fetch_one(pool)
    .await
    .expect("state");

    sqlx::query_scalar(
        "INSERT INTO cities (state_id, name, ibge_code)
         VALUES ($1, 'DRS Mvmt City', 6666010)
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
    let code = format!("DWM{}", &uid.simple().to_string()[..12]);
    let city_id = create_test_city(pool).await;

    sqlx::query_scalar(
        "INSERT INTO warehouses (name, code, warehouse_type, city_id, is_active)
         VALUES ($1, $2, 'SECTOR', $3, true)
         RETURNING id",
    )
    .bind(format!("DRS Mvmt WH {}", &uid.to_string()[..8]))
    .bind(code)
    .bind(city_id)
    .fetch_one(pool)
    .await
    .expect("warehouse")
}

/// Creates a full catmat chain and pre-loads stock.  Returns (catalog_item_id, unit_id).
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

    // Pre-load stock for exit operations
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

/// Creates a fulfilled requisition and returns (requisition_id, requisition_number)
async fn create_fulfilled_requisition(pool: &PgPool, warehouse_id: Uuid, user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let number = format!("DRSFULFILL{}", &id.simple().to_string()[..8]);
    let dest_unit = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO requisitions
         (id, requisition_number, warehouse_id, destination_unit_id, requester_id,
          status, priority, request_date, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'FULFILLED', 'NORMAL', CURRENT_DATE, NOW(), NOW())",
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

async fn get_admin_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE username = 'vinicius'")
        .fetch_one(pool)
        .await
        .expect("vinicius not found")
}

// ============================================================================
// STANDALONE ENTRY TESTS (RF-009)
// ============================================================================

#[tokio::test]
async fn test_standalone_entry_donation() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    assert_eq!(body["warehouse_id"].as_str().unwrap(), warehouse_id.to_string());
}

#[tokio::test]
async fn test_standalone_entry_inventory_adjustment() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    assert_eq!(body["warehouse_id"].as_str().unwrap(), warehouse_id.to_string());
}

#[tokio::test]
async fn test_return_entry_invalid_requisition_status() {
    let app = common::spawn_app().await;
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    // Create a PENDING requisition — returns are only allowed for FULFILLED / PARTIALLY_FULFILLED
    let pending_req_id = {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO requisitions
             (id, requisition_number, warehouse_id, destination_unit_id, requester_id,
              status, priority, request_date, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'PENDING', 'NORMAL', CURRENT_DATE, NOW(), NOW())",
        )
        .bind(id)
        .bind(format!("DRSPEND{}", &id.simple().to_string()[..8]))
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    let admin_id = get_admin_id(&app.db_auth).await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    assert_eq!(body["sei_process_number"].as_str().unwrap(), "23108.012345/2026-07");
    assert_eq!(body["warehouse_id"].as_str().unwrap(), warehouse_id.to_string());
}

#[tokio::test]
async fn test_disposal_exit_invalid_sei_format() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    // Wrong SEI format — should fail validation (RF-039)
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

    // Each valid SEI format should work
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

    let invalid_seis = [
        "2310.012345/2026-07",  // too few digits in first block
        "23108.12345/2026-07",  // too few digits in second block
        "23108.012345/26-07",   // year too short
        "23108.012345-2026/07", // wrong separators
        "23108.012345/2026",    // missing last block
        "",                     // empty
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/manual-exits", warehouse_id))
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
    assert_eq!(body["warehouse_id"].as_str().unwrap(), warehouse_id.to_string());
}

#[tokio::test]
async fn test_manual_exit_empty_document_number_fails() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/manual-exits", warehouse_id))
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/manual-exits", warehouse_id))
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

    let response = app
        .api
        .post(&format!("/api/admin/warehouses/{}/manual-exits", warehouse_id))
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

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
    assert_eq!(body["warehouse_id"].as_str().unwrap(), warehouse_id.to_string());
}

#[tokio::test]
async fn test_list_movements_after_entry() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    // Create a standalone entry first
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
    // Should have a DONATION_IN movement
    assert!(movements.iter().any(|m| m["movement_type"]
        .as_str()
        .map(|t| t.contains("DONATION") || t.contains("Donation"))
        .unwrap_or(false)));
}

#[tokio::test]
async fn test_list_movements_with_limit_and_offset() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item_id, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    // Create multiple entries
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

    // Fetch with limit=2
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
    let warehouse_id = create_test_warehouse(&app.db_auth).await;
    let (item1, unit_id) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;
    let (item2, _) = create_catalog_item_with_stock(&app.db_auth, warehouse_id).await;

    // Create entries for both items
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

    // Filter by item1
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
    // All returned movements should be for item1
    for m in movements {
        assert_eq!(m["catalog_item_id"].as_str().unwrap(), item1.to_string());
    }
}

#[tokio::test]
async fn test_list_movements_warehouse_not_found() {
    let app = common::spawn_app().await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}/movements", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    // Service returns empty list if warehouse doesn't exist (no items), or 404
    assert!(
        response.status_code() == StatusCode::OK || response.status_code() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_movements_require_auth() {
    let app = common::spawn_app().await;
    let warehouse_id = create_test_warehouse(&app.db_auth).await;

    let response = app
        .api
        .get(&format!("/api/admin/warehouses/{}/movements", warehouse_id))
        .await;

    assert!(
        response.status_code() == StatusCode::UNAUTHORIZED
            || response.status_code() == StatusCode::FORBIDDEN
    );
}
