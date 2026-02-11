mod common;

use common::TestApp;
use http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

// ============================
// HELPERS
// ============================

fn random_name(prefix: &str) -> String {
    format!("{}-{}", prefix, &Uuid::new_v4().simple().to_string()[..8])
}

/// Generate a valid CPF with correct check digits
fn generate_cpf() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    let mut digits: Vec<u32> = uuid
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(9)
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    while digits.len() < 9 {
        digits.push(0);
    }

    // Ensure not all same digit
    if digits.iter().all(|&d| d == digits[0]) {
        digits[8] = (digits[0] + 1) % 10;
    }

    // First check digit
    let sum1: u32 = digits.iter().enumerate().map(|(i, d)| d * (10 - i as u32)).sum();
    let check1 = {
        let rem = (sum1 * 10) % 11;
        if rem >= 10 { 0 } else { rem }
    };
    digits.push(check1);

    // Second check digit
    let sum2: u32 = digits.iter().enumerate().map(|(i, d)| d * (11 - i as u32)).sum();
    let check2 = {
        let rem = (sum2 * 10) % 11;
        if rem >= 10 { 0 } else { rem }
    };
    digits.push(check2);

    digits.iter().map(|d| d.to_string()).collect()
}

/// Generate a valid CNPJ with correct check digits
fn generate_cnpj() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    let mut digits: Vec<u32> = uuid
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(12)
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    while digits.len() < 12 {
        digits.push(0);
    }

    // Ensure not all same digit
    if digits.iter().all(|&d| d == digits[0]) {
        digits[11] = (digits[0] + 1) % 10;
    }

    // First check digit
    let weights1: Vec<u32> = vec![5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = digits.iter().zip(weights1.iter()).map(|(d, w)| d * w).sum();
    let check1 = {
        let rem = sum1 % 11;
        if rem < 2 { 0 } else { 11 - rem }
    };
    digits.push(check1);

    // Second check digit
    let weights2: Vec<u32> = vec![6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = digits.iter().zip(weights2.iter()).map(|(d, w)| d * w).sum();
    let check2 = {
        let rem = sum2 % 11;
        if rem < 2 { 0 } else { 11 - rem }
    };
    digits.push(check2);

    digits.iter().map(|d| d.to_string()).collect()
}

async fn create_supplier_individual(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "INDIVIDUAL",
            "legal_name": random_name("PF Fornecedor"),
            "document_number": generate_cpf(),
            "representative_name": "João da Silva",
            "address": "Rua das Flores, 123",
            "neighborhood": "Centro",
            "zip_code": "78000000",
            "email": format!("{}@test.com", &Uuid::new_v4().simple().to_string()[..8]),
            "phone": "(65) 99999-0000",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_supplier_legal_entity(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "LEGAL_ENTITY",
            "legal_name": random_name("PJ Empresa"),
            "trade_name": random_name("Fantasia"),
            "document_number": generate_cnpj(),
            "representative_name": "Maria Oliveira",
            "address": "Av. Brasil, 500",
            "neighborhood": "Jardim Tropical",
            "zip_code": "78050000",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

// ============================
// CRUD TESTS
// ============================

#[tokio::test]
async fn test_create_supplier_individual() {
    let app = common::spawn_app().await;
    let supplier = create_supplier_individual(&app).await;
    assert_eq!(supplier["supplier_type"], "INDIVIDUAL");
    assert!(supplier["legal_name"].as_str().is_some());
    assert!(supplier["document_number"].as_str().is_some());
}

#[tokio::test]
async fn test_create_supplier_legal_entity() {
    let app = common::spawn_app().await;
    let supplier = create_supplier_legal_entity(&app).await;
    assert_eq!(supplier["supplier_type"], "LEGAL_ENTITY");
    assert!(supplier["trade_name"].as_str().is_some());
}

#[tokio::test]
async fn test_create_supplier_government_unit() {
    let app = common::spawn_app().await;
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "GOVERNMENT_UNIT",
            "legal_name": random_name("UG Federal"),
            "document_number": format!("153072/{}", &Uuid::new_v4().simple().to_string()[..5]),
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    let supplier: Value = response.json();
    assert_eq!(supplier["supplier_type"], "GOVERNMENT_UNIT");
}

#[tokio::test]
async fn test_get_supplier() {
    let app = common::spawn_app().await;
    let created = create_supplier_individual(&app).await;
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/suppliers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let supplier: Value = response.json();
    assert_eq!(supplier["id"], id);
}

#[tokio::test]
async fn test_update_supplier() {
    let app = common::spawn_app().await;
    let created = create_supplier_legal_entity(&app).await;
    let id = created["id"].as_str().unwrap();

    let new_name = random_name("Updated");
    let response = app
        .api
        .put(&format!("/api/admin/suppliers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "trade_name": new_name,
            "email": "updated@test.com",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let updated: Value = response.json();
    assert_eq!(updated["trade_name"], new_name);
}

#[tokio::test]
async fn test_delete_supplier() {
    let app = common::spawn_app().await;
    let created = create_supplier_individual(&app).await;
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/suppliers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Should not be found after deletion
    let response = app
        .api
        .get(&format!("/api/admin/suppliers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_suppliers() {
    let app = common::spawn_app().await;

    // Create a couple suppliers first
    create_supplier_individual(&app).await;
    create_supplier_legal_entity(&app).await;

    let response = app
        .api
        .get("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    assert!(list["total"].as_i64().unwrap() >= 2);
    assert!(list["suppliers"].as_array().is_some());
}

#[tokio::test]
async fn test_list_suppliers_filter_by_type() {
    let app = common::spawn_app().await;
    create_supplier_individual(&app).await;

    let response = app
        .api
        .get("/api/admin/suppliers?supplier_type=INDIVIDUAL")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    // All returned suppliers should be INDIVIDUAL
    for s in list["suppliers"].as_array().unwrap() {
        assert_eq!(s["supplier_type"], "INDIVIDUAL");
    }
}

// ============================
// VALIDATION TESTS
// ============================

#[tokio::test]
async fn test_create_supplier_invalid_cpf() {
    let app = common::spawn_app().await;
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "INDIVIDUAL",
            "legal_name": "Teste CPF Invalido",
            "document_number": "11111111111",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_supplier_invalid_cnpj() {
    let app = common::spawn_app().await;
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "LEGAL_ENTITY",
            "legal_name": "Teste CNPJ Invalido",
            "document_number": "12345678000199",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_supplier_duplicate_document() {
    let app = common::spawn_app().await;
    let cpf = generate_cpf();

    // First creation should succeed
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "INDIVIDUAL",
            "legal_name": random_name("Dup1"),
            "document_number": &cpf,
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);

    // Second creation with same document should conflict
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "INDIVIDUAL",
            "legal_name": random_name("Dup2"),
            "document_number": &cpf,
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_supplier_formatted_cpf() {
    let app = common::spawn_app().await;
    // Use a known valid CPF with formatting
    let response = app
        .api
        .post("/api/admin/suppliers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "supplier_type": "INDIVIDUAL",
            "legal_name": random_name("Formatted"),
            "document_number": "529.982.247-25",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let supplier: Value = response.json();
    // Document should be stored normalized (digits only)
    assert_eq!(supplier["document_number"], "52998224725");
}
