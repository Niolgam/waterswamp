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
    pub total_abastecimentos: i64,
    pub total_litros: Option<Decimal>,
    pub total_custo: Option<Decimal>,
    pub media_consumo_l100km: Option<Decimal>,
    pub custo_por_km: Option<Decimal>,
    pub km_total_percorrido: Option<i64>,
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
    pub total_viagens: i64,
    pub km_total_viagens: Option<i64>,
    pub custo_total_manutencao: Option<Decimal>,
    pub custo_total_combustivel: Option<Decimal>,
    pub os_abertas: i64,
    pub total_sinistros: i64,
}

// ── RF-REL-03: Resumo consolidado da frota ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct FleetSummaryDto {
    pub total_veiculos: i64,
    pub veiculos_ativos: i64,
    pub veiculos_manutencao: i64,
    pub veiculos_indisponiveis: i64,
    pub veiculos_em_uso: i64,
    pub veiculos_livres: i64,
    pub percentual_disponibilidade: Option<Decimal>,
    pub total_viagens_mes: i64,
    pub km_total_mes: Option<i64>,
    pub custo_total_combustivel_mes: Option<Decimal>,
    pub custo_total_manutencao_mes: Option<Decimal>,
    pub custo_total_frota_mes: Option<Decimal>,
}

// ── Filtros de data para relatórios ─────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReportDateFilter {
    pub data_inicio: Option<DateTime<Utc>>,
    pub data_fim: Option<DateTime<Utc>>,
}
