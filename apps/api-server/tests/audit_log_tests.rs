// =============================================================================
// TESTS FOR AUDIT LOG SYSTEM
// Add to apps/api-server/tests/audit_log_tests.rs
// =============================================================================

mod common;

use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn test_list_audit_logs_empty() {
    let app = common::spawn_app().await;

    // List audit logs (should be empty or have only setup logs)
    let response = app
        .api
        .get("/api/admin/audit-logs")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["logs"].is_array());
    assert!(body["total"].is_i64());
    assert!(body["limit"].is_i64());
    assert!(body["offset"].is_i64());
}

#[tokio::test]
async fn test_list_audit_logs_with_pagination() {
    let app = common::spawn_app().await;

    // Create some audit log entries first
    for i in 0..5 {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (user_id, action, resource, method, status_code)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_action_{}", i))
        .bind("/test/resource")
        .bind("GET")
        .bind(200)
        .execute(&app.db_logs)
        .await
        .unwrap();
    }

    // Test pagination
    let response = app
        .api
        .get("/api/admin/audit-logs?limit=2&offset=0")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["limit"], 2);
    assert_eq!(body["offset"], 0);
    assert!(body["logs"].as_array().unwrap().len() <= 2);
}

#[tokio::test]
async fn test_list_audit_logs_with_filters() {
    let app = common::spawn_app().await;

    let user_id = Uuid::new_v4();

    // Create test entries
    sqlx::query(
        r#"
        INSERT INTO audit_logs (user_id, username, action, resource, method, status_code, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6, $7::INET)
        "#,
    )
    .bind(user_id)
    .bind("test_user")
    .bind("login")
    .bind("/login")
    .bind("POST")
    .bind(200)
    .bind("192.168.1.1")
    .execute(&app.db_logs)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO audit_logs (user_id, username, action, resource, method, status_code, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6, $7::INET)
        "#,
    )
    .bind(user_id)
    .bind("test_user")
    .bind("login_failed")
    .bind("/login")
    .bind("POST")
    .bind(401)
    .bind("192.168.1.1")
    .execute(&app.db_logs)
    .await
    .unwrap();

    // Filter by action
    let response = app
        .api
        .get(&format!("/api/admin/audit-logs?action=login_failed"))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body["logs"].as_array().unwrap();
    for log in logs {
        assert_eq!(log["action"], "login_failed");
    }

    // Filter by user_id
    let response = app
        .api
        .get(&format!("/api/admin/audit-logs?user_id={}", user_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body["logs"].as_array().unwrap();
    assert!(logs.len() >= 2);

    // Filter by status code range (errors only)
    let response = app
        .api
        .get("/api/admin/audit-logs?min_status_code=400")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body["logs"].as_array().unwrap();
    for log in logs {
        assert!(log["status_code"].as_i64().unwrap_or(0) >= 400);
    }
}

#[tokio::test]
async fn test_get_audit_log_by_id() {
    let app = common::spawn_app().await;

    // Create a test entry
    let log_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO audit_logs (action, resource, method, status_code)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind("test_action")
    .bind("/test")
    .bind("GET")
    .bind(200)
    .fetch_one(&app.db_logs)
    .await
    .unwrap();

    // Get by ID
    let response = app
        .api
        .get(&format!("/api/admin/audit-logs/{}", log_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["id"], log_id.to_string());
    assert_eq!(body["action"], "test_action");
}

#[tokio::test]
async fn test_get_audit_log_not_found() {
    let app = common::spawn_app().await;

    let fake_id = Uuid::new_v4();

    let response = app
        .api
        .get(&format!("/api/admin/audit-logs/{}", fake_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_get_audit_stats() {
    let app = common::spawn_app().await;

    // Create some test data
    sqlx::query(
        r#"
        INSERT INTO audit_logs (action, resource)
        VALUES ($1, $2)
        "#,
    )
    .bind("login")
    .bind("/login")
    .execute(&app.db_logs)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO audit_logs (action, resource)
        VALUES ($1, $2)
        "#,
    )
    .bind("login_failed")
    .bind("/login")
    .execute(&app.db_logs)
    .await
    .unwrap();

    let response = app
        .api
        .get("/api/admin/audit-logs/stats")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["total_logs"].is_i64());
    assert!(body["logs_today"].is_i64());
    assert!(body["logs_this_week"].is_i64());
    assert!(body["failed_logins_today"].is_i64());
    assert!(body["unique_users_today"].is_i64());
    assert!(body["top_actions"].is_array());
    assert!(body["top_resources"].is_array());
}

#[tokio::test]
async fn test_get_failed_logins() {
    let app = common::spawn_app().await;

    // Create failed login entries
    for _ in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (username, action, resource, status_code, ip_address)
            VALUES ($1, $2, $3, $4, $5::INET)
            "#,
        )
        .bind("hacker")
        .bind("login_failed")
        .bind("/login")
        .bind(401)
        .bind("10.0.0.1")
        .execute(&app.db_logs)
        .await
        .unwrap();
    }

    let response = app
        .api
        .get("/api/admin/audit-logs/failed-logins?hours=24&limit=10")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body.as_array().unwrap();
    assert!(logs.len() >= 3);

    for log in logs {
        assert_eq!(log["action"], "login_failed");
    }
}

#[tokio::test]
async fn test_get_suspicious_ips() {
    let app = common::spawn_app().await;

    // Simulate brute force attack from one IP
    let attacker_ip = "10.20.30.40";

    for i in 0..10 {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (username, action, resource, status_code, ip_address)
            VALUES ($1, $2, $3, $4, $5::INET)
            "#,
        )
        .bind(format!("user{}", i))
        .bind("login_failed")
        .bind("/login")
        .bind(401)
        .bind(attacker_ip)
        .execute(&app.db_logs)
        .await
        .unwrap();
    }

    let response = app
        .api
        .get("/api/admin/audit-logs/suspicious-ips?hours=24&threshold=5")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let suspicious = body.as_array().unwrap();

    // Should find our attacker IP
    let attacker = suspicious
        .iter()
        .find(|ip| ip["ip_address"].as_str().unwrap_or("") == attacker_ip);

    assert!(attacker.is_some(), "Should detect attacker IP");

    let attacker = attacker.unwrap();
    assert!(attacker["failed_attempts"].as_i64().unwrap() >= 10);
    assert!(attacker["unique_usernames"].as_i64().unwrap() >= 5);
}

