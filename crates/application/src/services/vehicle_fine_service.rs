use crate::errors::ServiceError;
use domain::{
    models::vehicle_fine::*,
    ports::vehicle_fine::*,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct VehicleFineService {
    fine_type_repo: Arc<dyn VehicleFineTypeRepositoryPort>,
    fine_repo: Arc<dyn VehicleFineRepositoryPort>,
}

impl VehicleFineService {
    pub fn new(
        fine_type_repo: Arc<dyn VehicleFineTypeRepositoryPort>,
        fine_repo: Arc<dyn VehicleFineRepositoryPort>,
    ) -> Self {
        Self {
            fine_type_repo,
            fine_repo,
        }
    }

    // ============================
    // Fine Type operations
    // ============================

    pub async fn create_fine_type(
        &self,
        payload: CreateVehicleFineTypePayload,
        created_by: Option<Uuid>,
    ) -> Result<VehicleFineTypeDto, ServiceError> {
        let code = payload.code.trim().to_uppercase();
        if code.is_empty() {
            return Err(ServiceError::BadRequest("Código da infração é obrigatório".to_string()));
        }
        if payload.description.trim().is_empty() {
            return Err(ServiceError::BadRequest("Descrição da infração é obrigatória".to_string()));
        }
        if payload.points < 0 {
            return Err(ServiceError::BadRequest("Pontos devem ser zero ou positivos".to_string()));
        }
        if payload.fine_amount < Decimal::ZERO {
            return Err(ServiceError::BadRequest("Valor da multa deve ser zero ou positivo".to_string()));
        }

        if self.fine_type_repo.exists_by_code(&code).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict("Código de infração já cadastrado".to_string()));
        }

        self.fine_type_repo
            .create(
                &code,
                payload.description.trim(),
                &payload.severity,
                payload.points,
                payload.fine_amount,
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_fine_type(&self, id: Uuid) -> Result<VehicleFineTypeDto, ServiceError> {
        self.fine_type_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Tipo de multa não encontrado".to_string()))
    }

    pub async fn update_fine_type(
        &self,
        id: Uuid,
        payload: UpdateVehicleFineTypePayload,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineTypeDto, ServiceError> {
        let _ = self.get_fine_type(id).await?;

        if let Some(ref code) = payload.code {
            let normalized = code.trim().to_uppercase();
            if normalized.is_empty() {
                return Err(ServiceError::BadRequest("Código da infração é obrigatório".to_string()));
            }
            if self.fine_type_repo.exists_by_code_excluding(&normalized, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict("Código de infração já cadastrado".to_string()));
            }
        }
        if let Some(ref description) = payload.description {
            if description.trim().is_empty() {
                return Err(ServiceError::BadRequest("Descrição da infração é obrigatória".to_string()));
            }
        }
        if let Some(points) = payload.points {
            if points < 0 {
                return Err(ServiceError::BadRequest("Pontos devem ser zero ou positivos".to_string()));
            }
        }
        if let Some(amount) = payload.fine_amount {
            if amount < Decimal::ZERO {
                return Err(ServiceError::BadRequest("Valor da multa deve ser zero ou positivo".to_string()));
            }
        }

        let code_str = payload.code.as_ref().map(|c| c.trim().to_uppercase());
        let desc_str = payload.description.as_ref().map(|d| d.trim().to_string());

        self.fine_type_repo
            .update(
                id,
                code_str.as_deref(),
                desc_str.as_deref(),
                payload.severity.as_ref(),
                payload.points,
                payload.fine_amount,
                payload.is_active,
                updated_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_fine_type(&self, id: Uuid) -> Result<bool, ServiceError> {
        let _ = self.get_fine_type(id).await?;
        self.fine_type_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_fine_types(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        severity: Option<FineSeverity>,
        is_active: Option<bool>,
    ) -> Result<(Vec<VehicleFineTypeDto>, i64), ServiceError> {
        self.fine_type_repo
            .list(limit, offset, search, severity, is_active)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Fine operations
    // ============================

    pub async fn create_fine(
        &self,
        payload: CreateVehicleFinePayload,
        created_by: Option<Uuid>,
    ) -> Result<VehicleFineWithDetailsDto, ServiceError> {
        if payload.fine_amount < Decimal::ZERO {
            return Err(ServiceError::BadRequest("Valor da multa deve ser zero ou positivo".to_string()));
        }
        if let Some(discount) = payload.discount_amount {
            if discount < Decimal::ZERO {
                return Err(ServiceError::BadRequest("Valor do desconto deve ser zero ou positivo".to_string()));
            }
        }

        let fine = self.fine_repo
            .create(
                payload.vehicle_id,
                payload.fine_type_id,
                payload.supplier_id,
                payload.driver_id,
                payload.auto_number.as_deref(),
                payload.fine_date,
                payload.notification_date,
                payload.due_date,
                payload.location.as_deref(),
                payload.sei_process_number.as_deref(),
                payload.fine_amount,
                payload.discount_amount,
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.fine_repo
            .find_with_details_by_id(fine.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar multa criada".to_string()))
    }

    pub async fn get_fine(&self, id: Uuid) -> Result<VehicleFineWithDetailsDto, ServiceError> {
        self.fine_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Multa não encontrada".to_string()))
    }

    pub async fn update_fine(
        &self,
        id: Uuid,
        payload: UpdateVehicleFinePayload,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineWithDetailsDto, ServiceError> {
        let current = self.fine_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Multa não encontrada".to_string()))?;

        if current.is_deleted {
            return Err(ServiceError::BadRequest("Não é possível editar uma multa excluída".to_string()));
        }

        if let Some(amount) = payload.fine_amount {
            if amount < Decimal::ZERO {
                return Err(ServiceError::BadRequest("Valor da multa deve ser zero ou positivo".to_string()));
            }
        }
        if let Some(discount) = payload.discount_amount {
            if discount < Decimal::ZERO {
                return Err(ServiceError::BadRequest("Valor do desconto deve ser zero ou positivo".to_string()));
            }
        }
        if let Some(paid) = payload.paid_amount {
            if paid < Decimal::ZERO {
                return Err(ServiceError::BadRequest("Valor pago deve ser zero ou positivo".to_string()));
            }
        }

        let _ = self.fine_repo
            .update(
                id,
                payload.vehicle_id,
                payload.fine_type_id,
                payload.supplier_id,
                payload.driver_id,
                payload.auto_number.as_deref(),
                payload.fine_date,
                payload.notification_date,
                payload.due_date,
                payload.location.as_deref(),
                payload.sei_process_number.as_deref(),
                payload.fine_amount,
                payload.discount_amount,
                payload.paid_amount,
                payload.payment_date,
                payload.payment_status.as_ref(),
                payload.notes.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.fine_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar multa atualizada".to_string()))
    }

    pub async fn delete_fine(&self, id: Uuid, deleted_by: Option<Uuid>) -> Result<bool, ServiceError> {
        let _ = self.fine_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Multa não encontrada".to_string()))?;
        self.fine_repo.soft_delete(id, deleted_by).await.map_err(ServiceError::from)
    }

    pub async fn restore_fine(&self, id: Uuid) -> Result<VehicleFineWithDetailsDto, ServiceError> {
        let current = self.fine_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Multa não encontrada".to_string()))?;

        if !current.is_deleted {
            return Err(ServiceError::BadRequest("Multa não está excluída".to_string()));
        }

        self.fine_repo.restore(id).await.map_err(ServiceError::from)?;

        self.fine_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar multa restaurada".to_string()))
    }

    pub async fn list_fines(
        &self,
        limit: i64,
        offset: i64,
        vehicle_id: Option<Uuid>,
        fine_type_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        payment_status: Option<FinePaymentStatus>,
        search: Option<String>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleFineWithDetailsDto>, i64), ServiceError> {
        self.fine_repo
            .list(limit, offset, vehicle_id, fine_type_id, supplier_id, driver_id, payment_status, search, include_deleted)
            .await
            .map_err(ServiceError::from)
    }
}
