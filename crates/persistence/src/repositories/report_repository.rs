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
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<FuelConsumptionDto>, RepositoryError> {
        sqlx::query_as::<_, FuelConsumptionDto>(
            r#"
            SELECT
                v.id                                                        AS vehicle_id,
                v.license_plate,
                COUNT(f.id)                                                 AS total_fuelings,
                SUM(f.quantity_liters)                                      AS total_liters,
                SUM(f.total_cost)                                           AS total_cost,
                ROUND(AVG(f.consumo_litros_100km), 2)                       AS avg_consumption_l100km,
                CASE
                    WHEN SUM(NULLIF(f.odometer_km - f.km_anterior, 0)) > 0
                    THEN ROUND(SUM(f.total_cost)
                         / SUM(NULLIF(f.odometer_km - f.km_anterior, 0)), 4)
                    ELSE NULL
                END                                                         AS cost_per_km,
                SUM(NULLIF(f.odometer_km - f.km_anterior, 0))::BIGINT      AS total_km_driven
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
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn vehicle_dashboard(
        &self,
        vehicle_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
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
                COALESCE(trip_stats.total_trips, 0)                 AS total_trips,
                trip_stats.total_km_trips,
                COALESCE(mnt_stats.total_maintenance_cost, 0)       AS total_maintenance_cost,
                COALESCE(fuel_stats.total_fuel_cost, 0)             AS total_fuel_cost,
                COALESCE(os_stats.open_orders, 0)                   AS open_orders,
                COALESCE(inc_stats.total_incidents, 0)              AS total_incidents
            FROM vehicles v
            LEFT JOIN vehicle_makes  vm  ON vm.id  = v.make_id
            LEFT JOIN vehicle_models vmo ON vmo.id = v.model_id
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS total_trips, SUM(km_percorridos) AS total_km_trips
                FROM vehicle_trips
                WHERE vehicle_id = v.id AND status = 'CONCLUIDA'
                  AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
            ) trip_stats ON true
            LEFT JOIN LATERAL (
                SELECT SUM(custo_real) AS total_maintenance_cost
                FROM vehicle_maintenance_orders
                WHERE vehicle_id = v.id AND status = 'CONCLUIDA'
                  AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
            ) mnt_stats ON true
            LEFT JOIN LATERAL (
                SELECT SUM(total_cost) AS total_fuel_cost
                FROM fuelings
                WHERE vehicle_id = v.id
                  AND ($2::TIMESTAMPTZ IS NULL OR fueling_date >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR fueling_date <= $3)
            ) fuel_stats ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS open_orders
                FROM vehicle_maintenance_orders
                WHERE vehicle_id = v.id AND status IN ('ABERTA','EM_EXECUCAO')
            ) os_stats ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS total_incidents
                FROM vehicle_incidents
                WHERE vehicle_id = v.id
                  AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
            ) inc_stats ON true
            WHERE v.id = $1 AND v.is_deleted = false
            "#,
        )
        .bind(vehicle_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?
        .ok_or(RepositoryError::NotFound)
    }

    async fn fleet_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<FleetSummaryDto, RepositoryError> {
        sqlx::query_as::<_, FleetSummaryDto>(
            r#"
            SELECT
                -- contagens de status
                COUNT(*)                                                FILTER (WHERE NOT is_deleted)                          AS total_vehicles,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'ATIVO')       AS active_vehicles,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'MANUTENCAO')  AS vehicles_in_maintenance,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'INDISPONIVEL') AS unavailable_vehicles,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND allocation_status::TEXT = 'EM_USO')       AS vehicles_in_use,
                COUNT(*)                                                FILTER (WHERE NOT is_deleted AND allocation_status::TEXT = 'LIVRE')        AS available_vehicles,
                CASE WHEN COUNT(*) FILTER (WHERE NOT is_deleted) > 0
                     THEN ROUND(
                         COUNT(*) FILTER (WHERE NOT is_deleted AND operational_status::TEXT = 'ATIVO')::NUMERIC
                         / COUNT(*) FILTER (WHERE NOT is_deleted)::NUMERIC * 100, 1)
                     ELSE NULL
                END                                                     AS availability_percentage,
                (SELECT COUNT(*) FROM vehicle_trips
                 WHERE status = 'CONCLUIDA'
                   AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2))   AS monthly_trips,
                (SELECT SUM(km_percorridos) FROM vehicle_trips
                 WHERE status = 'CONCLUIDA'
                   AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2))   AS monthly_km,
                (SELECT SUM(total_cost) FROM fuelings
                 WHERE ($1::TIMESTAMPTZ IS NULL OR fueling_date >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR fueling_date <= $2)) AS monthly_fuel_cost,
                (SELECT SUM(custo_real) FROM vehicle_maintenance_orders
                 WHERE status = 'CONCLUIDA'
                   AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                   AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2))   AS monthly_maintenance_cost,
                (
                    COALESCE((SELECT SUM(total_cost) FROM fuelings
                     WHERE ($1::TIMESTAMPTZ IS NULL OR fueling_date >= $1)
                       AND ($2::TIMESTAMPTZ IS NULL OR fueling_date <= $2)), 0)
                  + COALESCE((SELECT SUM(custo_real) FROM vehicle_maintenance_orders
                     WHERE status = 'CONCLUIDA'
                       AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
                       AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2)), 0)
                )                                                       AS monthly_fleet_cost
            FROM vehicles
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
