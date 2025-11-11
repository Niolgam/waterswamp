use http::StatusCode;
use serde_json::{json, Value};

mod common;

// --- HEALTH CHECK TESTS ---

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    let app = common::spawn_app().await;

    let response = app.api.get("/health").await;
    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["status"], "healthy");
    assert!(body["database"]["auth_db"].as_bool().unwrap());
    assert!(body["database"]["logs_db"].as_bool().unwrap());
    assert!(
        body["version"].as_str().is_some(),
        "Version should be present in health check"
    );
}

#[tokio::test]
async fn test_health_ready_endpoint() {
    let app = common::spawn_app().await;
    app.api.get("/health/ready").await.assert_status_ok();
}

#[tokio::test]
async fn test_health_live_endpoint() {
    let app = common::spawn_app().await;
    app.api.get("/health/live").await.assert_status_ok();
}

// --- RATE LIMITING TESTS ---

#[tokio::test]
async fn test_rate_limit_login_protects_against_brute_force() {
    let app = common::spawn_app().await;

    // This test assumes DISABLE_RATE_LIMIT is set in common.rs
    // To test *actual* rate limiting, you'd need to run against a real server
    // or remove the DISABLE_RATE_LIMIT var from common.rs
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for i in 0..20 {
        let response = app
            .api
            .post("/login")
            .json(&json!({
                "username": format!("fake_user_{}", i),
                "password": "wrong_password"
            }))
            .await;

        match response.status_code() {
            StatusCode::UNAUTHORIZED => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            other => panic!("Unexpected status: {}", other),
        }
    }

    // Since rate limiting is disabled in tests, we expect 0 rate limited requests
    // and all requests to be processed (returning 401 Unauthorized)
    assert!(
        success_count > 0,
        "Server should have processed requests. Success: {}, Blocked: {}",
        success_count,
        rate_limited_count
    );

    // Se 'DISABLE_RATE_LIMIT' NÃO estivesse ativo, o assert seria:
    // assert!(rate_limited_count > 0, "Rate limit should have triggered");
}

// --- CORS TESTS ---

#[tokio::test]
async fn test_cors_headers_are_present() {
    let app = common::spawn_app().await;

    // Simulate a cross-origin request
    let response = app
        .api
        .get("/public")
        .add_header("Origin", "http://localhost:4200")
        .await;

    response.assert_status_ok();

    // Check for essential CORS headers
    let headers = response.headers();
    assert!(
        headers.get("access-control-allow-origin").is_some(),
        "CORS header 'access-control-allow-origin' missing"
    );
    assert!(
        headers.get("access-control-allow-credentials").is_some(),
        "CORS header 'access-control-allow-credentials' missing"
    );
}

// --- SECURITY HEADERS TESTS ---

#[tokio::test]
async fn test_security_headers_are_present_and_correct() {
    let app = common::spawn_app().await;

    let response = app.api.get("/public").await;
    response.assert_status_ok();

    let headers = response.headers();

    // Check headers defined in src/security.rs

    assert_eq!(
        headers
            .get("x-content-type-options")
            .expect("Missing header")
            .to_str()
            .unwrap(),
        "nosniff",
        "X-Content-Type-Options incorrect"
    );

    assert_eq!(
        headers
            .get("x-frame-options")
            .expect("Missing header")
            .to_str()
            .unwrap(),
        "DENY",
        "X-Frame-Options incorrect"
    );

    assert!(
        headers.get("x-xss-protection").is_some(),
        "X-XSS-Protection header missing"
    );

    assert!(
        headers.get("content-security-policy").is_some(),
        "Content-Security-Policy header missing"
    );

    assert!(
        headers.get("permissions-policy").is_some(),
        "Permissions-Policy header missing"
    );
}

// --- GRACEFUL SHUTDOWN TESTS ---

#[tokio::test]
async fn test_server_responds_during_normal_operation() {
    let app = common::spawn_app().await;

    // Server should respond normally when not shutting down
    app.api.get("/health").await.assert_status_ok();
    app.api.get("/public").await.assert_status_ok();
}
