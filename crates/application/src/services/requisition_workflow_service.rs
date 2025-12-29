use crate::errors::ServiceError;
use domain::models::{
    CreateRequisitionItemPayload, CreateRequisitionPayload, FulfillRequisitionItemPayload,
    FulfillRequisitionPayload, RequisitionDto, RequisitionItemDto, RequisitionStatus,
    RequisitionWithDetailsDto,
};
use domain::ports::{
    MaterialRepositoryPort, RequisitionItemRepositoryPort, RequisitionRepositoryPort,
    WarehouseRepositoryPort,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct RequisitionWorkflowService {
    requisition_repo: Arc<dyn RequisitionRepositoryPort>,
    requisition_item_repo: Arc<dyn RequisitionItemRepositoryPort>,
    warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
    material_repo: Arc<dyn MaterialRepositoryPort>,
}

impl RequisitionWorkflowService {
    pub fn new(
        requisition_repo: Arc<dyn RequisitionRepositoryPort>,
        requisition_item_repo: Arc<dyn RequisitionItemRepositoryPort>,
        warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
        material_repo: Arc<dyn MaterialRepositoryPort>,
    ) -> Self {
        Self {
            requisition_repo,
            requisition_item_repo,
            warehouse_repo,
            material_repo,
        }
    }

    /// Cria uma nova requisição com seus itens
    pub async fn create_requisition(
        &self,
        warehouse_id: Uuid,
        requester_id: Uuid,
        items: Vec<CreateRequisitionItemPayload>,
        notes: Option<String>,
    ) -> Result<(RequisitionDto, Vec<RequisitionItemDto>), ServiceError> {
        // Validate warehouse exists
        self.warehouse_repo
            .find_by_id(warehouse_id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))?;

        // Validate at least one item
        if items.is_empty() {
            return Err(ServiceError::BadRequest(
                "Requisição deve ter pelo menos um item".to_string(),
            ));
        }

        // Validate all materials exist and calculate total value
        let mut total_value = Decimal::ZERO;
        for item in &items {
            let material = self
                .material_repo
                .find_by_id(item.material_id)
                .await?
                .ok_or(ServiceError::NotFound(format!(
                    "Material {} não encontrado",
                    item.material_id
                )))?;

            // Use estimated value for calculation
            let item_total = item.requested_quantity * material.estimated_value;
            total_value += item_total;
        }

        // Create requisition
        let requisition = self
            .requisition_repo
            .create(warehouse_id, requester_id, total_value, notes.as_deref())
            .await?;

        // Create requisition items
        let mut created_items = Vec::new();
        for item in items {
            let material = self
                .material_repo
                .find_by_id(item.material_id)
                .await?
                .unwrap(); // Safe unwrap - já validamos antes

            let unit_value = material.estimated_value;
            let item_total = item.requested_quantity * unit_value;

            let requisition_item = self
                .requisition_item_repo
                .create(
                    requisition.id,
                    item.material_id,
                    item.requested_quantity,
                    unit_value,
                    item_total,
                )
                .await?;

            created_items.push(requisition_item);
        }

        Ok((requisition, created_items))
    }

    /// Aprova uma requisição
    pub async fn approve_requisition(
        &self,
        requisition_id: Uuid,
        approved_by: Uuid,
        notes: Option<String>,
    ) -> Result<RequisitionDto, ServiceError> {
        // Check if requisition exists and is in correct status
        let requisition = self
            .requisition_repo
            .find_by_id(requisition_id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser aprovada. Status atual: {:?}",
                requisition.status
            )));
        }

        // Approve requisition
        let approved = self
            .requisition_repo
            .approve(requisition_id, approved_by, notes.as_deref())
            .await?;

        Ok(approved)
    }

    /// Rejeita uma requisição
    pub async fn reject_requisition(
        &self,
        requisition_id: Uuid,
        rejection_reason: String,
    ) -> Result<RequisitionDto, ServiceError> {
        // Check if requisition exists and is in correct status
        let requisition = self
            .requisition_repo
            .find_by_id(requisition_id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser rejeitada. Status atual: {:?}",
                requisition.status
            )));
        }

        // Validate rejection reason
        if rejection_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo da rejeição é obrigatório".to_string(),
            ));
        }

        // Reject requisition
        let rejected = self
            .requisition_repo
            .reject(requisition_id, &rejection_reason)
            .await?;

        Ok(rejected)
    }

    /// Atende uma requisição (total ou parcialmente)
    pub async fn fulfill_requisition(
        &self,
        requisition_id: Uuid,
        fulfilled_by: Uuid,
        items: Vec<FulfillRequisitionItemPayload>,
        notes: Option<String>,
    ) -> Result<RequisitionDto, ServiceError> {
        // Check if requisition exists and is approved
        let requisition = self
            .requisition_repo
            .find_by_id(requisition_id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))?;

        if requisition.status != RequisitionStatus::Approved {
            return Err(ServiceError::BadRequest(format!(
                "Requisição não pode ser atendida. Status atual: {:?}",
                requisition.status
            )));
        }

        // Get all requisition items
        let requisition_items = self
            .requisition_item_repo
            .find_by_requisition_id(requisition_id)
            .await?;

        // Update fulfilled quantities
        for fulfill_item in items {
            // Find the requisition item
            let req_item = requisition_items
                .iter()
                .find(|i| i.id == fulfill_item.requisition_item_id)
                .ok_or(ServiceError::NotFound(format!(
                    "Item {} não encontrado na requisição",
                    fulfill_item.requisition_item_id
                )))?;

            // Validate fulfilled quantity
            if fulfill_item.fulfilled_quantity > req_item.requested_quantity {
                return Err(ServiceError::BadRequest(format!(
                    "Quantidade atendida ({}) não pode ser maior que a solicitada ({})",
                    fulfill_item.fulfilled_quantity, req_item.requested_quantity
                )));
            }

            // Update fulfilled quantity
            self.requisition_item_repo
                .update_fulfilled_quantity(
                    fulfill_item.requisition_item_id,
                    fulfill_item.fulfilled_quantity,
                )
                .await?;
        }

        // Mark requisition as fulfilled
        let fulfilled = self
            .requisition_repo
            .fulfill(requisition_id, fulfilled_by, notes.as_deref())
            .await?;

        Ok(fulfilled)
    }

    /// Obtém uma requisição com detalhes e itens
    pub async fn get_requisition_with_details(
        &self,
        requisition_id: Uuid,
    ) -> Result<RequisitionWithDetailsDto, ServiceError> {
        let mut requisition = self
            .requisition_repo
            .find_with_details_by_id(requisition_id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))?;

        // Populate items
        let items = self
            .requisition_item_repo
            .find_by_requisition_id(requisition_id)
            .await?;

        requisition.items = items;

        Ok(requisition)
    }

    /// Lista requisições com filtros
    pub async fn list_requisitions(
        &self,
        limit: i64,
        offset: i64,
        warehouse_id: Option<Uuid>,
        requester_id: Option<Uuid>,
        status: Option<RequisitionStatus>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<RequisitionWithDetailsDto>, i64), ServiceError> {
        let (mut requisitions, total) = self
            .requisition_repo
            .list(
                limit,
                offset,
                warehouse_id,
                requester_id,
                status,
                start_date,
                end_date,
            )
            .await?;

        // Populate items for each requisition
        for requisition in &mut requisitions {
            let items = self
                .requisition_item_repo
                .find_by_requisition_id(requisition.id)
                .await?;
            requisition.items = items;
        }

        Ok((requisitions, total))
    }

    /// Cancela uma requisição (apenas se ainda estiver pendente)
    pub async fn cancel_requisition(
        &self,
        requisition_id: Uuid,
    ) -> Result<RequisitionDto, ServiceError> {
        let requisition = self
            .requisition_repo
            .find_by_id(requisition_id)
            .await?
            .ok_or(ServiceError::NotFound("Requisição não encontrada".to_string()))?;

        if requisition.status != RequisitionStatus::Pending {
            return Err(ServiceError::BadRequest(format!(
                "Apenas requisições pendentes podem ser canceladas. Status atual: {:?}",
                requisition.status
            )));
        }

        let cancelled = self
            .requisition_repo
            .update_status(requisition_id, RequisitionStatus::Cancelled)
            .await?;

        Ok(cancelled)
    }
}
