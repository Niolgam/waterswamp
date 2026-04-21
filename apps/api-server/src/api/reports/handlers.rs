use crate::infra::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct FuelConsumptionQuery {
    pub vehicle_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

// RF-REL-01: GET /fleet/reports/fuel-consumption
pub async fn get_fuel_consumption(
    State(state): State<AppState>,
    Query(q): Query<FuelConsumptionQuery>,
) -> impl IntoResponse {
    match state
        .fleet_report_service
        .fuel_consumption(q.vehicle_id, q.start_date, q.end_date)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// RF-REL-02: GET /fleet/reports/vehicles/{id}/dashboard
pub async fn get_vehicle_dashboard(
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Query(q): Query<DateRangeQuery>,
) -> impl IntoResponse {
    match state
        .fleet_report_service
        .vehicle_dashboard(vehicle_id, q.start_date, q.end_date)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("não encontrado") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, e.to_string()).into_response()
        }
    }
}

// RF-REL-03: GET /fleet/reports/fleet-summary
pub async fn get_fleet_summary(
    State(state): State<AppState>,
    Query(q): Query<DateRangeQuery>,
) -> impl IntoResponse {
    match state
        .fleet_report_service
        .fleet_summary(q.start_date, q.end_date)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
