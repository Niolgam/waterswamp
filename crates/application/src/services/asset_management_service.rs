use crate::errors::ServiceError;
use chrono::{Datelike, Utc};
use domain::{
    models::asset_management::*,
    models::vehicle::{AllocationStatus, OperationalStatus},
    ports::asset_management::*,
    ports::vehicle::{VehicleModelRepositoryPort, VehicleRepositoryPort, VehicleStatusHistoryRepositoryPort},
    models::vehicle::VehicleStatus,
};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::sync::Arc;
use uuid::Uuid;

pub struct AssetManagementService {
    transfer_repo: Arc<dyn VehicleDepartmentTransferRepositoryPort>,
    depreciation_repo: Arc<dyn DepreciationConfigRepositoryPort>,
    incident_repo: Arc<dyn VehicleIncidentRepositoryPort>,
    disposal_repo: Arc<dyn VehicleDisposalRepositoryPort>,
    fuel_catalog_repo: Arc<dyn FleetFuelCatalogRepositoryPort>,
    maintenance_service_repo: Arc<dyn FleetMaintenanceServiceRepositoryPort>,
    system_param_repo: Arc<dyn FleetSystemParamRepositoryPort>,
    checklist_repo: Arc<dyn FleetChecklistTemplateRepositoryPort>,
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    vehicle_model_repo: Arc<dyn VehicleModelRepositoryPort>,
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl AssetManagementService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transfer_repo: Arc<dyn VehicleDepartmentTransferRepositoryPort>,
        depreciation_repo: Arc<dyn DepreciationConfigRepositoryPort>,
        incident_repo: Arc<dyn VehicleIncidentRepositoryPort>,
        disposal_repo: Arc<dyn VehicleDisposalRepositoryPort>,
        fuel_catalog_repo: Arc<dyn FleetFuelCatalogRepositoryPort>,
        maintenance_service_repo: Arc<dyn FleetMaintenanceServiceRepositoryPort>,
        system_param_repo: Arc<dyn FleetSystemParamRepositoryPort>,
        checklist_repo: Arc<dyn FleetChecklistTemplateRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        vehicle_model_repo: Arc<dyn VehicleModelRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self {
            transfer_repo,
            depreciation_repo,
            incident_repo,
            disposal_repo,
            fuel_catalog_repo,
            maintenance_service_repo,
            system_param_repo,
            checklist_repo,
            vehicle_repo,
            vehicle_model_repo,
            status_history_repo,
        }
    }

    // ── RF-AST-06: Transferência Departamental ──────────────────────────────

    pub async fn transfer_department(
        &self,
        vehicle_id: Uuid,
        payload: CreateVehicleDepartmentTransferPayload,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDepartmentTransferDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.department_id == Some(payload.target_dept_id) {
            return Err(ServiceError::BadRequest(
                "Destino igual ao departamento atual".to_string(),
            ));
        }

        let transfer = self.transfer_repo
            .create(
                vehicle_id,
                vehicle.department_id,
                payload.target_dept_id,
                payload.effective_date,
                &payload.reason,
                payload.documento_sei.as_deref(),
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        // Atualiza department_id no veículo
        let _ = self.vehicle_repo
            .update(
                vehicle_id,
                None, None, None, None, None, None, None,
                None, None, None, None,
                None, None,
                None, None, None, None,
                None, None, None,
                None, Some(payload.target_dept_id),
                None, None,
                created_by,
                None,
            )
            .await
            .map_err(ServiceError::from)?;

        Ok(transfer)
    }

    pub async fn list_transfers(
        &self,
        vehicle_id: Uuid,
    ) -> Result<Vec<VehicleDepartmentTransferDto>, ServiceError> {
        self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.transfer_repo
            .list_by_vehicle(vehicle_id)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-AST-11: Depreciação ──────────────────────────────────────────────

    pub async fn upsert_depreciation_config(
        &self,
        payload: UpsertDepreciationConfigPayload,
        updated_by: Option<Uuid>,
    ) -> Result<DepreciationConfigDto, ServiceError> {
        if payload.useful_life_years <= Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "useful_life_years deve ser > 0".to_string(),
            ));
        }
        if payload.residual_value_min < Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "residual_value_min não pode ser negativo".to_string(),
            ));
        }

        self.depreciation_repo
            .upsert(
                payload.vehicle_category_id,
                payload.useful_life_years,
                payload.residual_value_min,
                payload.is_active.unwrap_or(true),
                payload.notes.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_depreciation_configs(
        &self,
    ) -> Result<Vec<DepreciationConfigDto>, ServiceError> {
        self.depreciation_repo.list().await.map_err(ServiceError::from)
    }

    /// Calcula a depreciação linear de um veículo até a data atual (DRS RN20).
    pub async fn calculate_depreciation(
        &self,
        vehicle_id: Uuid,
    ) -> Result<DepreciationCalculationDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        let purchase_value = vehicle.purchase_value.ok_or_else(|| {
            ServiceError::BadRequest("Veículo sem valor de aquisição registrado".to_string())
        })?;
        let acquisition_date = vehicle.acquisition_date.ok_or_else(|| {
            ServiceError::BadRequest("Veículo sem data de aquisição registrada".to_string())
        })?;

        // Busca o modelo do veículo para obter category_id
        let model = self.vehicle_model_repo
            .find_by_id(vehicle.model_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Modelo do veículo não encontrado".to_string()))?;

        let category_id = model.category_id.ok_or_else(|| {
            ServiceError::BadRequest("Modelo do veículo sem categoria definida".to_string())
        })?;

        let config = self.depreciation_repo
            .find_by_category(category_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| {
                ServiceError::NotFound(
                    "Configuração de depreciação não encontrada para esta categoria".to_string(),
                )
            })?;

        let today = Utc::now().date_naive();
        let months_elapsed = ((today.year() - acquisition_date.year()) * 12
            + (today.month() as i32 - acquisition_date.month() as i32))
            .max(0) as i64;

        let useful_life_months = (config.useful_life_years
            .to_f64()
            .unwrap_or(0.0)
            * 12.0) as i64;

        let depreciable = (purchase_value - config.residual_value_min).max(Decimal::ZERO);
        let depreciation_monthly = if useful_life_months > 0 {
            depreciable / Decimal::from(useful_life_months)
        } else {
            Decimal::ZERO
        };

        let accumulated = (depreciation_monthly * Decimal::from(months_elapsed))
            .min(depreciable);

        let estimated_residual = (purchase_value - accumulated).max(config.residual_value_min);
        let is_fully_depreciated = months_elapsed >= useful_life_months;

        Ok(DepreciationCalculationDto {
            vehicle_id,
            purchase_value,
            acquisition_date,
            useful_life_years: config.useful_life_years,
            residual_value_min: config.residual_value_min,
            months_elapsed,
            depreciation_monthly,
            accumulated_depreciation: accumulated,
            estimated_residual_value: estimated_residual,
            is_fully_depreciated,
        })
    }

    // ── RF-AST-12: Sinistros ────────────────────────────────────────────────

    /// Abre sinistro e transiciona o veículo para INDISPONIVEL/LIVRE (RF-AST-12).
    pub async fn open_incident(
        &self,
        vehicle_id: Uuid,
        payload: CreateVehicleIncidentPayload,
        created_by: Option<Uuid>,
    ) -> Result<VehicleIncidentDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.allocation_status != AllocationStatus::Livre {
            return Err(ServiceError::Conflict(
                "Veículo em uso — cancele a viagem antes de registrar o sinistro (RN-FSM-03)"
                    .to_string(),
            ));
        }

        // Transição operacional: → INDISPONIVEL (com OCC)
        let _ = self.vehicle_repo
            .change_operational_status(
                vehicle_id,
                OperationalStatus::Indisponivel,
                payload.vehicle_version,
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        let _ = self.status_history_repo
            .create(
                vehicle_id,
                Some(vehicle.status.clone()),
                VehicleStatus::Inactive,
                Some("Sinistro registrado (RF-AST-12)"),
                created_by,
            )
            .await;

        self.incident_repo
            .create(
                vehicle_id,
                payload.incident_type,
                payload.occurred_at,
                payload.location.as_deref(),
                &payload.police_report_number,
                payload.insurance_number.as_deref(),
                payload.description.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_incident(
        &self,
        id: Uuid,
        payload: UpdateVehicleIncidentPayload,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleIncidentDto, ServiceError> {
        let _ = self.incident_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Sinistro não encontrado".to_string()))?;

        self.incident_repo
            .update_status(
                id,
                payload.status,
                payload.resolution_notes.as_deref(),
                payload.insurance_number.as_deref(),
                updated_by,
                payload.version,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_incidents(
        &self,
        vehicle_id: Uuid,
        status: Option<VehicleIncidentStatus>,
    ) -> Result<Vec<VehicleIncidentDto>, ServiceError> {
        self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.incident_repo
            .list_by_vehicle(vehicle_id, status)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-AST-09/10: Processo de Baixa ────────────────────────────────────

    /// Inicia processo de baixa: veículo → INDISPONIVEL, depreciação suspensa (RF-AST-09).
    pub async fn open_disposal(
        &self,
        vehicle_id: Uuid,
        payload: CreateDisposalProcessPayload,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDisposalProcessDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.allocation_status != AllocationStatus::Livre {
            return Err(ServiceError::Conflict(
                "Veículo em uso — encerre a viagem antes de iniciar a baixa".to_string(),
            ));
        }

        // Verifica se já existe processo de baixa ativo
        if let Some(existing) = self.disposal_repo.find_by_vehicle(vehicle_id).await.map_err(ServiceError::from)? {
            if !matches!(existing.status, DisposalStatus::Completed | DisposalStatus::Cancelled) {
                return Err(ServiceError::Conflict(
                    "Já existe um processo de baixa ativo para este veículo".to_string(),
                ));
            }
        }

        // Transiciona para INDISPONIVEL (OCC via vehicle_version do payload)
        let _ = self.vehicle_repo
            .change_operational_status(
                vehicle_id,
                OperationalStatus::Indisponivel,
                payload.vehicle_version,
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        let _ = self.status_history_repo
            .create(
                vehicle_id,
                Some(vehicle.status),
                VehicleStatus::Inactive,
                Some("Processo de baixa iniciado (RF-AST-09)"),
                created_by,
            )
            .await;

        self.disposal_repo
            .create(
                vehicle_id,
                payload.destination,
                &payload.justification,
                &payload.report_number,
                payload.documento_sei.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn advance_disposal(
        &self,
        disposal_id: Uuid,
        payload: AdvanceDisposalPayload,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleDisposalProcessDto, ServiceError> {
        let disposal = self.disposal_repo
            .find_by_id(disposal_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Processo de baixa não encontrado".to_string()))?;

        // FSM: INICIADO → EM_ANDAMENTO → CONCLUIDO | CANCELADO
        let valid = match (&disposal.status, &payload.new_status) {
            (DisposalStatus::Initiated, DisposalStatus::InProgress) => true,
            (DisposalStatus::InProgress, DisposalStatus::Completed) => true,
            (DisposalStatus::Initiated | DisposalStatus::InProgress, DisposalStatus::Cancelled) => true,
            _ => false,
        };
        if !valid {
            return Err(ServiceError::BadRequest(format!(
                "Transição inválida do processo de baixa"
            )));
        }

        if payload.new_status == DisposalStatus::Cancelled && payload.cancellation_reason.is_none() {
            return Err(ServiceError::BadRequest(
                "Motivo de cancelamento obrigatório".to_string(),
            ));
        }

        let completed_by = if payload.new_status == DisposalStatus::Completed { updated_by } else { None };
        let cancelled_by = if payload.new_status == DisposalStatus::Cancelled { updated_by } else { None };

        self.disposal_repo
            .advance_status(
                disposal_id,
                payload.new_status,
                completed_by,
                cancelled_by,
                payload.cancellation_reason.as_deref(),
                payload.version,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn add_disposal_step(
        &self,
        disposal_id: Uuid,
        payload: CreateDisposalStepPayload,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDisposalStepDto, ServiceError> {
        let disposal = self.disposal_repo
            .find_by_id(disposal_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Processo de baixa não encontrado".to_string()))?;

        if matches!(disposal.status, DisposalStatus::Completed | DisposalStatus::Cancelled) {
            return Err(ServiceError::BadRequest(
                "Não é possível adicionar etapas a um processo finalizado".to_string(),
            ));
        }

        self.disposal_repo
            .add_step(
                disposal_id,
                &payload.description,
                &payload.documento_sei,
                payload.execution_date,
                payload.responsavel_id,
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_disposal_steps(
        &self,
        disposal_id: Uuid,
    ) -> Result<Vec<VehicleDisposalStepDto>, ServiceError> {
        self.disposal_repo
            .find_by_id(disposal_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Processo de baixa não encontrado".to_string()))?;

        self.disposal_repo
            .list_steps(disposal_id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_disposals(
        &self,
        status: Option<DisposalStatus>,
    ) -> Result<Vec<VehicleDisposalProcessDto>, ServiceError> {
        self.disposal_repo.list(status).await.map_err(ServiceError::from)
    }

    pub async fn get_disposal_by_vehicle(
        &self,
        vehicle_id: Uuid,
    ) -> Result<Option<VehicleDisposalProcessDto>, ServiceError> {
        self.disposal_repo.find_by_vehicle(vehicle_id).await.map_err(ServiceError::from)
    }

    // ── RF-ADM-07: Catálogo de Combustíveis ────────────────────────────────

    pub async fn create_fuel(
        &self,
        payload: CreateFleetFuelCatalogPayload,
        created_by: Option<Uuid>,
    ) -> Result<FleetFuelCatalogDto, ServiceError> {
        self.fuel_catalog_repo
            .create(
                &payload.name,
                payload.catmat_item_id,
                payload.unit.as_deref().unwrap_or("LITRO"),
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_fuel(
        &self,
        id: Uuid,
        payload: UpdateFleetFuelCatalogPayload,
        updated_by: Option<Uuid>,
    ) -> Result<FleetFuelCatalogDto, ServiceError> {
        self.fuel_catalog_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Combustível não encontrado".to_string()))?;

        self.fuel_catalog_repo
            .update(
                id,
                payload.name.as_deref(),
                payload.catmat_item_id.map(Some),
                payload.unit.as_deref(),
                payload.active,
                payload.notes.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_fuels(&self, only_active: bool) -> Result<Vec<FleetFuelCatalogDto>, ServiceError> {
        self.fuel_catalog_repo.list(only_active).await.map_err(ServiceError::from)
    }

    // ── RF-ADM-08: Catálogo de Serviços de Manutenção ──────────────────────

    pub async fn create_maintenance_service(
        &self,
        payload: CreateFleetMaintenanceServicePayload,
        created_by: Option<Uuid>,
    ) -> Result<FleetMaintenanceServiceDto, ServiceError> {
        self.maintenance_service_repo
            .create(
                &payload.name,
                payload.catser_item_id,
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_maintenance_service(
        &self,
        id: Uuid,
        payload: UpdateFleetMaintenanceServicePayload,
        updated_by: Option<Uuid>,
    ) -> Result<FleetMaintenanceServiceDto, ServiceError> {
        self.maintenance_service_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Serviço de manutenção não encontrado".to_string()))?;

        self.maintenance_service_repo
            .update(
                id,
                payload.name.as_deref(),
                payload.catser_item_id.map(Some),
                payload.active,
                payload.notes.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_maintenance_services(
        &self,
        only_active: bool,
    ) -> Result<Vec<FleetMaintenanceServiceDto>, ServiceError> {
        self.maintenance_service_repo.list(only_active).await.map_err(ServiceError::from)
    }

    // ── RF-ADM-01: Parâmetros do Sistema ───────────────────────────────────

    pub async fn upsert_system_param(
        &self,
        payload: UpsertFleetSystemParamPayload,
        updated_by: Option<Uuid>,
    ) -> Result<FleetSystemParamDto, ServiceError> {
        self.system_param_repo
            .upsert(
                &payload.key,
                &payload.value,
                payload.description.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_system_params(&self) -> Result<Vec<FleetSystemParamDto>, ServiceError> {
        self.system_param_repo.list().await.map_err(ServiceError::from)
    }

    // ── RF-ADM-02: Templates de Checklist ──────────────────────────────────

    pub async fn create_checklist_template(
        &self,
        payload: CreateFleetChecklistTemplatePayload,
        created_by: Option<Uuid>,
    ) -> Result<FleetChecklistTemplateDto, ServiceError> {
        self.checklist_repo
            .create(&payload.name, payload.description.as_deref(), created_by)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_checklist_templates(
        &self,
        only_active: bool,
    ) -> Result<Vec<FleetChecklistTemplateDto>, ServiceError> {
        self.checklist_repo.list(only_active).await.map_err(ServiceError::from)
    }

    pub async fn add_checklist_item(
        &self,
        template_id: Uuid,
        payload: CreateFleetChecklistItemPayload,
    ) -> Result<FleetChecklistItemDto, ServiceError> {
        self.checklist_repo
            .find_by_id(template_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Template de checklist não encontrado".to_string()))?;

        self.checklist_repo
            .add_item(
                template_id,
                &payload.description,
                payload.required.unwrap_or(true),
                payload.order_index.unwrap_or(0),
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_checklist_items(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<FleetChecklistItemDto>, ServiceError> {
        self.checklist_repo
            .find_by_id(template_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Template de checklist não encontrado".to_string()))?;

        self.checklist_repo.list_items(template_id).await.map_err(ServiceError::from)
    }
}