#[tokio::test]
async fn test_get_user_audit_logs() {
    let app = common::spawn_app().await;

    let user_id = Uuid::new_v4();

    // Create entries for this user
    for action in ["login", "logout", "token_refresh"].iter() {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (user_id, username, action, resource)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind("specific_user")
        .bind(action)
        .bind(format!("/{}", action))
        .execute(&app.db_logs)
        .await
        .unwrap();
    }

    let response = app
        .api
        .get(&format!("/api/admin/audit-logs/user/{}?limit=50", user_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body.as_array().unwrap();
    assert!(logs.len() >= 3);

    // All logs should belong to this user
    for log in logs {
        assert_eq!(log["user_id"], user_id.to_string());
    }
}

#[tokio::test]
async fn test_cleanup_old_logs() {
    let app = common::spawn_app().await;

    // Create an old log entry (manually set old timestamp)
    sqlx::query(
        r#"
        INSERT INTO audit_logs (action, resource, created_at)
        VALUES ($1, $2, NOW() - INTERVAL '100 days')
        "#,
    )
    .bind("old_action")
    .bind("/old")
    .execute(&app.db_logs)
    .await
    .unwrap();

    // Create a recent log entry
    sqlx::query(
        r#"
        INSERT INTO audit_logs (action, resource)
        VALUES ($1, $2)
        "#,
    )
    .bind("recent_action")
    .bind("/recent")
    .execute(&app.db_logs)
    .await
    .unwrap();

    // Cleanup logs older than 90 days
    let response = app
        .api
        .post("/api/admin/audit-logs/cleanup")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .json(&json!({
            "retention_days": 90
        }))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["deleted_count"].as_i64().unwrap() >= 1);
    assert_eq!(body["retention_days"], 90);

    // Verify old log is gone
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_logs WHERE action = 'old_action'
        "#,
    )
    .fetch_one(&app.db_logs)
    .await
    .unwrap();

    assert_eq!(count, 0, "Old log should be deleted");

    // Verify recent log still exists
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_logs WHERE action = 'recent_action'
        "#,
    )
    .fetch_one(&app.db_logs)
    .await
    .unwrap();

    assert_eq!(count, 1, "Recent log should still exist");
}

