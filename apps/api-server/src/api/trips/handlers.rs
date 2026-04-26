use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use application::errors::ServiceError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

fn occ_response(msg: String) -> axum::response::Response {
    use axum::response::IntoResponse;
    (
        StatusCode::CONFLICT,
        Json(serde_json::json!({
            "type": "optimistic-lock-failure",
            "title": "Conflict",
            "status": 409,
            "detail": msg
        })),
    )
        .into_response()
}

// ── RF-USO-01: List / Create ─────────────────────────────────────────────

pub async fn list_trips(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(filters): Query<TripListFilters>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (trips, total) = state
        .trip_service
        .list_trips(filters)
        .await
        .map_err(|e| (StatusCode::from(&e), e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": trips, "total": total })))
}

pub async fn create_trip(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTripPayload>,
) -> Result<(StatusCode, Json<VehicleTripDto>), (StatusCode, String)> {
    state
        .trip_service
        .request_trip(payload, Some(user.id))
        .await
        .map(|t| (StatusCode::CREATED, Json(t)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_trip(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleTripDto>, (StatusCode, String)> {
    state
        .trip_service
        .get_trip(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ── RF-USO-01: Approve / Reject ──────────────────────────────────────────

pub async fn review_trip(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewTripPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.review_trip(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-VIG-04: Allocate (APROVADA → ALOCADA) ─────────────────────────────

pub async fn allocate_trip(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AllocateTripPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.allocate_trip(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-USO-02: Check-out — vehicle departure (ALOCADA → EM_CURSO) ────────

pub async fn checkout(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CheckoutPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.checkout(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-USO-03: Check-in — vehicle return (EM_CURSO → AGUARDANDO_PC) ──────

pub async fn checkin(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CheckinPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.checkin(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-USO: Finalize (AGUARDANDO_PC → CONCLUIDA) ─────────────────────────

pub async fn finalize_trip(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FinalizeTripPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.finalize_trip(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-ADM-06: Set conflict ───────────────────────────────────────────────

pub async fn set_conflict(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SetConflictPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.set_conflict(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}

// ── RF-USO-04: Cancel ────────────────────────────────────────────────────

pub async fn cancel_trip(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelTripPayload>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    match state.trip_service.cancel_trip(id, payload, user.id).await {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(ServiceError::OptimisticLockConflict(msg)) => occ_response(msg),
        Err(e) => (StatusCode::from(&e), e.to_string()).into_response(),
    }
}
