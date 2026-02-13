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

fn generate_cnh() -> String {
    format!("{}", &Uuid::new_v4().simple().to_string()[..11])
}

async fn create_outsourced_driver(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "OUTSOURCED",
            "full_name": random_name("Motorista Terceirizado"),
            "cpf": generate_cpf(),
            "cnh_number": generate_cnh(),
            "cnh_category": "D",
            "cnh_expiration": "2027-12-31",
            "phone": "(65) 99999-1111",
            "email": format!("{}@test.com", &Uuid::new_v4().simple().to_string()[..8]),
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

async fn create_server_driver(app: &TestApp) -> Value {
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "SERVER",
            "full_name": random_name("Servidor Motorista"),
            "cpf": generate_cpf(),
            "cnh_number": generate_cnh(),
            "cnh_category": "B",
            "cnh_expiration": "2028-06-15",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED, "Failed: {}", response.text());
    response.json()
}

// ============================
// CRUD TESTS
// ============================

#[tokio::test]
async fn test_create_driver_outsourced() {
    let app = common::spawn_app().await;
    let driver = create_outsourced_driver(&app).await;
    assert_eq!(driver["driver_type"], "OUTSOURCED");
    assert!(driver["full_name"].as_str().is_some());
    assert!(driver["cpf"].as_str().is_some());
    assert_eq!(driver["cnh_category"], "D");
}

#[tokio::test]
async fn test_create_driver_server() {
    let app = common::spawn_app().await;
    let driver = create_server_driver(&app).await;
    assert_eq!(driver["driver_type"], "SERVER");
    assert_eq!(driver["cnh_category"], "B");
}

#[tokio::test]
async fn test_get_driver() {
    let app = common::spawn_app().await;
    let created = create_outsourced_driver(&app).await;
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/drivers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let driver: Value = response.json();
    assert_eq!(driver["id"], id);
}

#[tokio::test]
async fn test_update_driver() {
    let app = common::spawn_app().await;
    let created = create_server_driver(&app).await;
    let id = created["id"].as_str().unwrap();

    let new_name = random_name("Atualizado");
    let response = app
        .api
        .put(&format!("/api/admin/drivers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "full_name": new_name,
            "cnh_category": "D",
            "phone": "(65) 98888-0000",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let updated: Value = response.json();
    assert_eq!(updated["full_name"], new_name);
    assert_eq!(updated["cnh_category"], "D");
}

#[tokio::test]
async fn test_delete_driver() {
    let app = common::spawn_app().await;
    let created = create_outsourced_driver(&app).await;
    let id = created["id"].as_str().unwrap();

    let response = app
        .api
        .delete(&format!("/api/admin/drivers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // Should not be found after deletion
    let response = app
        .api
        .get(&format!("/api/admin/drivers/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_drivers() {
    let app = common::spawn_app().await;

    create_outsourced_driver(&app).await;
    create_server_driver(&app).await;

    let response = app
        .api
        .get("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    assert!(list["total"].as_i64().unwrap() >= 2);
    assert!(list["drivers"].as_array().is_some());
}

#[tokio::test]
async fn test_list_drivers_filter_by_type() {
    let app = common::spawn_app().await;
    create_outsourced_driver(&app).await;
    create_server_driver(&app).await;

    let response = app
        .api
        .get("/api/admin/drivers?driver_type=OUTSOURCED")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let list: Value = response.json();
    for d in list["drivers"].as_array().unwrap() {
        assert_eq!(d["driver_type"], "OUTSOURCED");
    }
}

// ============================
// VALIDATION TESTS
// ============================

#[tokio::test]
async fn test_create_driver_invalid_cpf() {
    let app = common::spawn_app().await;
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "SERVER",
            "full_name": "Teste CPF Invalido",
            "cpf": "11111111111",
            "cnh_number": generate_cnh(),
            "cnh_category": "B",
            "cnh_expiration": "2027-01-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_driver_invalid_cnh_category() {
    let app = common::spawn_app().await;
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "SERVER",
            "full_name": "Teste Categoria Invalida",
            "cpf": generate_cpf(),
            "cnh_number": generate_cnh(),
            "cnh_category": "Z",
            "cnh_expiration": "2027-01-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_driver_duplicate_cpf() {
    let app = common::spawn_app().await;
    let cpf = generate_cpf();

    // First creation should succeed
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "OUTSOURCED",
            "full_name": random_name("Dup1"),
            "cpf": &cpf,
            "cnh_number": generate_cnh(),
            "cnh_category": "B",
            "cnh_expiration": "2027-01-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);

    // Second creation with same CPF should conflict
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "SERVER",
            "full_name": random_name("Dup2"),
            "cpf": &cpf,
            "cnh_number": generate_cnh(),
            "cnh_category": "D",
            "cnh_expiration": "2027-06-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_driver_duplicate_cnh() {
    let app = common::spawn_app().await;
    let cnh = generate_cnh();

    // First creation should succeed
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "OUTSOURCED",
            "full_name": random_name("CnhDup1"),
            "cpf": generate_cpf(),
            "cnh_number": &cnh,
            "cnh_category": "B",
            "cnh_expiration": "2027-01-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);

    // Second creation with same CNH should conflict
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "SERVER",
            "full_name": random_name("CnhDup2"),
            "cpf": generate_cpf(),
            "cnh_number": &cnh,
            "cnh_category": "D",
            "cnh_expiration": "2027-06-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_driver_formatted_cpf() {
    let app = common::spawn_app().await;
    let raw_cpf = generate_cpf();
    let formatted_cpf = format!(
        "{}.{}.{}-{}",
        &raw_cpf[0..3],
        &raw_cpf[3..6],
        &raw_cpf[6..9],
        &raw_cpf[9..11]
    );
    let response = app
        .api
        .post("/api/admin/drivers")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "driver_type": "OUTSOURCED",
            "full_name": random_name("CPF Formatado"),
            "cpf": &formatted_cpf,
            "cnh_number": generate_cnh(),
            "cnh_category": "E",
            "cnh_expiration": "2028-01-01",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::CREATED);
    let driver: Value = response.json();
    // CPF should be stored normalized (digits only)
    assert_eq!(driver["cpf"], raw_cpf);
}