#[tokio::test]
async fn test_audit_logs_sorting() {
    let app = common::spawn_app().await;

    // Create entries with different timestamps
    sqlx::query(
        r#"
        INSERT INTO audit_logs (action, resource, created_at)
        VALUES 
            ('action_1', '/test', NOW() - INTERVAL '3 hours'),
            ('action_2', '/test', NOW() - INTERVAL '2 hours'),
            ('action_3', '/test', NOW() - INTERVAL '1 hour')
        "#,
    )
    .execute(&app.db_logs)
    .await
    .unwrap();

    // Sort ascending (oldest first)
    let response = app
        .api
        .get("/api/admin/audit-logs?sort_order=asc&sort_by=created_at")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body["logs"].as_array().unwrap();

    if logs.len() >= 2 {
        let first_time = logs[0]["created_at"].as_str().unwrap();
        let second_time = logs[1]["created_at"].as_str().unwrap();
        assert!(
            first_time <= second_time,
            "Should be sorted in ascending order"
        );
    }

    // Sort descending (newest first) - default
    let response = app
        .api
        .get("/api/admin/audit-logs?sort_order=desc&sort_by=created_at")
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    let logs = body["logs"].as_array().unwrap();

    if logs.len() >= 2 {
        let first_time = logs[0]["created_at"].as_str().unwrap();
        let second_time = logs[1]["created_at"].as_str().unwrap();
        assert!(
            first_time >= second_time,
            "Should be sorted in descending order"
        );
    }
}

#[tokio::test]
async fn test_unauthorized_access_to_audit_logs() {
    let app = common::spawn_app().await;

    // Try to access without token
    let response = app.api.get("/api/admin/audit-logs").await;

    assert_eq!(response.status_code(), 401);

    // Try to access with user token (not admin)
    let response = app
        .api
        .get("/api/admin/audit-logs")
        .add_header("Authorization", format!("Bearer {}", app.user_token))
        .await;

    assert_eq!(response.status_code(), 403);
}

#[tokio::test]
async fn test_audit_log_details_json() {
    let app = common::spawn_app().await;

    let details = serde_json::json!({
        "old_role": "user",
        "new_role": "admin",
        "changed_by": "alice"
    });

    let log_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO audit_logs (action, resource, details)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind("user_role_changed")
    .bind("/api/admin/users/123")
    .bind(&details)
    .fetch_one(&app.db_logs)
    .await
    .unwrap();

    let response = app
        .api
        .get(&format!("/api/admin/audit-logs/{}", log_id))
        .add_header("Authorization", format!("Bearer {}", app.admin_token))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["details"]["old_role"], "user");
    assert_eq!(body["details"]["new_role"], "admin");
}

// =============================================================================
// INTEGRATION TEST: Verify middleware is logging
// =============================================================================

#[tokio::test]
async fn test_middleware_logs_login_attempt() {
    let app = common::spawn_app().await;

    // Clear existing logs
    sqlx::query("DELETE FROM audit_logs")
        .execute(&app.db_logs)
        .await
        .ok();

    // Perform a login (this should be logged by middleware)
    let _login_response = app
        .api
        .post("/login")
        .json(&json!({
            "username": "alice",
            "password": "password123"
        }))
        .await;

    // Give middleware time to write log
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Check if login was logged
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_logs 
        WHERE action = 'login' OR resource = '/login'
        "#,
    )
    .fetch_one(&app.db_logs)
    .await
    .unwrap();

    assert!(count >= 1, "Login should be logged by middleware");
}
