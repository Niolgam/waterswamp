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

// ============================
// Vehicle Disposal Repository
// ============================

pub struct VehicleDisposalRepository {
    pool: PgPool,
}

impl VehicleDisposalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleDisposalRepositoryPort for VehicleDisposalRepository {
    async fn create(
        &self,
        vehicle_id: Uuid,
        destino: DisposalDestination,
        justificativa: &str,
        numero_laudo: &str,
        documento_sei: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDisposalProcessDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDisposalProcessDto>(
            r#"
            INSERT INTO vehicle_disposal_processes
                (vehicle_id, destino, justificativa, numero_laudo, documento_sei, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            RETURNING *
            "#,
        )
        .bind(vehicle_id)
        .bind(destino)
        .bind(justificativa)
        .bind(numero_laudo)
        .bind(documento_sei)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleDisposalProcessDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDisposalProcessDto>(
            "SELECT * FROM vehicle_disposal_processes WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_vehicle(&self, vehicle_id: Uuid) -> Result<Option<VehicleDisposalProcessDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDisposalProcessDto>(
            "SELECT * FROM vehicle_disposal_processes WHERE vehicle_id = $1",
        )
        .bind(vehicle_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn advance_status(
        &self,
        id: Uuid,
        new_status: DisposalStatus,
        concluido_por: Option<Uuid>,
        cancelado_por: Option<Uuid>,
        motivo_cancelamento: Option<&str>,
        version: i32,
    ) -> Result<VehicleDisposalProcessDto, RepositoryError> {
        let is_concluido = new_status == DisposalStatus::Concluido;
        let is_cancelado = new_status == DisposalStatus::Cancelado;

        let result = sqlx::query_as::<_, VehicleDisposalProcessDto>(
            r#"
            UPDATE vehicle_disposal_processes
            SET status              = $2,
                concluido_por       = CASE WHEN $5 THEN $3 ELSE concluido_por END,
                concluido_em        = CASE WHEN $5 THEN NOW() ELSE concluido_em END,
                cancelado_por       = CASE WHEN $6 THEN $4 ELSE cancelado_por END,
                cancelado_em        = CASE WHEN $6 THEN NOW() ELSE cancelado_em END,
                motivo_cancelamento = COALESCE($7, motivo_cancelamento),
                version             = version + 1,
                updated_at          = NOW()
            WHERE id = $1 AND version = $8
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(new_status)
        .bind(concluido_por)
        .bind(cancelado_por)
        .bind(is_concluido)
        .bind(is_cancelado)
        .bind(motivo_cancelamento)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| RepositoryError::OptimisticLockConflict(format!("vehicle_disposal:{}", id)))
    }

    async fn list(&self, status: Option<DisposalStatus>) -> Result<Vec<VehicleDisposalProcessDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDisposalProcessDto>(
            r#"
            SELECT * FROM vehicle_disposal_processes
            WHERE ($1::disposal_status_enum IS NULL OR status = $1)
            ORDER BY created_at DESC
            "#,
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn add_step(
        &self,
        disposal_id: Uuid,
        descricao: &str,
        documento_sei: &str,
        data_execucao: NaiveDate,
        responsavel_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDisposalStepDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDisposalStepDto>(
            r#"
            INSERT INTO vehicle_disposal_steps
                (disposal_id, descricao, documento_sei, data_execucao, responsavel_id, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(disposal_id)
        .bind(descricao)
        .bind(documento_sei)
        .bind(data_execucao)
        .bind(responsavel_id)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_steps(&self, disposal_id: Uuid) -> Result<Vec<VehicleDisposalStepDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDisposalStepDto>(
            "SELECT * FROM vehicle_disposal_steps WHERE disposal_id = $1 ORDER BY data_execucao, created_at",
        )
        .bind(disposal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Fleet Fuel Catalog Repository
// ============================

pub struct FleetFuelCatalogRepository {
    pool: PgPool,
}

impl FleetFuelCatalogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FleetFuelCatalogRepositoryPort for FleetFuelCatalogRepository {
    async fn create(
        &self,
        nome: &str,
        catmat_item_id: Option<Uuid>,
        unidade: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FleetFuelCatalogDto, RepositoryError> {
        sqlx::query_as::<_, FleetFuelCatalogDto>(
            r#"
            INSERT INTO fleet_fuel_catalog (nome, catmat_item_id, unidade, notes, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING *
            "#,
        )
        .bind(nome)
        .bind(catmat_item_id)
        .bind(unidade)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FleetFuelCatalogDto>, RepositoryError> {
        sqlx::query_as::<_, FleetFuelCatalogDto>("SELECT * FROM fleet_fuel_catalog WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        nome: Option<&str>,
        catmat_item_id: Option<Option<Uuid>>,
        unidade: Option<&str>,
        ativo: Option<bool>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FleetFuelCatalogDto, RepositoryError> {
        sqlx::query_as::<_, FleetFuelCatalogDto>(
            r#"
            UPDATE fleet_fuel_catalog
            SET nome           = COALESCE($2, nome),
                catmat_item_id = CASE WHEN $3 IS NOT NULL THEN $3 ELSE catmat_item_id END,
                unidade        = COALESCE($4, unidade),
                ativo          = COALESCE($5, ativo),
                notes          = COALESCE($6, notes),
                updated_by     = $7,
                updated_at     = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(nome)
        .bind(catmat_item_id.flatten())
        .bind(unidade)
        .bind(ativo)
        .bind(notes)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(&self, only_active: bool) -> Result<Vec<FleetFuelCatalogDto>, RepositoryError> {
        sqlx::query_as::<_, FleetFuelCatalogDto>(
            "SELECT * FROM fleet_fuel_catalog WHERE ($1 = FALSE OR ativo = TRUE) ORDER BY nome",
        )
        .bind(only_active)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Fleet Maintenance Services Repository
// ============================

pub struct FleetMaintenanceServiceRepository {
    pool: PgPool,
}

impl FleetMaintenanceServiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FleetMaintenanceServiceRepositoryPort for FleetMaintenanceServiceRepository {
    async fn create(
        &self,
        nome: &str,
        catser_item_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FleetMaintenanceServiceDto, RepositoryError> {
        sqlx::query_as::<_, FleetMaintenanceServiceDto>(
            r#"
            INSERT INTO fleet_maintenance_services (nome, catser_item_id, notes, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $4)
            RETURNING *
            "#,
        )
        .bind(nome)
        .bind(catser_item_id)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FleetMaintenanceServiceDto>, RepositoryError> {
        sqlx::query_as::<_, FleetMaintenanceServiceDto>(
            "SELECT * FROM fleet_maintenance_services WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        nome: Option<&str>,
        catser_item_id: Option<Option<Uuid>>,
        ativo: Option<bool>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FleetMaintenanceServiceDto, RepositoryError> {
        sqlx::query_as::<_, FleetMaintenanceServiceDto>(
            r#"
            UPDATE fleet_maintenance_services
            SET nome           = COALESCE($2, nome),
                catser_item_id = CASE WHEN $3 IS NOT NULL THEN $3 ELSE catser_item_id END,
                ativo          = COALESCE($4, ativo),
                notes          = COALESCE($5, notes),
                updated_by     = $6,
                updated_at     = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(nome)
        .bind(catser_item_id.flatten())
        .bind(ativo)
        .bind(notes)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(&self, only_active: bool) -> Result<Vec<FleetMaintenanceServiceDto>, RepositoryError> {
        sqlx::query_as::<_, FleetMaintenanceServiceDto>(
            "SELECT * FROM fleet_maintenance_services WHERE ($1 = FALSE OR ativo = TRUE) ORDER BY nome",
        )
        .bind(only_active)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Fleet System Params Repository
// ============================

pub struct FleetSystemParamRepository {
    pool: PgPool,
}

impl FleetSystemParamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FleetSystemParamRepositoryPort for FleetSystemParamRepository {
    async fn find_by_key(&self, chave: &str) -> Result<Option<FleetSystemParamDto>, RepositoryError> {
        sqlx::query_as::<_, FleetSystemParamDto>(
            "SELECT * FROM fleet_system_params WHERE chave = $1",
        )
        .bind(chave)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn upsert(
        &self,
        chave: &str,
        valor: &str,
        descricao: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<FleetSystemParamDto, RepositoryError> {
        sqlx::query_as::<_, FleetSystemParamDto>(
            r#"
            INSERT INTO fleet_system_params (chave, valor, descricao, updated_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (chave) DO UPDATE
              SET valor      = EXCLUDED.valor,
                  descricao  = COALESCE(EXCLUDED.descricao, fleet_system_params.descricao),
                  updated_by = EXCLUDED.updated_by,
                  updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(chave)
        .bind(valor)
        .bind(descricao)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(&self) -> Result<Vec<FleetSystemParamDto>, RepositoryError> {
        sqlx::query_as::<_, FleetSystemParamDto>(
            "SELECT * FROM fleet_system_params ORDER BY chave",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Fleet Checklist Template Repository
// ============================

pub struct FleetChecklistTemplateRepository {
    pool: PgPool,
}

impl FleetChecklistTemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FleetChecklistTemplateRepositoryPort for FleetChecklistTemplateRepository {
    async fn create(
        &self,
        nome: &str,
        descricao: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<FleetChecklistTemplateDto, RepositoryError> {
        sqlx::query_as::<_, FleetChecklistTemplateDto>(
            r#"
            INSERT INTO fleet_checklist_templates (nome, descricao, created_by, updated_by)
            VALUES ($1, $2, $3, $3)
            RETURNING *
            "#,
        )
        .bind(nome)
        .bind(descricao)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FleetChecklistTemplateDto>, RepositoryError> {
        sqlx::query_as::<_, FleetChecklistTemplateDto>(
            "SELECT * FROM fleet_checklist_templates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(&self, only_active: bool) -> Result<Vec<FleetChecklistTemplateDto>, RepositoryError> {
        sqlx::query_as::<_, FleetChecklistTemplateDto>(
            "SELECT * FROM fleet_checklist_templates WHERE ($1 = FALSE OR ativo = TRUE) ORDER BY nome",
        )
        .bind(only_active)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn add_item(
        &self,
        template_id: Uuid,
        descricao: &str,
        obrigatorio: bool,
        ordem: i32,
    ) -> Result<FleetChecklistItemDto, RepositoryError> {
        sqlx::query_as::<_, FleetChecklistItemDto>(
            r#"
            INSERT INTO fleet_checklist_items (template_id, descricao, obrigatorio, ordem)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(template_id)
        .bind(descricao)
        .bind(obrigatorio)
        .bind(ordem)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_items(&self, template_id: Uuid) -> Result<Vec<FleetChecklistItemDto>, RepositoryError> {
        sqlx::query_as::<_, FleetChecklistItemDto>(
            "SELECT * FROM fleet_checklist_items WHERE template_id = $1 ORDER BY ordem, created_at",
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
