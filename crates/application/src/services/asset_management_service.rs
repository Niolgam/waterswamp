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
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    vehicle_model_repo: Arc<dyn VehicleModelRepositoryPort>,
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl AssetManagementService {
    pub fn new(
        transfer_repo: Arc<dyn VehicleDepartmentTransferRepositoryPort>,
        depreciation_repo: Arc<dyn DepreciationConfigRepositoryPort>,
        incident_repo: Arc<dyn VehicleIncidentRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        vehicle_model_repo: Arc<dyn VehicleModelRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self { transfer_repo, depreciation_repo, incident_repo, vehicle_repo, vehicle_model_repo, status_history_repo }
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

        if vehicle.department_id == Some(payload.dept_destino_id) {
            return Err(ServiceError::BadRequest(
                "Destino igual ao departamento atual".to_string(),
            ));
        }

        let transfer = self.transfer_repo
            .create(
                vehicle_id,
                vehicle.department_id,
                payload.dept_destino_id,
                payload.data_efetiva,
                &payload.motivo,
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
                None, Some(payload.dept_destino_id),
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
                payload.tipo,
                payload.data_ocorrencia,
                payload.local_ocorrencia.as_deref(),
                &payload.numero_bo,
                payload.numero_seguradora.as_deref(),
                payload.descricao.as_deref(),
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
                payload.notas_resolucao.as_deref(),
                payload.numero_seguradora.as_deref(),
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
}
