use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use domain::{
    errors::RepositoryError,
    models::asset_management::*,
    ports::asset_management::*,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================
// Vehicle Department Transfer Repository
// ============================

pub struct VehicleDepartmentTransferRepository {
    pool: PgPool,
}

impl VehicleDepartmentTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleDepartmentTransferRepositoryPort for VehicleDepartmentTransferRepository {
    async fn create(
        &self,
        vehicle_id: Uuid,
        dept_origem_id: Option<Uuid>,
        dept_destino_id: Uuid,
        data_efetiva: NaiveDate,
        motivo: &str,
        documento_sei: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDepartmentTransferDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDepartmentTransferDto>(
            r#"
            INSERT INTO vehicle_department_transfers
                (vehicle_id, dept_origem_id, dept_destino_id, data_efetiva,
                 motivo, documento_sei, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(vehicle_id)
        .bind(dept_origem_id)
        .bind(dept_destino_id)
        .bind(data_efetiva)
        .bind(motivo)
        .bind(documento_sei)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
    ) -> Result<Vec<VehicleDepartmentTransferDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDepartmentTransferDto>(
            "SELECT * FROM vehicle_department_transfers WHERE vehicle_id = $1 ORDER BY data_efetiva DESC",
        )
        .bind(vehicle_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Depreciation Config Repository
// ============================

pub struct DepreciationConfigRepository {
    pool: PgPool,
}

impl DepreciationConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DepreciationConfigRepositoryPort for DepreciationConfigRepository {
    async fn find_by_category(
        &self,
        vehicle_category_id: Uuid,
    ) -> Result<Option<DepreciationConfigDto>, RepositoryError> {
        sqlx::query_as::<_, DepreciationConfigDto>(
            "SELECT * FROM depreciation_configs WHERE vehicle_category_id = $1",
        )
        .bind(vehicle_category_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DepreciationConfigDto>, RepositoryError> {
        sqlx::query_as::<_, DepreciationConfigDto>(
            "SELECT * FROM depreciation_configs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn upsert(
        &self,
        vehicle_category_id: Uuid,
        useful_life_years: Decimal,
        residual_value_min: Decimal,
        is_active: bool,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<DepreciationConfigDto, RepositoryError> {
        sqlx::query_as::<_, DepreciationConfigDto>(
            r#"
            INSERT INTO depreciation_configs
                (vehicle_category_id, useful_life_years, residual_value_min, is_active, notes, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            ON CONFLICT (vehicle_category_id) DO UPDATE
              SET useful_life_years = EXCLUDED.useful_life_years,
                  residual_value_min = EXCLUDED.residual_value_min,
                  is_active  = EXCLUDED.is_active,
                  notes      = EXCLUDED.notes,
                  updated_by = EXCLUDED.updated_by,
                  updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(vehicle_category_id)
        .bind(useful_life_years)
        .bind(residual_value_min)
        .bind(is_active)
        .bind(notes)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(&self) -> Result<Vec<DepreciationConfigDto>, RepositoryError> {
        sqlx::query_as::<_, DepreciationConfigDto>(
            "SELECT * FROM depreciation_configs ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Vehicle Incident Repository
// ============================

pub struct VehicleIncidentRepository {
    pool: PgPool,
}

impl VehicleIncidentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleIncidentRepositoryPort for VehicleIncidentRepository {
    async fn create(
        &self,
        vehicle_id: Uuid,
        tipo: VehicleIncidentType,
        data_ocorrencia: DateTime<Utc>,
        local_ocorrencia: Option<&str>,
        numero_bo: &str,
        numero_seguradora: Option<&str>,
        descricao: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleIncidentDto, RepositoryError> {
        sqlx::query_as::<_, VehicleIncidentDto>(
            r#"
            INSERT INTO vehicle_incidents
                (vehicle_id, tipo, data_ocorrencia, local_ocorrencia,
                 numero_bo, numero_seguradora, descricao, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(vehicle_id)
        .bind(tipo)
        .bind(data_ocorrencia)
        .bind(local_ocorrencia)
        .bind(numero_bo)
        .bind(numero_seguradora)
        .bind(descricao)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleIncidentDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleIncidentDto>(
            "SELECT * FROM vehicle_incidents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: VehicleIncidentStatus,
        notas_resolucao: Option<&str>,
        numero_seguradora: Option<&str>,
        encerrado_por: Option<Uuid>,
        version: i32,
    ) -> Result<VehicleIncidentDto, RepositoryError> {
        let is_closing = matches!(
            status,
            VehicleIncidentStatus::EncerrradoRecuperado | VehicleIncidentStatus::EncerradoPerdaTotal
        );

        let result = sqlx::query_as::<_, VehicleIncidentDto>(
            r#"
            UPDATE vehicle_incidents
            SET status           = $2,
                notas_resolucao  = COALESCE($3, notas_resolucao),
                numero_seguradora = COALESCE($4, numero_seguradora),
                encerrado_por    = CASE WHEN $6 THEN $5 ELSE encerrado_por END,
                encerrado_em     = CASE WHEN $6 THEN NOW() ELSE encerrado_em END,
                updated_at       = NOW(),
                version          = version + 1
            WHERE id = $1 AND version = $7
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(notas_resolucao)
        .bind(numero_seguradora)
        .bind(encerrado_por)
        .bind(is_closing)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| {
            RepositoryError::OptimisticLockConflict(format!("vehicle_incident:{}", id))
        })
    }

    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        status: Option<VehicleIncidentStatus>,
    ) -> Result<Vec<VehicleIncidentDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleIncidentDto>(
            r#"
            SELECT * FROM vehicle_incidents
            WHERE vehicle_id = $1
              AND ($2::vehicle_incident_status_enum IS NULL OR status = $2)
            ORDER BY data_ocorrencia DESC
            "#,
        )
        .bind(vehicle_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
