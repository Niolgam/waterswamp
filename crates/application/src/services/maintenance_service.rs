use crate::errors::ServiceError;
use chrono::Local;
use domain::{
    models::maintenance::*,
    models::vehicle::{AllocationStatus, OperationalStatus, VehicleStatus},
    ports::maintenance::MaintenanceOrderRepositoryPort,
    ports::vehicle::{VehicleRepositoryPort, VehicleStatusHistoryRepositoryPort},
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct MaintenanceService {
    order_repo: Arc<dyn MaintenanceOrderRepositoryPort>,
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl MaintenanceService {
    pub fn new(
        order_repo: Arc<dyn MaintenanceOrderRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self { order_repo, vehicle_repo, status_history_repo }
    }

    // ── RF-MNT-01: Abrir OS ────────────────────────────────────────────────

    /// Abre uma OS e transiciona o veículo para MANUTENCAO (RF-MNT-01).
    /// RN-FSM-01: MANUTENCAO só se allocation_status = LIVRE.
    pub async fn open_order(
        &self,
        vehicle_id: Uuid,
        payload: CreateMaintenanceOrderPayload,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, ServiceError> {
        let vehicle = self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if vehicle.allocation_status != AllocationStatus::Livre {
            return Err(ServiceError::Conflict(
                "Veículo em uso — encerre a viagem antes de abrir OS (RN-FSM-01)".to_string(),
            ));
        }

        // Transiciona operational_status → MANUTENCAO (OCC)
        let _ = self.vehicle_repo
            .change_operational_status(
                vehicle_id,
                OperationalStatus::Manutencao,
                payload.vehicle_version,
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        let _ = self.status_history_repo
            .create(
                vehicle_id,
                Some(vehicle.status),
                VehicleStatus::InMaintenance,
                Some(&format!("OS aberta (RF-MNT-01): {}", payload.titulo)),
                created_by,
            )
            .await;

        let data_abertura = payload.data_abertura
            .unwrap_or_else(|| Local::now().date_naive());

        self.order_repo
            .create(
                vehicle_id,
                payload.tipo,
                &payload.titulo,
                payload.descricao.as_deref(),
                payload.fornecedor_id,
                data_abertura,
                payload.data_prevista_conclusao,
                payload.km_abertura,
                payload.custo_previsto,
                payload.numero_os_externo.as_deref(),
                payload.documento_sei.as_deref(),
                payload.incident_id,
                payload.notas.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-MNT-02: Avançar OS (EM_EXECUCAO / CONCLUIDA / CANCELADA) ────────

    pub async fn advance_order(
        &self,
        order_id: Uuid,
        payload: AdvanceMaintenanceOrderPayload,
        updated_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, ServiceError> {
        let order = self.order_repo
            .find_by_id(order_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))?;

        // Valida transições de FSM
        let valid = match (&order.status, &payload.new_status) {
            (MaintenanceOrderStatus::Aberta, MaintenanceOrderStatus::EmExecucao) => true,
            (MaintenanceOrderStatus::Aberta | MaintenanceOrderStatus::EmExecucao,
             MaintenanceOrderStatus::Concluida) => true,
            (MaintenanceOrderStatus::Aberta | MaintenanceOrderStatus::EmExecucao,
             MaintenanceOrderStatus::Cancelada) => true,
            _ => false,
        };
        if !valid {
            return Err(ServiceError::BadRequest(format!(
                "Transição inválida: {:?} → {:?}",
                order.status, payload.new_status
            )));
        }

        if payload.new_status == MaintenanceOrderStatus::Cancelada
            && payload.motivo_cancelamento.is_none()
        {
            return Err(ServiceError::BadRequest(
                "Motivo de cancelamento obrigatório".to_string(),
            ));
        }

        if payload.new_status == MaintenanceOrderStatus::Concluida
            && payload.custo_real.is_none()
        {
            return Err(ServiceError::BadRequest(
                "custo_real obrigatório ao concluir a OS".to_string(),
            ));
        }

        let concluido_por = if payload.new_status == MaintenanceOrderStatus::Concluida {
            updated_by
        } else {
            None
        };
        let cancelado_por = if payload.new_status == MaintenanceOrderStatus::Cancelada {
            updated_by
        } else {
            None
        };

        let updated = self.order_repo
            .advance_status(
                order_id,
                payload.new_status.clone(),
                payload.custo_real,
                payload.data_conclusao,
                payload.notas.as_deref(),
                payload.motivo_cancelamento.as_deref(),
                concluido_por,
                cancelado_por,
                payload.version,
            )
            .await
            .map_err(ServiceError::from)?;

        // Ao concluir ou cancelar: veículo → ATIVO
        if matches!(
            payload.new_status,
            MaintenanceOrderStatus::Concluida | MaintenanceOrderStatus::Cancelada
        ) {
            // Buscamos a versão atual do veículo para OCC
            if let Ok(Some(vehicle)) = self.vehicle_repo.find_by_id(order.vehicle_id).await {
                let _ = self.vehicle_repo
                    .change_operational_status(
                        order.vehicle_id,
                        OperationalStatus::Ativo,
                        vehicle.version,
                        updated_by,
                    )
                    .await;

                let _ = self.status_history_repo
                    .create(
                        order.vehicle_id,
                        Some(VehicleStatus::InMaintenance),
                        VehicleStatus::Active,
                        Some("OS encerrada — veículo retornou ao serviço ativo"),
                        updated_by,
                    )
                    .await;
            }
        }

        Ok(updated)
    }

    // ── RF-MNT-03: Itens de serviço ─────────────────────────────────────────

    pub async fn add_item(
        &self,
        order_id: Uuid,
        payload: CreateMaintenanceOrderItemPayload,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderItemDto, ServiceError> {
        let order = self.order_repo
            .find_by_id(order_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))?;

        if matches!(
            order.status,
            MaintenanceOrderStatus::Concluida | MaintenanceOrderStatus::Cancelada
        ) {
            return Err(ServiceError::BadRequest(
                "Não é possível adicionar itens a uma OS finalizada".to_string(),
            ));
        }

        self.order_repo
            .add_item(
                order_id,
                payload.service_id,
                &payload.descricao,
                payload.quantidade.unwrap_or(Decimal::ONE),
                payload.custo_unitario,
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_items(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<MaintenanceOrderItemDto>, ServiceError> {
        self.order_repo
            .find_by_id(order_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))?;

        self.order_repo
            .list_items(order_id)
            .await
            .map_err(ServiceError::from)
    }

    // ── RF-MNT-04: Listagem e custo ─────────────────────────────────────────

    pub async fn get_order(&self, id: Uuid) -> Result<MaintenanceOrderDto, ServiceError> {
        self.order_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("OS não encontrada".to_string()))
    }

    pub async fn list_orders(
        &self,
        vehicle_id: Option<Uuid>,
        status: Option<MaintenanceOrderStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MaintenanceOrderDto>, i64), ServiceError> {
        self.order_repo
            .list(vehicle_id, status, limit, offset)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn cost_summary(
        &self,
        vehicle_id: Uuid,
    ) -> Result<MaintenanceCostSummaryDto, ServiceError> {
        self.vehicle_repo
            .find_by_id(vehicle_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.order_repo
            .cost_summary(vehicle_id)
            .await
            .map_err(ServiceError::from)
    }
}
