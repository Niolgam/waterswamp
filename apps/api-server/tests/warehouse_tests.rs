mod common;

use axum::http::StatusCode;
use common::spawn_app;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::str::FromStr;
use uuid::Uuid;

/// Helper to generate unique codes for tests
fn unique_code(prefix: &str) -> String {
    format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
}

// ============================
// Material Groups Tests
// ============================

#[tokio::test]
async fn test_create_material_group_success() {
    let app = spawn_app().await;
    let code = unique_code("GRP");

    let response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": "Materiais de Consumo",
            "description": "Materiais de consumo em geral",
            "expense_element": "339030",
            "is_personnel_exclusive": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["material_group"]["code"], code);
    assert_eq!(body["material_group"]["name"], "Materiais de Consumo");
}

#[tokio::test]
async fn test_create_material_group_duplicate_code() {
    let app = spawn_app().await;
    let code = unique_code("DUP");

    // Create first group
    app.api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": "Grupo 1",
            "is_personnel_exclusive": false
        }))
        .await;

    // Try to create duplicate
    let response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": code,
            "name": "Grupo 2",
            "is_personnel_exclusive": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

// ============================
// Materials Tests
// ============================

#[tokio::test]
async fn test_create_material_success() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    // Create material group first
    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Equipamentos",
            "is_personnel_exclusive": false
        }))
        .await;

    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    // Create material
    let response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Caneta Azul",
            "description": "Caneta esferográfica azul",
            "catmat_code": "123456",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "2.50",
            "minimum_quantity": "100",
            "is_active": true
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["material"]["code"], material_code);
    assert_eq!(body["material"]["name"], "Caneta Azul");
}

// ============================
// Warehouse Tests
// ============================

async fn create_test_warehouse(app: &common::TestApp) -> String {
    // Get a city ID (assuming cities exist from migrations/seeds)
    let city_id: Uuid = sqlx::query_scalar("SELECT id FROM cities LIMIT 1")
        .fetch_one(&app.db_auth)
        .await
        .expect("Need at least one city");

    let warehouse_code = unique_code("ALM");

    let response = app
        .api
        .post("/api/admin/warehouse/warehouses")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "city_id": city_id.to_string(),
            "name": "Almoxarifado Central",
            "code": warehouse_code,
            "address": "Rua Principal, 100"
        }))
        .await;

    let body: Value = response.json();
    body["warehouse"]["id"].as_str().unwrap().to_string()
}

// ============================
// Stock Movement Tests (Weighted Average)
// ============================

#[tokio::test]
async fn test_stock_entry_calculates_weighted_average() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    // Setup: Create material group, material, and warehouse
    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Test Group",
            "is_personnel_exclusive": false
        }))
        .await;
    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    let material_response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Test Material",
            "catmat_code": "111111",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "10.00",
            "is_active": true
        }))
        .await;
    let material_body: Value = material_response.json();
    let material_id = material_body["material"]["id"].as_str().unwrap();

    let warehouse_id = create_test_warehouse(&app).await;

    // First entry: 100 units @ R$ 7.00
    let entry1 = app
        .api
        .post("/api/admin/warehouse/stock/entry")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "100",
            "unit_value": "7.00",
            "document_number": "NF001",
            "notes": "First entry"
        }))
        .await;

    assert_eq!(entry1.status_code(), StatusCode::CREATED);
    let body1: Value = entry1.json();

    // Check: quantity = 100, average = 7.00
    assert_eq!(body1["stock"]["quantity"], "100");
    let avg1 = Decimal::from_str(body1["stock"]["average_unit_value"].as_str().unwrap()).unwrap();
    assert_eq!(avg1, Decimal::from_str("7.00").unwrap());

    // Second entry: 50 units @ R$ 8.00
    // Expected: new_avg = (100*7 + 50*8) / 150 = 1100/150 = 7.33333...
    let entry2 = app
        .api
        .post("/api/admin/warehouse/stock/entry")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "50",
            "unit_value": "8.00",
            "document_number": "NF002"
        }))
        .await;

    assert_eq!(entry2.status_code(), StatusCode::CREATED);
    let body2: Value = entry2.json();

    // Check: quantity = 150, average ≈ 7.33
    assert_eq!(body2["stock"]["quantity"], "150");
    let avg2 = Decimal::from_str(body2["stock"]["average_unit_value"].as_str().unwrap()).unwrap();
    let expected_avg = Decimal::from_str("7.333333333333333333333333333").unwrap();
    assert_eq!(avg2, expected_avg);
}

