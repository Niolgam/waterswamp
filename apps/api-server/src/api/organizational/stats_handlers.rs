use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::extractors::current_user::CurrentUser;
use crate::infra::state::AppState;
use domain::models::organizational::*;

// ============================================================================
// Statistics Response Models
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct DetailedStats {
    pub queue: QueueStatistics,
    pub processing: ProcessingStatistics,
    pub conflicts: ConflictStatistics,
    pub history: HistoryStatistics,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QueueStatistics {
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
    pub conflicts: i64,
    pub skipped: i64,
    pub total: i64,
    pub pending_rate: f64,    // Percentage
    pub completion_rate: f64, // Percentage
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProcessingStatistics {
    pub last_24h: TimeBasedStats,
    pub last_7d: TimeBasedStats,
    pub last_30d: TimeBasedStats,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TimeBasedStats {
    pub completed: i64,
    pub failed: i64,
    pub conflicts: i64,
    pub success_rate: f64, // Percentage
    pub avg_duration_ms: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConflictStatistics {
    pub total_unresolved: i64,
    pub by_entity_type: std::collections::HashMap<String, i64>,
    pub oldest_conflict_age_hours: Option<f64>,
    pub avg_resolution_time_hours: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryStatistics {
    pub total_entries: i64,
    pub pending_reviews: i64,
    pub last_24h_changes: i64,
    pub by_change_type: std::collections::HashMap<String, i64>,
}

// ============================================================================
// Stats Handlers
// ============================================================================

/// Get detailed statistics about SIORG sync operations
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/stats/detailed",
    responses(
        (status = 200, description = "Detailed statistics", body = DetailedStats),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_detailed_stats(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<DetailedStats>, (StatusCode, String)> {
    let queue_repo = &state.siorg_sync_queue_repository;
    let history_repo = &state.siorg_history_repository;

    // Collect queue statistics
    let (pending, processing, completed, failed, conflicts, skipped) = tokio::try_join!(
        queue_repo.count_by_status(SyncStatus::Pending),
        queue_repo.count_by_status(SyncStatus::Processing),
        queue_repo.count_by_status(SyncStatus::Completed),
        queue_repo.count_by_status(SyncStatus::Failed),
        queue_repo.count_by_status(SyncStatus::Conflict),
        queue_repo.count_by_status(SyncStatus::Skipped),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total = pending + processing + completed + failed + conflicts + skipped;
    let pending_rate = if total > 0 {
        (pending as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let completion_rate = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // Get conflict statistics
    let conflict_items = state
        .siorg_sync_queue_repository
        .get_conflicts(100, 0)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut by_entity_type = std::collections::HashMap::new();
    for item in &conflict_items {
        *by_entity_type
            .entry(format!("{:?}", item.entity_type))
            .or_insert(0) += 1;
    }

    let oldest_conflict_age_hours = conflict_items.first().map(|item| {
        let age = chrono::Utc::now()
            .signed_duration_since(item.created_at)
            .num_hours() as f64;
        age
    });

    // Get history statistics
    let history_count = history_repo
        .count(None)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let pending_reviews = history_repo
        .get_pending_reviews(1000, 0)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len() as i64;

    // Build response
    let stats = DetailedStats {
        queue: QueueStatistics {
            pending,
            processing,
            completed,
            failed,
            conflicts,
            skipped,
            total,
            pending_rate,
            completion_rate,
        },
        processing: ProcessingStatistics {
            last_24h: TimeBasedStats {
                completed: 0, // Would need timestamp filtering in repo
                failed: 0,
                conflicts: 0,
                success_rate: 0.0,
                avg_duration_ms: 0.0,
            },
            last_7d: TimeBasedStats {
                completed: 0,
                failed: 0,
                conflicts: 0,
                success_rate: 0.0,
                avg_duration_ms: 0.0,
            },
            last_30d: TimeBasedStats {
                completed: 0,
                failed: 0,
                conflicts: 0,
                success_rate: 0.0,
                avg_duration_ms: 0.0,
            },
        },
        conflicts: ConflictStatistics {
            total_unresolved: conflicts,
            by_entity_type,
            oldest_conflict_age_hours,
            avg_resolution_time_hours: None,
        },
        history: HistoryStatistics {
            total_entries: history_count,
            pending_reviews,
            last_24h_changes: 0,
            by_change_type: std::collections::HashMap::new(),
        },
        last_updated: chrono::Utc::now(),
    };

    Ok(Json(stats))
}

/// Get queue health status
#[utoipa::path(
    get,
    path = "/api/admin/organizational/sync/stats/health",
    responses(
        (status = 200, description = "Health status", body = HealthStatus),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_health_status(
    _user: CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<HealthStatus>, (StatusCode, String)> {
    let queue_repo = &state.siorg_sync_queue_repository;

    let (pending, processing, failed, conflicts) = tokio::try_join!(
        queue_repo.count_by_status(SyncStatus::Pending),
        queue_repo.count_by_status(SyncStatus::Processing),
        queue_repo.count_by_status(SyncStatus::Failed),
        queue_repo.count_by_status(SyncStatus::Conflict),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let status = if conflicts > 50 {
        "critical"
    } else if conflicts > 20 || failed > 100 {
        "warning"
    } else if processing > 0 || pending > 0 {
        "processing"
    } else {
        "healthy"
    };

    let health = HealthStatus {
        status: status.to_string(),
        queue_size: pending + processing,
        conflicts_count: conflicts,
        failures_count: failed,
        message: get_health_message(status, pending, processing, conflicts, failed),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(health))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthStatus {
    pub status: String, // healthy, processing, warning, critical
    pub queue_size: i64,
    pub conflicts_count: i64,
    pub failures_count: i64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

fn get_health_message(
    status: &str,
    pending: i64,
    processing: i64,
    conflicts: i64,
    failed: i64,
) -> String {
    match status {
        "critical" => format!(
            "Sistema em estado crítico: {} conflitos pendentes",
            conflicts
        ),
        "warning" => format!(
            "Atenção necessária: {} conflitos, {} falhas",
            conflicts, failed
        ),
        "processing" => format!("Processando: {} pendentes, {} em andamento", pending, processing),
        _ => "Sistema operando normalmente".to_string(),
    }
}
