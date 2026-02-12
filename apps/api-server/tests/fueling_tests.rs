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
    if digits.iter().all(|&d| d == digits[0]) {
        digits[8] = (digits[0] + 1) % 10;
    }
    let sum1: u32 = digits.iter().enumerate().map(|(i, d)| d * (10 - i as u32)).sum();
    let check1 = { let rem = (sum1 * 10) % 11; if rem >= 10 { 0 } else { rem } };
    digits.push(check1);
    let sum2: u32 = digits.iter().enumerate().map(|(i, d)| d * (11 - i as u32)).sum();
    let check2 = { let rem = (sum2 * 10) % 11; if rem >= 10 { 0 } else { rem } };
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
    if digits.iter().all(|&d| d == digits[0]) {
        digits[11] = (digits[0] + 1) % 10;
    }
    let weights1: Vec<u32> = vec![5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = digits.iter().zip(weights1.iter()).map(|(d, w)| d * w).sum();
    let check1 = { let rem = sum1 % 11; if rem < 2 { 0 } else { 11 - rem } };
    digits.push(check1);
    let weights2: Vec<u32> = vec![6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = digits.iter().zip(weights2.iter()).map(|(d, w)| d * w).sum();
    let check2 = { let rem = sum2 % 11; if rem < 2 { 0 } else { 11 - rem } };
    digits.push(check2);
    digits.iter().map(|d| d.to_string()).collect()
}

fn random_plate() -> String {
    let hex = Uuid::new_v4().simple().to_string();
    let chars: Vec<char> = hex.chars().collect();
    format!(
        "{}{}{}{}{}{}{}",
        chars[0].to_uppercase().next().unwrap_or('A'),
        chars[1].to_uppercase().next().unwrap_or('B'),
        chars[2].to_uppercase().next().unwrap_or('C'),
        chars[3].to_digit(16).unwrap_or(1),
        chars[4].to_uppercase().next().unwrap_or('D'),
        chars[5].to_digit(16).unwrap_or(2),
        chars[6].to_digit(16).unwrap_or(3),
    )
}

fn random_chassis() -> String {
    Uuid::new_v4().simple().to_string()[..17].to_uppercase()
}

fn random_renavam() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    uuid.chars().filter(|c| c.is_ascii_digit()).take(11).collect::<String>()
}

