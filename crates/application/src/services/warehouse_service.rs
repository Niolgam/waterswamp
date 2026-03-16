use crate::errors::ServiceError;
use domain::{
    models::warehouse::*,
    ports::warehouse::*,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct WarehouseService {
    warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
    stock_repo: Arc<dyn WarehouseStockRepositoryPort>,
}

impl WarehouseService {
    pub fn new(
        warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
        stock_repo: Arc<dyn WarehouseStockRepositoryPort>,
    ) -> Self {
        Self {
            warehouse_repo,
            stock_repo,
        }
    }

    // ============================
    // Warehouse CRUD
    // ============================

    pub async fn create_warehouse(
        &self,
        payload: CreateWarehousePayload,
    ) -> Result<WarehouseWithDetailsDto, ServiceError> {
        if payload.name.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Nome do almoxarifado é obrigatório".to_string(),
            ));
        }
        if payload.code.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Código do almoxarifado é obrigatório".to_string(),
            ));
        }

        if self
            .warehouse_repo
            .exists_by_code(&payload.code)
            .await
            .map_err(ServiceError::from)?
        {
            return Err(ServiceError::Conflict(format!(
                "Almoxarifado com código '{}' já existe",
                payload.code
            )));
        }

        let allows_transfers = payload.allows_transfers.unwrap_or(true);
        let is_budgetary = payload.is_budgetary.unwrap_or(false);

        let warehouse = self
            .warehouse_repo
            .create(
                &payload.name,
                &payload.code,
                payload.warehouse_type,
                payload.city_id,
                payload.responsible_user_id,
                payload.responsible_unit_id,
                allows_transfers,
                is_budgetary,
                payload.address.as_deref(),
                payload.phone.as_deref(),
                payload.email.as_deref(),
            )
            .await
            .map_err(ServiceError::from)?;

        self.warehouse_repo
            .find_with_details_by_id(warehouse.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar almoxarifado criado".to_string(),
            ))
    }

    pub async fn get_warehouse(
        &self,
        id: Uuid,
    ) -> Result<WarehouseWithDetailsDto, ServiceError> {
        self.warehouse_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Almoxarifado não encontrado".to_string()))
    }

    pub async fn update_warehouse(
        &self,
        id: Uuid,
        payload: UpdateWarehousePayload,
    ) -> Result<WarehouseWithDetailsDto, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Almoxarifado não encontrado".to_string()))?;

        if let Some(ref code) = payload.code {
            if self
                .warehouse_repo
                .exists_by_code_excluding(code, id)
                .await
                .map_err(ServiceError::from)?
            {
                return Err(ServiceError::Conflict(format!(
                    "Almoxarifado com código '{}' já existe",
                    code
                )));
            }
        }

        let _ = self
            .warehouse_repo
            .update(
                id,
                payload.name.as_deref(),
                payload.code.as_deref(),
                payload.warehouse_type,
                payload.city_id,
                payload.responsible_user_id,
                payload.responsible_unit_id,
                payload.allows_transfers,
                payload.is_budgetary,
                payload.address.as_deref(),
                payload.phone.as_deref(),
                payload.email.as_deref(),
                payload.is_active,
            )
            .await
            .map_err(ServiceError::from)?;

        self.warehouse_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal(
                "Falha ao buscar almoxarifado atualizado".to_string(),
            ))
    }

    pub async fn delete_warehouse(&self, id: Uuid) -> Result<bool, ServiceError> {
        let _ = self
            .warehouse_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Almoxarifado não encontrado".to_string()))?;

        self.warehouse_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_warehouses(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        warehouse_type: Option<WarehouseType>,
        city_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<(Vec<WarehouseWithDetailsDto>, i64), ServiceError> {
        self.warehouse_repo
            .list(limit, offset, search, warehouse_type, city_id, is_active)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Warehouse Stock
    // ============================

    pub async fn get_stock(
        &self,
        id: Uuid,
    ) -> Result<WarehouseStockWithDetailsDto, ServiceError> {
        self.stock_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))
    }

    pub async fn list_warehouse_stocks(
        &self,
        warehouse_id: Uuid,
        limit: i64,
        offset: i64,
        search: Option<String>,
        is_blocked: Option<bool>,
    ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), ServiceError> {
        // Ensure warehouse exists
        let _ = self
            .warehouse_repo
            .find_by_id(warehouse_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Almoxarifado não encontrado".to_string()))?;

        self.stock_repo
            .list_by_warehouse(warehouse_id, limit, offset, search, is_blocked)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_stock_params(
        &self,
        id: Uuid,
        payload: UpdateStockParamsPayload,
    ) -> Result<WarehouseStockDto, ServiceError> {
        let _ = self
            .stock_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))?;

        self.stock_repo
            .update_params(
                id,
                payload.min_stock,
                payload.max_stock,
                payload.reorder_point,
                payload.resupply_days,
                payload.location.as_deref(),
                payload.secondary_location.as_deref(),
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn block_stock(
        &self,
        id: Uuid,
        payload: BlockStockPayload,
        blocked_by: Uuid,
    ) -> Result<WarehouseStockDto, ServiceError> {
        let current = self
            .stock_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))?;

        if current.is_blocked {
            return Err(ServiceError::BadRequest(
                "Estoque já está bloqueado".to_string(),
            ));
        }

        if payload.block_reason.trim().is_empty() {
            return Err(ServiceError::BadRequest(
                "Motivo de bloqueio é obrigatório".to_string(),
            ));
        }

        self.stock_repo
            .block(id, &payload.block_reason, blocked_by)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn unblock_stock(&self, id: Uuid) -> Result<WarehouseStockDto, ServiceError> {
        let current = self
            .stock_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))?;

        if !current.is_blocked {
            return Err(ServiceError::BadRequest(
                "Estoque não está bloqueado".to_string(),
            ));
        }

        self.stock_repo
            .unblock(id)
            .await
            .map_err(ServiceError::from)
    }
}
