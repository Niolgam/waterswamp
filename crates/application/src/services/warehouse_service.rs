use crate::errors::ServiceError;
use domain::models::{
    CreateMaterialGroupPayload, CreateMaterialPayload, CreateStockMovementPayload,
    CreateWarehousePayload, CreateWarehouseStockPayload, MaterialDto, MaterialGroupDto,
    MaterialWithGroupDto, MovementType, PaginatedMaterialGroups, PaginatedMaterials,
    StockMovementDto, StockMovementWithDetailsDto, UpdateMaterialGroupPayload,
    UpdateMaterialPayload, UpdateWarehousePayload, UpdateWarehouseStockPayload, WarehouseDto,
    WarehouseStockDto, WarehouseStockWithDetailsDto, WarehouseWithCityDto,
};
use domain::ports::{
    MaterialGroupRepositoryPort, MaterialRepositoryPort, StockMovementRepositoryPort,
    WarehouseRepositoryPort, WarehouseStockRepositoryPort,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct WarehouseService {
    material_group_repo: Arc<dyn MaterialGroupRepositoryPort>,
    material_repo: Arc<dyn MaterialRepositoryPort>,
    warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
    warehouse_stock_repo: Arc<dyn WarehouseStockRepositoryPort>,
    stock_movement_repo: Arc<dyn StockMovementRepositoryPort>,
}

impl WarehouseService {
    pub fn new(
        material_group_repo: Arc<dyn MaterialGroupRepositoryPort>,
        material_repo: Arc<dyn MaterialRepositoryPort>,
        warehouse_repo: Arc<dyn WarehouseRepositoryPort>,
        warehouse_stock_repo: Arc<dyn WarehouseStockRepositoryPort>,
        stock_movement_repo: Arc<dyn StockMovementRepositoryPort>,
    ) -> Self {
        Self {
            material_group_repo,
            material_repo,
            warehouse_repo,
            warehouse_stock_repo,
            stock_movement_repo,
        }
    }

    // ============================
    // Material Group Operations
    // ============================

    pub async fn create_material_group(
        &self,
        payload: CreateMaterialGroupPayload,
    ) -> Result<MaterialGroupDto, ServiceError> {
        // Check if material group code already exists
        if self
            .material_group_repo
            .exists_by_code(&payload.code)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Grupo de material com código '{}' já existe",
                payload.code
            )));
        }

        let is_personnel_exclusive = payload.is_personnel_exclusive.unwrap_or(false);

        let material_group = self
            .material_group_repo
            .create(
                &payload.code,
                &payload.name,
                payload.description.as_deref(),
                payload.expense_element.as_deref(),
                is_personnel_exclusive,
            )
            .await?;

        Ok(material_group)
    }

    pub async fn get_material_group(&self, id: Uuid) -> Result<MaterialGroupDto, ServiceError> {
        self.material_group_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Grupo de material não encontrado".to_string(),
            ))
    }

    pub async fn get_material_group_by_code(
        &self,
        code: &domain::value_objects::MaterialCode,
    ) -> Result<MaterialGroupDto, ServiceError> {
        self.material_group_repo
            .find_by_code(code)
            .await?
            .ok_or(ServiceError::NotFound(
                "Grupo de material não encontrado".to_string(),
            ))
    }

    pub async fn update_material_group(
        &self,
        id: Uuid,
        payload: UpdateMaterialGroupPayload,
    ) -> Result<MaterialGroupDto, ServiceError> {
        // Check if material group exists
        let _ = self.get_material_group(id).await?;

        // If updating code, check for duplicates
        if let Some(ref new_code) = payload.code {
            if self
                .material_group_repo
                .exists_by_code_excluding(new_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Grupo de material com código '{}' já existe",
                    new_code
                )));
            }
        }

        let material_group = self
            .material_group_repo
            .update(
                id,
                payload.code.as_ref(),
                payload.name.as_deref(),
                payload.description.as_deref(),
                payload.expense_element.as_deref(),
                payload.is_personnel_exclusive,
                payload.is_active,
            )
            .await?;

        Ok(material_group)
    }

    pub async fn delete_material_group(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.material_group_repo.delete(id).await?;

        if !deleted {
            return Err(ServiceError::NotFound(
                "Grupo de material não encontrado".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn list_material_groups(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        is_personnel_exclusive: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<PaginatedMaterialGroups, ServiceError> {
        let (material_groups, total) = self
            .material_group_repo
            .list(limit, offset, search, is_personnel_exclusive, is_active)
            .await?;

        Ok(PaginatedMaterialGroups {
            material_groups,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // Material Operations
    // ============================

    pub async fn create_material(
        &self,
        payload: CreateMaterialPayload,
    ) -> Result<MaterialDto, ServiceError> {
        // Verify material group exists
        let _ = self.get_material_group(payload.material_group_id).await?;

        // Check if material name already exists in this group
        if self
            .material_repo
            .exists_by_name_in_group(&payload.name, payload.material_group_id)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Material '{}' já existe neste grupo",
                payload.name
            )));
        }

        let material = self
            .material_repo
            .create(
                payload.material_group_id,
                &payload.name,
                payload.estimated_value,
                &payload.unit_of_measure,
                &payload.specification,
                payload.search_links.as_deref(),
                payload.catmat_code.as_ref(),
                payload.photo_url.as_deref(),
            )
            .await?;

        Ok(material)
    }

    pub async fn get_material(&self, id: Uuid) -> Result<MaterialDto, ServiceError> {
        self.material_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Material não encontrado".to_string()))
    }

    pub async fn get_material_with_group(
        &self,
        id: Uuid,
    ) -> Result<MaterialWithGroupDto, ServiceError> {
        self.material_repo
            .find_with_group_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Material não encontrado".to_string()))
    }

    pub async fn update_material(
        &self,
        id: Uuid,
        payload: UpdateMaterialPayload,
    ) -> Result<MaterialDto, ServiceError> {
        // Check if material exists
        let existing_material = self.get_material(id).await?;

        // If updating material_group_id, verify it exists
        if let Some(new_group_id) = payload.material_group_id {
            let _ = self.get_material_group(new_group_id).await?;
        }

        // If updating name, check for duplicates within the group
        if let Some(ref new_name) = payload.name {
            let group_id = payload
                .material_group_id
                .unwrap_or(existing_material.material_group_id);

            if self
                .material_repo
                .exists_by_name_in_group_excluding(new_name, group_id, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Material '{}' já existe neste grupo",
                    new_name
                )));
            }
        }

        let material = self
            .material_repo
            .update(
                id,
                payload.material_group_id,
                payload.name.as_deref(),
                payload.estimated_value,
                payload.unit_of_measure.as_ref(),
                payload.specification.as_deref(),
                payload.search_links.as_deref(),
                payload.catmat_code.as_ref(),
                payload.photo_url.as_deref(),
                payload.is_active,
            )
            .await?;

        Ok(material)
    }

    pub async fn delete_material(&self, id: Uuid) -> Result<(), ServiceError> {
        let deleted = self.material_repo.delete(id).await?;

        if !deleted {
            return Err(ServiceError::NotFound("Material não encontrado".to_string()));
        }

        Ok(())
    }

    pub async fn list_materials(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        material_group_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<PaginatedMaterials, ServiceError> {
        let (materials, total) = self
            .material_repo
            .list(limit, offset, search, material_group_id, is_active)
            .await?;

        Ok(PaginatedMaterials {
            materials,
            total,
            limit,
            offset,
        })
    }

    // ============================
    // Warehouse Operations
    // ============================

    pub async fn create_warehouse(
        &self,
        payload: CreateWarehousePayload,
    ) -> Result<WarehouseDto, ServiceError> {
        // Check if warehouse code already exists
        if self.warehouse_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!(
                "Almoxarifado com código '{}' já existe",
                payload.code
            )));
        }

        let warehouse = self
            .warehouse_repo
            .create(
                &payload.name,
                &payload.code,
                payload.city_id,
                payload.responsible_user_id,
                payload.address.as_deref(),
                payload.phone.as_deref(),
                payload.email.as_deref(),
            )
            .await?;

        Ok(warehouse)
    }

    pub async fn get_warehouse(&self, id: Uuid) -> Result<WarehouseDto, ServiceError> {
        self.warehouse_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))
    }

    pub async fn get_warehouse_with_city(
        &self,
        id: Uuid,
    ) -> Result<WarehouseWithCityDto, ServiceError> {
        self.warehouse_repo
            .find_with_city_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Almoxarifado não encontrado".to_string(),
            ))
    }

    pub async fn update_warehouse(
        &self,
        id: Uuid,
        payload: UpdateWarehousePayload,
    ) -> Result<WarehouseDto, ServiceError> {
        // Check if warehouse exists
        let _ = self.get_warehouse(id).await?;

        // If updating code, check for duplicates
        if let Some(ref new_code) = payload.code {
            if self
                .warehouse_repo
                .exists_by_code_excluding(new_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Almoxarifado com código '{}' já existe",
                    new_code
                )));
            }
        }

        let warehouse = self
            .warehouse_repo
            .update(
                id,
                payload.name.as_deref(),
                payload.code.as_deref(),
                payload.city_id,
                payload.responsible_user_id,
                payload.address.as_deref(),
                payload.phone.as_deref(),
                payload.email.as_deref(),
                payload.is_active,
            )
            .await?;

        Ok(warehouse)
    }

    // ============================
    // Warehouse Stock Operations with Weighted Average
    // ============================

    /// Registra entrada de material com cálculo automático de média ponderada
    ///
    /// Fórmula: nova_média = (valor_total_atual + valor_entrada) / (qtd_atual + qtd_entrada)
    ///
    /// Exemplo:
    /// - Estoque atual: 100 unidades × R$ 7,00 = R$ 700,00
    /// - Entrada: 50 unidades × R$ 8,00 = R$ 400,00
    /// - Novo estoque: 150 unidades × R$ 7,33 = R$ 1.100,00
    pub async fn register_stock_entry(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
        quantity: Decimal,
        unit_value: Decimal,
        user_id: Uuid,
        document_number: Option<&str>,
        notes: Option<&str>,
    ) -> Result<(WarehouseStockDto, StockMovementDto), ServiceError> {
        // Validate positive values
        if quantity <= Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Quantidade deve ser maior que zero".to_string(),
            ));
        }
        if unit_value < Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Valor unitário não pode ser negativo".to_string(),
            ));
        }

        // Get or create stock record
        let mut stock = match self
            .warehouse_stock_repo
            .find_by_warehouse_and_material(warehouse_id, material_id)
            .await?
        {
            Some(s) => s,
            None => {
                // Create initial stock record
                self.warehouse_stock_repo
                    .create(
                        warehouse_id,
                        material_id,
                        Decimal::ZERO,
                        Decimal::ZERO,
                        None,
                        None,
                        None,
                    )
                    .await?
            }
        };

        let balance_before = stock.quantity;
        let average_before = stock.average_unit_value;

        // Calculate new weighted average
        let current_total_value = stock.quantity * stock.average_unit_value;
        let entry_total_value = quantity * unit_value;
        let new_quantity = stock.quantity + quantity;
        let new_average = if new_quantity > Decimal::ZERO {
            (current_total_value + entry_total_value) / new_quantity
        } else {
            Decimal::ZERO
        };

        // Update stock
        stock = self
            .warehouse_stock_repo
            .update_stock_and_average(stock.id, new_quantity, new_average)
            .await?;

        // Create stock movement record
        let movement = self
            .stock_movement_repo
            .create(
                stock.id,
                MovementType::Entry,
                quantity,
                unit_value,
                entry_total_value,
                balance_before,
                new_quantity,
                average_before,
                new_average,
                chrono::Utc::now(),
                document_number,
                None,
                user_id,
                notes,
            )
            .await?;

        Ok((stock, movement))
    }

    /// Registra saída de material usando média ponderada atual
    pub async fn register_stock_exit(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
        quantity: Decimal,
        user_id: Uuid,
        document_number: Option<&str>,
        requisition_id: Option<Uuid>,
        notes: Option<&str>,
    ) -> Result<(WarehouseStockDto, StockMovementDto), ServiceError> {
        // Validate positive quantity
        if quantity <= Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Quantidade deve ser maior que zero".to_string(),
            ));
        }

        // Get stock record
        let mut stock = self
            .warehouse_stock_repo
            .find_by_warehouse_and_material(warehouse_id, material_id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Estoque não encontrado para este material".to_string(),
            ))?;

        // Check sufficient quantity
        if stock.quantity < quantity {
            return Err(ServiceError::BadRequest(format!(
                "Estoque insuficiente. Disponível: {}, Solicitado: {}",
                stock.quantity, quantity
            )));
        }

        let balance_before = stock.quantity;
        let average_before = stock.average_unit_value;
        let new_quantity = stock.quantity - quantity;

        // Average stays the same on exit (weighted average doesn't change)
        let new_average = stock.average_unit_value;

        // Update stock
        stock = self
            .warehouse_stock_repo
            .update_stock_and_average(stock.id, new_quantity, new_average)
            .await?;

        // Create stock movement record (exit value uses current average)
        let unit_value = stock.average_unit_value;
        let total_value = quantity * unit_value;

        let movement = self
            .stock_movement_repo
            .create(
                stock.id,
                MovementType::Exit,
                quantity,
                unit_value,
                total_value,
                balance_before,
                new_quantity,
                average_before,
                new_average,
                chrono::Utc::now(),
                document_number,
                requisition_id,
                user_id,
                notes,
            )
            .await?;

        Ok((stock, movement))
    }

    /// Registra ajuste de estoque (pode ser positivo ou negativo)
    pub async fn register_stock_adjustment(
        &self,
        warehouse_id: Uuid,
        material_id: Uuid,
        adjustment_quantity: Decimal,
        reason: &str,
        user_id: Uuid,
    ) -> Result<(WarehouseStockDto, StockMovementDto), ServiceError> {
        if adjustment_quantity == Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Quantidade de ajuste não pode ser zero".to_string(),
            ));
        }

        let mut stock = self
            .warehouse_stock_repo
            .find_by_warehouse_and_material(warehouse_id, material_id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Estoque não encontrado para este material".to_string(),
            ))?;

        let balance_before = stock.quantity;
        let average_before = stock.average_unit_value;
        let new_quantity = stock.quantity + adjustment_quantity;

        if new_quantity < Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Ajuste resultaria em estoque negativo".to_string(),
            ));
        }

        // Average stays the same on adjustment
        let new_average = stock.average_unit_value;

        stock = self
            .warehouse_stock_repo
            .update_stock_and_average(stock.id, new_quantity, new_average)
            .await?;

        // Create movement record
        let movement = self
            .stock_movement_repo
            .create(
                stock.id,
                MovementType::Adjustment,
                adjustment_quantity.abs(),
                stock.average_unit_value,
                adjustment_quantity.abs() * stock.average_unit_value,
                balance_before,
                new_quantity,
                average_before,
                new_average,
                chrono::Utc::now(),
                None,
                None,
                user_id,
                Some(reason),
            )
            .await?;

        Ok((stock, movement))
    }

    pub async fn get_warehouse_stock(
        &self,
        id: Uuid,
    ) -> Result<WarehouseStockWithDetailsDto, ServiceError> {
        self.warehouse_stock_repo
            .find_with_details_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Estoque não encontrado".to_string()))
    }
}

#[cfg(test)]
#[path = "warehouse_service_tests.rs"]
mod tests;
