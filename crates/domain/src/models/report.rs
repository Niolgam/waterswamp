use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ── RF-REL-01: Consumo de combustível por veículo ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FuelConsumptionDto {
    pub vehicle_id: Uuid,
    pub license_plate: Option<String>,
    #[sqlx(rename = "total_fuelings")]
    pub total_fuelings: i64,
    #[sqlx(rename = "total_liters")]
    pub total_liters: Option<Decimal>,
    #[sqlx(rename = "total_cost")]
    pub total_cost: Option<Decimal>,
    #[sqlx(rename = "avg_consumption_l100km")]
    pub avg_consumption_l100km: Option<Decimal>,
    #[sqlx(rename = "cost_per_km")]
    pub cost_per_km: Option<Decimal>,
    #[sqlx(rename = "total_km_driven")]
    pub total_km_driven: Option<i64>,
}

// ── RF-REL-02: Dashboard por veículo ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VehicleDashboardDto {
    pub vehicle_id: Uuid,
    pub license_plate: Option<String>,
    pub make_name: Option<String>,
    pub model_name: Option<String>,
    pub operational_status: String,
    pub allocation_status: String,
    #[sqlx(rename = "total_trips")]
    pub total_trips: i64,
    #[sqlx(rename = "total_km_trips")]
    pub total_km_trips: Option<i64>,
    #[sqlx(rename = "total_maintenance_cost")]
    pub total_maintenance_cost: Option<Decimal>,
    #[sqlx(rename = "total_fuel_cost")]
    pub total_fuel_cost: Option<Decimal>,
    #[sqlx(rename = "open_orders")]
    pub open_orders: i64,
    #[sqlx(rename = "total_incidents")]
    pub total_incidents: i64,
}

// ── RF-REL-03: Resumo consolidado da frota ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetSummaryDto {
    #[sqlx(rename = "total_vehicles")]
    pub total_vehicles: i64,
    #[sqlx(rename = "active_vehicles")]
    pub active_vehicles: i64,
    #[sqlx(rename = "vehicles_in_maintenance")]
    pub vehicles_in_maintenance: i64,
    #[sqlx(rename = "unavailable_vehicles")]
    pub unavailable_vehicles: i64,
    #[sqlx(rename = "vehicles_in_use")]
    pub vehicles_in_use: i64,
    #[sqlx(rename = "available_vehicles")]
    pub available_vehicles: i64,
    #[sqlx(rename = "availability_percentage")]
    pub availability_percentage: Option<Decimal>,
    #[sqlx(rename = "monthly_trips")]
    pub monthly_trips: i64,
    #[sqlx(rename = "monthly_km")]
    pub monthly_km: Option<i64>,
    #[sqlx(rename = "monthly_fuel_cost")]
    pub monthly_fuel_cost: Option<Decimal>,
    #[sqlx(rename = "monthly_maintenance_cost")]
    pub monthly_maintenance_cost: Option<Decimal>,
    #[sqlx(rename = "monthly_fleet_cost")]
    pub monthly_fleet_cost: Option<Decimal>,
}

// ── Filtros de data para relatórios ─────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReportDateFilter {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}
