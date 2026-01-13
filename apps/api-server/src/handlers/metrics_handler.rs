use axum::{http::StatusCode, response::IntoResponse};
use prometheus::{Encoder, TextEncoder};

/// Handler for Prometheus metrics endpoint
///
/// Returns metrics in Prometheus text format
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = String),
    ),
    tag = "Metrics"
)]
pub async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => {
            let metrics_text = String::from_utf8(buffer).unwrap_or_else(|_| {
                "# Failed to encode metrics as UTF-8\n".to_string()
            });

            (
                StatusCode::OK,
                [("content-type", "text/plain; version=0.0.4")],
                metrics_text,
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "text/plain")],
            format!("# Error encoding metrics: {}\n", e),
        ),
    }
}

/// Handler for metrics with queue size updates
///
/// Updates queue size metrics before returning all metrics
pub async fn metrics_with_queue_stats(
    state: axum::extract::State<crate::infra::state::AppState>,
) -> impl IntoResponse {
    // Update queue size metrics
    let repo = &state.siorg_sync_queue_repository;

    // Update all queue status metrics
    let statuses = [
        ("PENDING", domain::models::organizational::SyncStatus::Pending),
        ("PROCESSING", domain::models::organizational::SyncStatus::Processing),
        ("COMPLETED", domain::models::organizational::SyncStatus::Completed),
        ("FAILED", domain::models::organizational::SyncStatus::Failed),
        ("CONFLICT", domain::models::organizational::SyncStatus::Conflict),
        ("SKIPPED", domain::models::organizational::SyncStatus::Skipped),
    ];

    for (label, status) in statuses {
        if let Ok(count) = repo.count_by_status(status).await {
            application::metrics::update_queue_size(label, count);
        }
    }

    // Return metrics
    metrics().await
}