#[tokio::test]
async fn test_stock_exit_maintains_average() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    // Setup: Create material group, material, and warehouse
    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Exit Test Group",
            "is_personnel_exclusive": false
        }))
        .await;
    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    let material_response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Exit Test Material",
            "catmat_code": "222222",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "5.00",
            "is_active": true
        }))
        .await;
    let material_body: Value = material_response.json();
    let material_id = material_body["material"]["id"].as_str().unwrap();

    let warehouse_id = create_test_warehouse(&app).await;

    // Entry: 200 units @ R$ 10.00
    app.api
        .post("/api/admin/warehouse/stock/entry")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "200",
            "unit_value": "10.00"
        }))
        .await;

    // Exit: 50 units (average should remain 10.00)
    let exit_response = app
        .api
        .post("/api/admin/warehouse/stock/exit")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "50",
            "notes": "Test exit"
        }))
        .await;

    assert_eq!(exit_response.status_code(), StatusCode::CREATED);
    let body: Value = exit_response.json();

    // Check: quantity = 150, average still 10.00
    assert_eq!(body["stock"]["quantity"], "150");
    let avg = Decimal::from_str(body["stock"]["average_unit_value"].as_str().unwrap()).unwrap();
    assert_eq!(avg, Decimal::from_str("10.00").unwrap());
}

#[tokio::test]
async fn test_stock_exit_insufficient_quantity() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Insufficient Test",
            "is_personnel_exclusive": false
        }))
        .await;
    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    let material_response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Insufficient Material",
            "catmat_code": "333333",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "1.00",
            "is_active": true
        }))
        .await;
    let material_body: Value = material_response.json();
    let material_id = material_body["material"]["id"].as_str().unwrap();

    let warehouse_id = create_test_warehouse(&app).await;

    // Entry: only 10 units
    app.api
        .post("/api/admin/warehouse/stock/entry")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "10",
            "unit_value": "5.00"
        }))
        .await;

    // Try to exit 20 units (more than available)
    let exit_response = app
        .api
        .post("/api/admin/warehouse/stock/exit")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "20"
        }))
        .await;

    assert_eq!(exit_response.status_code(), StatusCode::BAD_REQUEST);
}

// ============================
// Requisition Workflow Tests
// ============================

#[tokio::test]
async fn test_requisition_workflow_complete() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    // Setup
    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Requisition Test",
            "is_personnel_exclusive": false
        }))
        .await;
    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    let material_response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Requisition Material",
            "catmat_code": "444444",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "15.00",
            "is_active": true
        }))
        .await;
    let material_body: Value = material_response.json();
    let material_id = material_body["material"]["id"].as_str().unwrap();

    let warehouse_id = create_test_warehouse(&app).await;

    // Step 1: Create requisition
    let create_response = app
        .api
        .post("/api/admin/requisitions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "items": [
                {
                    "material_id": material_id,
                    "requested_quantity": "10"
                }
            ],
            "notes": "Test requisition"
        }))
        .await;

    assert_eq!(create_response.status_code(), StatusCode::CREATED);
    let create_body: Value = create_response.json();
    let requisition_id = create_body["requisition"]["id"].as_str().unwrap();
    assert_eq!(create_body["requisition"]["status"], "PENDENTE");

    // Step 2: Approve requisition
    let approve_response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/approve", requisition_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "notes": "Approved"
        }))
        .await;

    assert_eq!(approve_response.status_code(), StatusCode::OK);
    let approve_body: Value = approve_response.json();
    assert_eq!(approve_body["requisition"]["status"], "APROVADA");

    // Step 3: Add stock to fulfill the requisition
    app.api
        .post("/api/admin/warehouse/stock/entry")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "20",
            "unit_value": "15.00"
        }))
        .await;

    // Step 4: Fulfill requisition
    let item_id = create_body["items"][0]["id"].as_str().unwrap();
    let fulfill_response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/fulfill", requisition_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "items": [
                {
                    "requisition_item_id": item_id,
                    "fulfilled_quantity": "10"
                }
            ],
            "notes": "Fulfilled completely"
        }))
        .await;

    assert_eq!(fulfill_response.status_code(), StatusCode::OK);
    let fulfill_body: Value = fulfill_response.json();
    assert_eq!(fulfill_body["requisition"]["status"], "ATENDIDA");
}

