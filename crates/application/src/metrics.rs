use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_histogram_vec, register_int_gauge_vec, CounterVec,
    HistogramVec, IntGaugeVec,
};

// ============================================================================
// Prometheus Metrics for SIORG Sync
// ============================================================================

lazy_static! {
    /// Total number of sync operations processed
    pub static ref SYNC_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "siorg_sync_operations_total",
        "Total number of SIORG sync operations processed",
        &["entity_type", "operation", "status"]
    )
    .expect("Failed to create sync_operations_total metric");

    /// Duration of sync operations in seconds
    pub static ref SYNC_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "siorg_sync_operation_duration_seconds",
        "Duration of SIORG sync operations in seconds",
        &["entity_type", "operation"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
    )
    .expect("Failed to create sync_operation_duration metric");

    /// Current size of the sync queue by status
    pub static ref SYNC_QUEUE_SIZE: IntGaugeVec = register_int_gauge_vec!(
        "siorg_sync_queue_size",
        "Current number of items in sync queue by status",
        &["status"]
    )
    .expect("Failed to create sync_queue_size metric");

    /// Number of conflicts detected
    pub static ref SYNC_CONFLICTS_TOTAL: CounterVec = register_counter_vec!(
        "siorg_sync_conflicts_total",
        "Total number of sync conflicts detected",
        &["entity_type"]
    )
    .expect("Failed to create sync_conflicts_total metric");

    /// Number of conflict resolutions
    pub static ref SYNC_CONFLICT_RESOLUTIONS_TOTAL: CounterVec = register_counter_vec!(
        "siorg_sync_conflict_resolutions_total",
        "Total number of conflict resolutions",
        &["action"]
    )
    .expect("Failed to create sync_conflict_resolutions_total metric");

    /// Number of retry attempts
    pub static ref SYNC_RETRY_ATTEMPTS_TOTAL: CounterVec = register_counter_vec!(
        "siorg_sync_retry_attempts_total",
        "Total number of retry attempts",
        &["entity_type", "attempt"]
    )
    .expect("Failed to create sync_retry_attempts_total metric");

    /// Worker batch processing duration
    pub static ref WORKER_BATCH_DURATION: HistogramVec = register_histogram_vec!(
        "siorg_worker_batch_duration_seconds",
        "Duration of worker batch processing in seconds",
        &["worker_id"],
        vec![0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
    )
    .expect("Failed to create worker_batch_duration metric");

    /// Worker batch size processed
    pub static ref WORKER_BATCH_SIZE: HistogramVec = register_histogram_vec!(
        "siorg_worker_batch_size",
        "Number of items processed in worker batch",
        &["worker_id"],
        vec![1.0, 5.0, 10.0, 20.0, 50.0, 100.0]
    )
    .expect("Failed to create worker_batch_size metric");

    /// API request duration
    pub static ref API_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "siorg_api_request_duration_seconds",
        "Duration of API requests in seconds",
        &["method", "endpoint", "status"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Failed to create api_request_duration metric");

    /// SIORG API call duration
    pub static ref SIORG_API_CALL_DURATION: HistogramVec = register_histogram_vec!(
        "siorg_external_api_call_duration_seconds",
        "Duration of external SIORG API calls in seconds",
        &["endpoint", "status"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
    )
    .expect("Failed to create siorg_api_call_duration metric");

    /// SIORG API call errors
    pub static ref SIORG_API_ERRORS_TOTAL: CounterVec = register_counter_vec!(
        "siorg_external_api_errors_total",
        "Total number of SIORG API call errors",
        &["endpoint", "error_type"]
    )
    .expect("Failed to create siorg_api_errors_total metric");
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Record a successful sync operation
pub fn record_sync_success(entity_type: &str, operation: &str, duration_secs: f64) {
    SYNC_OPERATIONS_TOTAL
        .with_label_values(&[entity_type, operation, "success"])
        .inc();

    SYNC_OPERATION_DURATION
        .with_label_values(&[entity_type, operation])
        .observe(duration_secs);
}

/// Record a failed sync operation
pub fn record_sync_failure(entity_type: &str, operation: &str, duration_secs: f64) {
    SYNC_OPERATIONS_TOTAL
        .with_label_values(&[entity_type, operation, "failed"])
        .inc();

    SYNC_OPERATION_DURATION
        .with_label_values(&[entity_type, operation])
        .observe(duration_secs);
}

/// Record a conflict detection
pub fn record_sync_conflict(entity_type: &str) {
    SYNC_CONFLICTS_TOTAL.with_label_values(&[entity_type]).inc();

    SYNC_OPERATIONS_TOTAL
        .with_label_values(&[entity_type, "unknown", "conflict"])
        .inc();
}

/// Record a conflict resolution
pub fn record_conflict_resolution(action: &str) {
    SYNC_CONFLICT_RESOLUTIONS_TOTAL
        .with_label_values(&[action])
        .inc();
}

/// Update queue size metrics
pub fn update_queue_size(status: &str, size: i64) {
    SYNC_QUEUE_SIZE
        .with_label_values(&[status])
        .set(size);
}

/// Record a retry attempt
pub fn record_retry_attempt(entity_type: &str, attempt: i32) {
    SYNC_RETRY_ATTEMPTS_TOTAL
        .with_label_values(&[entity_type, &attempt.to_string()])
        .inc();
}

/// Record worker batch processing
pub fn record_worker_batch(worker_id: &str, batch_size: usize, duration_secs: f64) {
    WORKER_BATCH_SIZE
        .with_label_values(&[worker_id])
        .observe(batch_size as f64);

    WORKER_BATCH_DURATION
        .with_label_values(&[worker_id])
        .observe(duration_secs);
}

/// Record SIORG API call
pub fn record_siorg_api_call(endpoint: &str, status: &str, duration_secs: f64) {
    SIORG_API_CALL_DURATION
        .with_label_values(&[endpoint, status])
        .observe(duration_secs);
}

/// Record SIORG API error
pub fn record_siorg_api_error(endpoint: &str, error_type: &str) {
    SIORG_API_ERRORS_TOTAL
        .with_label_values(&[endpoint, error_type])
        .inc();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_sync_success() {
        record_sync_success("ORGANIZATION", "CREATE", 1.5);
        // Metric should be recorded (can't easily assert in tests without prometheus test helpers)
    }

    #[test]
    fn test_update_queue_size() {
        update_queue_size("PENDING", 42);
        // Metric should be recorded
    }
}