/// Setup all prerequisite data and return (vehicle_id, driver_id, supplier_id, fuel_type_id)
async fn setup_prerequisites(app: &TestApp) -> (String, String, String, String) {
    let auth = format!("Bearer {}", app.admin_token);

    // Create fuel type
    let resp = app.api.post("/api/admin/fleet/fuel-types")
        .add_header("Authorization", &auth)
        .json(&json!({ "name": random_name("Diesel") }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "fuel_type: {}", resp.text());
    let fuel_type: Value = resp.json();
    let fuel_type_id = fuel_type["id"].as_str().unwrap().to_string();

    // Create vehicle make
    let resp = app.api.post("/api/admin/fleet/makes")
        .add_header("Authorization", &auth)
        .json(&json!({ "name": random_name("Toyota") }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "make: {}", resp.text());
    let make: Value = resp.json();
    let make_id = make["id"].as_str().unwrap();

    // Create vehicle model
    let resp = app.api.post("/api/admin/fleet/models")
        .add_header("Authorization", &auth)
        .json(&json!({ "make_id": make_id, "name": random_name("Hilux") }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "model: {}", resp.text());
    let model: Value = resp.json();
    let model_id = model["id"].as_str().unwrap();

    // Create vehicle color
    let resp = app.api.post("/api/admin/fleet/colors")
        .add_header("Authorization", &auth)
        .json(&json!({ "name": random_name("Branco") }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "color: {}", resp.text());
    let color: Value = resp.json();
    let color_id = color["id"].as_str().unwrap();

    // Create vehicle
    let resp = app.api.post("/api/admin/fleet/vehicles")
        .add_header("Authorization", &auth)
        .json(&json!({
            "license_plate": random_plate(),
            "chassis_number": random_chassis(),
            "renavam": random_renavam(),
            "model_id": model_id,
            "color_id": color_id,
            "fuel_type_id": &fuel_type_id,
            "manufacture_year": 2024,
            "model_year": 2025,
            "acquisition_type": "PURCHASE",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "vehicle: {}", resp.text());
    let vehicle: Value = resp.json();
    let vehicle_id = vehicle["id"].as_str().unwrap().to_string();

    // Create driver
    let resp = app.api.post("/api/admin/drivers")
        .add_header("Authorization", &auth)
        .json(&json!({
            "driver_type": "SERVER",
            "full_name": random_name("Motorista"),
            "cpf": generate_cpf(),
            "cnh_number": &Uuid::new_v4().simple().to_string()[..11],
            "cnh_category": "D",
            "cnh_expiration": "2028-12-31",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "driver: {}", resp.text());
    let driver: Value = resp.json();
    let driver_id = driver["id"].as_str().unwrap().to_string();

    // Create supplier (gas station)
    let resp = app.api.post("/api/admin/suppliers")
        .add_header("Authorization", &auth)
        .json(&json!({
            "legal_name": random_name("Posto"),
            "document_number": generate_cnpj(),
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "supplier: {}", resp.text());
    let supplier: Value = resp.json();
    let supplier_id = supplier["id"].as_str().unwrap().to_string();

    (vehicle_id, driver_id, supplier_id, fuel_type_id)
}

async fn create_fueling(app: &TestApp, vehicle_id: &str, driver_id: &str, supplier_id: &str, fuel_type_id: &str) -> Value {
    let resp = app.api.post("/api/admin/fuelings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "vehicle_id": vehicle_id,
            "driver_id": driver_id,
            "supplier_id": supplier_id,
            "fuel_type_id": fuel_type_id,
            "fueling_date": "2026-02-11T10:30:00Z",
            "odometer_km": 45200,
            "quantity_liters": 55.5,
            "unit_price": 6.299,
            "total_cost": 349.59,
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "fueling: {}", resp.text());
    resp.json()
}

// ============================
// CRUD TESTS
// ============================

#[tokio::test]
async fn test_create_fueling() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    let fueling = create_fueling(&app, &vid, &did, &sid, &ftid).await;

    assert_eq!(fueling["vehicle_id"], vid);
    assert_eq!(fueling["driver_id"], did);
    assert_eq!(fueling["supplier_id"], sid);
    assert_eq!(fueling["fuel_type_id"], ftid);
    assert_eq!(fueling["odometer_km"], 45200);
    // Verify joined detail fields
    assert!(fueling["vehicle_license_plate"].as_str().is_some());
    assert!(fueling["driver_name"].as_str().is_some());
    assert!(fueling["supplier_name"].as_str().is_some());
    assert!(fueling["fuel_type_name"].as_str().is_some());
}

#[tokio::test]
async fn test_create_fueling_without_supplier() {
    let app = common::spawn_app().await;
    let (vid, did, _sid, ftid) = setup_prerequisites(&app).await;

    let resp = app.api.post("/api/admin/fuelings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "vehicle_id": &vid,
            "driver_id": &did,
            "fuel_type_id": &ftid,
            "fueling_date": "2026-02-11T14:00:00Z",
            "odometer_km": 46000,
            "quantity_liters": 40.0,
            "unit_price": 5.89,
            "total_cost": 235.60,
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    let fueling: Value = resp.json();
    assert!(fueling["supplier_id"].is_null());
}

#[tokio::test]
async fn test_get_fueling() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    let created = create_fueling(&app, &vid, &did, &sid, &ftid).await;
    let id = created["id"].as_str().unwrap();

    let resp = app.api.get(&format!("/api/admin/fuelings/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let fueling: Value = resp.json();
    assert_eq!(fueling["id"], id);
}

#[tokio::test]
async fn test_update_fueling() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    let created = create_fueling(&app, &vid, &did, &sid, &ftid).await;
    let id = created["id"].as_str().unwrap();

    let resp = app.api.put(&format!("/api/admin/fuelings/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "odometer_km": 46500,
            "notes": "Abastecimento corrigido",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let updated: Value = resp.json();
    assert_eq!(updated["odometer_km"], 46500);
    assert_eq!(updated["notes"], "Abastecimento corrigido");
}

#[tokio::test]
async fn test_delete_fueling() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    let created = create_fueling(&app, &vid, &did, &sid, &ftid).await;
    let id = created["id"].as_str().unwrap();

    let resp = app.api.delete(&format!("/api/admin/fuelings/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);

    let resp = app.api.get(&format!("/api/admin/fuelings/{}", id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_fuelings() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    create_fueling(&app, &vid, &did, &sid, &ftid).await;

    let resp = app.api.get("/api/admin/fuelings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let list: Value = resp.json();
    assert!(list["total"].as_i64().unwrap() >= 1);
    assert!(list["fuelings"].as_array().is_some());
}

#[tokio::test]
async fn test_list_fuelings_filter_by_vehicle() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    create_fueling(&app, &vid, &did, &sid, &ftid).await;

    let resp = app.api.get(&format!("/api/admin/fuelings?vehicle_id={}", vid))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let list: Value = resp.json();
    for f in list["fuelings"].as_array().unwrap() {
        assert_eq!(f["vehicle_id"], vid);
    }
}

#[tokio::test]
async fn test_list_fuelings_filter_by_driver() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;
    create_fueling(&app, &vid, &did, &sid, &ftid).await;

    let resp = app.api.get(&format!("/api/admin/fuelings?driver_id={}", did))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let list: Value = resp.json();
    for f in list["fuelings"].as_array().unwrap() {
        assert_eq!(f["driver_id"], did);
    }
}

// ============================
// VALIDATION TESTS
// ============================

#[tokio::test]
async fn test_create_fueling_negative_quantity() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;

    let resp = app.api.post("/api/admin/fuelings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "vehicle_id": &vid,
            "driver_id": &did,
            "supplier_id": &sid,
            "fuel_type_id": &ftid,
            "fueling_date": "2026-02-11T10:00:00Z",
            "odometer_km": 45000,
            "quantity_liters": -10.0,
            "unit_price": 6.29,
            "total_cost": 62.90,
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_fueling_negative_odometer() {
    let app = common::spawn_app().await;
    let (vid, did, sid, ftid) = setup_prerequisites(&app).await;

    let resp = app.api.post("/api/admin/fuelings")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "vehicle_id": &vid,
            "driver_id": &did,
            "supplier_id": &sid,
            "fuel_type_id": &ftid,
            "fueling_date": "2026-02-11T10:00:00Z",
            "odometer_km": -100,
            "quantity_liters": 50.0,
            "unit_price": 6.29,
            "total_cost": 314.50,
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_fueling_not_found() {
    let app = common::spawn_app().await;
    let fake_id = Uuid::new_v4();

    let resp = app.api.get(&format!("/api/admin/fuelings/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