#[tokio::test]
async fn test_requisition_reject() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Reject Test",
            "is_personnel_exclusive": false
        }))
        .await;
    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    let material_response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Reject Material",
            "catmat_code": "555555",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "20.00",
            "is_active": true
        }))
        .await;
    let material_body: Value = material_response.json();
    let material_id = material_body["material"]["id"].as_str().unwrap();

    let warehouse_id = create_test_warehouse(&app).await;

    // Create requisition
    let create_response = app
        .api
        .post("/api/admin/requisitions")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "items": [
                {
                    "material_id": material_id,
                    "requested_quantity": "5"
                }
            ]
        }))
        .await;

    let create_body: Value = create_response.json();
    let requisition_id = create_body["requisition"]["id"].as_str().unwrap();

    // Reject requisition
    let reject_response = app
        .api
        .post(&format!("/api/admin/requisitions/{}/reject", requisition_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "rejection_reason": "Budget not available"
        }))
        .await;

    assert_eq!(reject_response.status_code(), StatusCode::OK);
    let reject_body: Value = reject_response.json();
    assert_eq!(reject_body["requisition"]["status"], "REJEITADA");
}

// ============================
// Reports Tests
// ============================

#[tokio::test]
async fn test_stock_value_report() {
    let app = spawn_app().await;
    let group_code = unique_code("GRP");
    let material_code = unique_code("MAT");

    // Setup: Create materials and stock
    let group_response = app
        .api
        .post("/api/admin/warehouse/material-groups")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "code": group_code,
            "name": "Report Test",
            "is_personnel_exclusive": false
        }))
        .await;
    let group_body: Value = group_response.json();
    let group_id = group_body["material_group"]["id"].as_str().unwrap();

    let material_response = app
        .api
        .post("/api/admin/warehouse/materials")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "material_group_id": group_id,
            "code": material_code,
            "name": "Report Material",
            "catmat_code": "666666",
            "unit_of_measure": "UNIDADE",
            "estimated_value": "100.00",
            "is_active": true
        }))
        .await;
    let material_body: Value = material_response.json();
    let material_id = material_body["material"]["id"].as_str().unwrap();

    let warehouse_id = create_test_warehouse(&app).await;

    // Add stock: 50 units @ R$ 100.00 = R$ 5,000.00 total
    app.api
        .post("/api/admin/warehouse/stock/entry")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "warehouse_id": warehouse_id,
            "material_id": material_id,
            "quantity": "50",
            "unit_value": "100.00"
        }))
        .await;

    // Get stock value report
    let report_response = app
        .api
        .get(&format!("/api/admin/warehouse/reports/stock-value?warehouse_id={}", warehouse_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(report_response.status_code(), StatusCode::OK);
    let body: Value = report_response.json();

    assert!(body["report"].is_array());
    let report = &body["report"][0];
    assert_eq!(report["total_items"], 1);
    assert_eq!(report["total_quantity"], "50");

    let total_value = Decimal::from_str(report["total_value"].as_str().unwrap()).unwrap();
    assert_eq!(total_value, Decimal::from_str("5000.00").unwrap());
}

#[tokio::test]
async fn test_consumption_report() {
    let app = spawn_app().await;

    // This test would require setting up stock entries and exits with specific dates
    // For now, we just test that the endpoint is accessible
    let response = app
        .api
        .get("/api/admin/warehouse/reports/consumption?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    // Should return 200 even if empty
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_most_requested_materials_report() {
    let app = spawn_app().await;

    let response = app
        .api
        .get("/api/admin/warehouse/reports/most-requested?limit=10")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert!(body["report"].is_array());
}
