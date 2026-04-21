use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::report::{FuelConsumptionDto, FleetSummaryDto, VehicleDashboardDto},
    ports::report::FleetReportRepositoryPort,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct FleetReportRepository {
    pool: PgPool,
}

impl FleetReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FleetReportRepositoryPort for FleetReportRepository {
    async fn fuel_consumption(
        &self,
        vehicle_id: Option<Uuid>,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<Vec<FuelConsumptionDto>, RepositoryError> {
        sqlx::query_as::<_, FuelConsumptionDto>(
            r#"
            SELECT
                v.id                                                        AS vehicle_id,
                v.license_plate,
                COUNT(f.id)                                                 AS total_abastecimentos,
                SUM(f.quantity_liters)                                      AS total_litros,
                SUM(f.total_cost)                                           AS total_custo,
                ROUND(AVG(f.consumo_litros_100km), 2)                       AS media_consumo_l100km,
                CASE
                    WHEN SUM(NULLIF(f.odometer_km - f.km_anterior, 0)) > 0
                    THEN ROUND(SUM(f.total_cost)
                         / SUM(NULLIF(f.odometer_km - f.km_anterior, 0)), 4)
                    ELSE NULL
                END                                                         AS custo_por_km,
                SUM(NULLIF(f.odometer_km - f.km_anterior, 0))::BIGINT      AS km_total_percorrido
            FROM vehicles v
            LEFT JOIN fuelings f ON f.vehicle_id = v.id
                AND ($2::TIMESTAMPTZ IS NULL OR f.fueling_date >= $2)
                AND ($3::TIMESTAMPTZ IS NULL OR f.fueling_date <= $3)
            WHERE v.is_deleted = false
              AND ($1::UUID IS NULL OR v.id = $1)
            GROUP BY v.id, v.license_plate
            ORDER BY v.license_plate
            "#,
        )
        .bind(vehicle_id)
        .bind(data_inicio)
        .bind(data_fim)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn vehicle_dashboard(
        &self,
        vehicle_id: Uuid,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<VehicleDashboardDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDashboardDto>(
            r#"
            SELECT
                v.id                                                AS vehicle_id,
                v.license_plate,
                vm.name                                             AS make_name,
                vmo.name                                            AS model_name,
                v.operational_status::TEXT,
                v.allocation_status::TEXT,
                COALESCE(trip_stats.total_viagens, 0)               AS total_viagens,
                trip_stats.km_total_viagens,
                COALESCE(mnt_stats.custo_total_manutencao, 0)       AS custo_total_manutencao,
                COALESCE(fuel_stats.custo_total_combustivel, 0)     AS custo_total_combustivel,
                COALESCE(os_stats.os_abertas, 0)                    AS os_abertas,
                COALESCE(inc_stats.total_sinistros, 0)              AS total_sinistros
            FROM vehicles v
            LEFT JOIN vehicle_makes  vm  ON vm.id  = v.make_id
            LEFT JOIN vehicle_models vmo ON vmo.id = v.model_id
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS total_viagens, SUM(km_percorridos) AS km_total_viagens
                FROM vehicle_trips
                WHERE vehicle_id = v.id AND status = 'CONCLUIDA'
                  AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
            ) trip_stats ON true
            LEFT JOIN LATERAL (
                SELECT SUM(custo_real) AS custo_total_manutencao
                FROM vehicle_maintenance_orders
                WHERE vehicle_id = v.id AND status = 'CONCLUIDA'
                  AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
            ) mnt_stats ON true
            LEFT JOIN LATERAL (
                SELECT SUM(total_cost) AS custo_total_combustivel
                FROM fuelings
                WHERE vehicle_id = v.id
                  AND ($2::TIMESTAMPTZ IS NULL OR fueling_date >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR fueling_date <= $3)
            ) fuel_stats ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS os_abertas
                FROM vehicle_maintenance_orders
                WHERE vehicle_id = v.id AND status IN ('ABERTA','EM_EXECUCAO')
            ) os_stats ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS total_sinistros
                FROM vehicle_incidents
                WHERE vehicle_id = v.id
                  AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
            ) inc_stats ON true
            WHERE v.id = $1 AND v.is_deleted = false
            "#,
        )
        .bind(vehicle_id)
        .bind(data_inicio)
        .bind(data_fim)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?
        .ok_or(RepositoryError::NotFound)
    }

    async fn fleet_summary(
        &self,
        data_inicio: Option<DateTime<Utc>>,
        data_fim: Option<DateTime<Utc>>,
    ) -> Result<FleetSummaryDto, RepositoryError> {
        sqlx::query_as::<_, FleetSummaryDto>(
            r#"
            SELECT
                -- contagens de status
                COUNT(*)                                                FILTER (WHERE NOT is_deleted)                          AS total_veiculos,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'ATIVO')       AS veiculos_ativos,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'MANUTENCAO')  AS veiculos_manutencao,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'INDISPONIVEL') AS veiculos_indisponiveis,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND allocation_status::TEXT = 'EM_USO')       AS veiculos_em_uso,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND allocation_status::TEXT = 'LIVRE')        AS veiculos_livres,
                -- percentual de disponibilidade (ativos e livres / total)
                CASE WHEN COUNT(*) FILTER (WHERE NOT is_deleted) > 0
                     THEN ROUND(
                         COUNT(*) FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'ATIVO')::NUMERIC
                         / COUNT(*) FILTER (WHERE NOT is_deleted)::NUMERIC * 100, 1)
                     ELSE NULL
                END                                                     AS percentual_disponibilidade,
                -- métricas de período
                (SELECT COUNT(*) FROM vehicle_trips
                 WHERE status = 'CONCLUIDA'
                   AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2))   AS total_viagens_mes,
                (SELECT SUM(km_percorridos) FROM vehicle_trips
                 WHERE status = 'CONCLUIDA'
                   AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2))   AS km_total_mes,
                (SELECT SUM(total_cost) FROM fuelings
                 WHERE ($1::TIMESTAMPTZ IS NULL OR fueling_date >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR fueling_date <= $2)) AS custo_total_combustivel_mes,
                (SELECT SUM(custo_real) FROM vehicle_maintenance_orders
                 WHERE status = 'CONCLUIDA'
                   AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2))   AS custo_total_manutencao_mes,
                (
                    COALESCE((SELECT SUM(total_cost) FROM fuelings
                     WHERE ($1::TIMESTAMPTZ IS NULL OR fueling_date >= $1)
                       AND ($2::TIMESTAMPTZ IS NULL OR fueling_date <= $2)), 0)
                  + COALESCE((SELECT SUM(custo_real) FROM vehicle_maintenance_orders
                     WHERE status = 'CONCLUIDA'
                       AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                       AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2)), 0)
                )                                                       AS custo_total_frota_mes
            FROM vehicles
            "#,
        )
        .bind(data_inicio)
        .bind(data_fim)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
