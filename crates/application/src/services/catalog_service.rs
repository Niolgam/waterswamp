use domain::{
    models::catalog::*,
    ports::catalog::*,
};
use crate::errors::ServiceError;
use std::sync::Arc;
use uuid::Uuid;

pub struct CatalogService {
    unit_repo: Arc<dyn UnitOfMeasureRepositoryPort>,
    conversion_repo: Arc<dyn UnitConversionRepositoryPort>,
    catmat_group_repo: Arc<dyn CatmatGroupRepositoryPort>,
    catmat_class_repo: Arc<dyn CatmatClassRepositoryPort>,
    catmat_item_repo: Arc<dyn CatmatItemRepositoryPort>,
    catser_group_repo: Arc<dyn CatserGroupRepositoryPort>,
    catser_class_repo: Arc<dyn CatserClassRepositoryPort>,
    catser_item_repo: Arc<dyn CatserItemRepositoryPort>,
}

impl CatalogService {
    pub fn new(
        unit_repo: Arc<dyn UnitOfMeasureRepositoryPort>,
        conversion_repo: Arc<dyn UnitConversionRepositoryPort>,
        catmat_group_repo: Arc<dyn CatmatGroupRepositoryPort>,
        catmat_class_repo: Arc<dyn CatmatClassRepositoryPort>,
        catmat_item_repo: Arc<dyn CatmatItemRepositoryPort>,
        catser_group_repo: Arc<dyn CatserGroupRepositoryPort>,
        catser_class_repo: Arc<dyn CatserClassRepositoryPort>,
        catser_item_repo: Arc<dyn CatserItemRepositoryPort>,
    ) -> Self {
        Self {
            unit_repo, conversion_repo,
            catmat_group_repo, catmat_class_repo, catmat_item_repo,
            catser_group_repo, catser_class_repo, catser_item_repo,
        }
    }

    // ============================
    // Unit of Measure
    // ============================

    pub async fn create_unit_of_measure(&self, payload: CreateUnitOfMeasurePayload) -> Result<UnitOfMeasureDto, ServiceError> {
        if self.unit_repo.exists_by_symbol(&payload.symbol).await? {
            return Err(ServiceError::Conflict(format!("Unidade com símbolo '{}' já existe", payload.symbol)));
        }
        self.unit_repo.create(&payload.name, &payload.symbol, payload.description.as_deref(), payload.is_base_unit).await.map_err(ServiceError::from)
    }

    pub async fn get_unit_of_measure(&self, id: Uuid) -> Result<UnitOfMeasureDto, ServiceError> {
        self.unit_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))
    }

    pub async fn update_unit_of_measure(&self, id: Uuid, payload: UpdateUnitOfMeasurePayload) -> Result<UnitOfMeasureDto, ServiceError> {
        let _ = self.get_unit_of_measure(id).await?;
        if let Some(ref symbol) = payload.symbol {
            if self.unit_repo.exists_by_symbol_excluding(symbol, id).await? {
                return Err(ServiceError::Conflict(format!("Unidade com símbolo '{}' já existe", symbol)));
            }
        }
        self.unit_repo.update(id, payload.name.as_deref(), payload.symbol.as_deref(), payload.description.as_deref(), payload.is_base_unit).await.map_err(ServiceError::from)
    }

    pub async fn delete_unit_of_measure(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.unit_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_units_of_measure(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<UnitOfMeasureDto>, i64), ServiceError> {
        self.unit_repo.list(limit, offset, search).await.map_err(ServiceError::from)
    }

    // ============================
    // Unit Conversions
    // ============================

    pub async fn create_unit_conversion(&self, payload: CreateUnitConversionPayload) -> Result<UnitConversionDto, ServiceError> {
        let _ = self.unit_repo.find_by_id(payload.from_unit_id).await?.ok_or(ServiceError::NotFound("Unidade de origem não encontrada".to_string()))?;
        let _ = self.unit_repo.find_by_id(payload.to_unit_id).await?.ok_or(ServiceError::NotFound("Unidade de destino não encontrada".to_string()))?;
        if payload.from_unit_id == payload.to_unit_id {
            return Err(ServiceError::BadRequest("Não é possível criar conversão para a mesma unidade".to_string()));
        }
        if self.conversion_repo.exists_conversion(payload.from_unit_id, payload.to_unit_id).await? {
            return Err(ServiceError::Conflict("Conversão já existe entre essas unidades".to_string()));
        }
        if payload.conversion_factor <= rust_decimal::Decimal::ZERO {
            return Err(ServiceError::BadRequest("Fator de conversão deve ser maior que zero".to_string()));
        }
        self.conversion_repo.create(payload.from_unit_id, payload.to_unit_id, payload.conversion_factor).await.map_err(ServiceError::from)
    }

    pub async fn get_unit_conversion(&self, id: Uuid) -> Result<UnitConversionWithDetailsDto, ServiceError> {
        self.conversion_repo.find_with_details_by_id(id).await?.ok_or(ServiceError::NotFound("Conversão de unidade não encontrada".to_string()))
    }

    pub async fn update_unit_conversion(&self, id: Uuid, payload: UpdateUnitConversionPayload) -> Result<UnitConversionDto, ServiceError> {
        let _ = self.conversion_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Conversão de unidade não encontrada".to_string()))?;
        if payload.conversion_factor <= rust_decimal::Decimal::ZERO {
            return Err(ServiceError::BadRequest("Fator de conversão deve ser maior que zero".to_string()));
        }
        self.conversion_repo.update(id, payload.conversion_factor).await.map_err(ServiceError::from)
    }

    pub async fn delete_unit_conversion(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.conversion_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_unit_conversions(&self, limit: i64, offset: i64, from_unit_id: Option<Uuid>, to_unit_id: Option<Uuid>) -> Result<(Vec<UnitConversionWithDetailsDto>, i64), ServiceError> {
        self.conversion_repo.list(limit, offset, from_unit_id, to_unit_id).await.map_err(ServiceError::from)
    }

    // ============================
    // CATMAT Groups
    // ============================

    pub async fn create_catmat_group(&self, payload: CreateCatmatGroupPayload) -> Result<CatmatGroupDto, ServiceError> {
        if self.catmat_group_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!("Grupo CATMAT com código '{}' já existe", payload.code)));
        }
        self.catmat_group_repo.create(&payload.code, &payload.name, payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn get_catmat_group(&self, id: Uuid) -> Result<CatmatGroupDto, ServiceError> {
        self.catmat_group_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Grupo CATMAT não encontrado".to_string()))
    }

    pub async fn update_catmat_group(&self, id: Uuid, payload: UpdateCatmatGroupPayload) -> Result<CatmatGroupDto, ServiceError> {
        let _ = self.get_catmat_group(id).await?;
        if let Some(ref code) = payload.code {
            if self.catmat_group_repo.exists_by_code_excluding(code, id).await? {
                return Err(ServiceError::Conflict(format!("Grupo CATMAT com código '{}' já existe", code)));
            }
        }
        self.catmat_group_repo.update(id, payload.code.as_deref(), payload.name.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_catmat_group(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.catmat_group_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_catmat_groups(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatmatGroupDto>, i64), ServiceError> {
        self.catmat_group_repo.list(limit, offset, search, is_active).await.map_err(ServiceError::from)
    }

    pub async fn get_catmat_tree(&self) -> Result<Vec<CatmatGroupTreeNode>, ServiceError> {
        self.catmat_group_repo.get_tree().await.map_err(ServiceError::from)
    }

    // ============================
    // CATMAT Classes
    // ============================

    pub async fn create_catmat_class(&self, payload: CreateCatmatClassPayload) -> Result<CatmatClassDto, ServiceError> {
        let _ = self.get_catmat_group(payload.group_id).await?;
        if self.catmat_class_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!("Classe CATMAT com código '{}' já existe", payload.code)));
        }
        self.catmat_class_repo.create(payload.group_id, &payload.code, &payload.name, payload.budget_classification_id, payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn get_catmat_class(&self, id: Uuid) -> Result<CatmatClassWithDetailsDto, ServiceError> {
        self.catmat_class_repo.find_with_details_by_id(id).await?.ok_or(ServiceError::NotFound("Classe CATMAT não encontrada".to_string()))
    }

    pub async fn update_catmat_class(&self, id: Uuid, payload: UpdateCatmatClassPayload) -> Result<CatmatClassDto, ServiceError> {
        let _ = self.catmat_class_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Classe CATMAT não encontrada".to_string()))?;
        if let Some(ref code) = payload.code {
            if self.catmat_class_repo.exists_by_code_excluding(code, id).await? {
                return Err(ServiceError::Conflict(format!("Classe CATMAT com código '{}' já existe", code)));
            }
        }
        if let Some(group_id) = payload.group_id {
            let _ = self.get_catmat_group(group_id).await?;
        }
        self.catmat_class_repo.update(id, payload.group_id, payload.code.as_deref(), payload.name.as_deref(), payload.budget_classification_id, payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_catmat_class(&self, id: Uuid) -> Result<bool, ServiceError> {
        if self.catmat_class_repo.has_items(id).await? {
            return Err(ServiceError::Conflict("Não é possível excluir: a classe possui itens vinculados".to_string()));
        }
        self.catmat_class_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_catmat_classes(&self, limit: i64, offset: i64, search: Option<String>, group_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatmatClassWithDetailsDto>, i64), ServiceError> {
        self.catmat_class_repo.list(limit, offset, search, group_id, is_active).await.map_err(ServiceError::from)
    }

    // ============================
    // CATMAT Items (PDM)
    // ============================

    pub async fn create_catmat_item(&self, payload: CreateCatmatItemPayload) -> Result<CatmatItemDto, ServiceError> {
        let _ = self.catmat_class_repo.find_by_id(payload.class_id).await?.ok_or(ServiceError::NotFound("Classe CATMAT não encontrada".to_string()))?;
        let _ = self.unit_repo.find_by_id(payload.unit_of_measure_id).await?.ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))?;
        if self.catmat_item_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!("Item CATMAT com código '{}' já existe", payload.code)));
        }
        if let Some(days) = payload.shelf_life_days {
            if days < 1 { return Err(ServiceError::BadRequest("Validade deve ser maior que 0 dias".to_string())); }
        }
        self.catmat_item_repo.create(
            payload.class_id, payload.unit_of_measure_id, &payload.code, &payload.description,
            payload.supplementary_description.as_deref(), payload.is_sustainable,
            payload.specification.as_deref(), payload.estimated_value,
            payload.search_links.as_deref(), payload.photo_url.as_deref(),
            payload.is_permanent, payload.shelf_life_days, payload.requires_batch_control, payload.is_active,
        ).await.map_err(ServiceError::from)
    }

    pub async fn get_catmat_item(&self, id: Uuid) -> Result<CatmatItemWithDetailsDto, ServiceError> {
        self.catmat_item_repo.find_with_details_by_id(id).await?.ok_or(ServiceError::NotFound("Item CATMAT não encontrado".to_string()))
    }

    pub async fn update_catmat_item(&self, id: Uuid, payload: UpdateCatmatItemPayload) -> Result<CatmatItemDto, ServiceError> {
        let _ = self.catmat_item_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Item CATMAT não encontrado".to_string()))?;
        if let Some(ref code) = payload.code {
            if self.catmat_item_repo.exists_by_code_excluding(code, id).await? {
                return Err(ServiceError::Conflict(format!("Item CATMAT com código '{}' já existe", code)));
            }
        }
        if let Some(class_id) = payload.class_id {
            let _ = self.catmat_class_repo.find_by_id(class_id).await?.ok_or(ServiceError::NotFound("Classe CATMAT não encontrada".to_string()))?;
        }
        if let Some(unit_id) = payload.unit_of_measure_id {
            let _ = self.unit_repo.find_by_id(unit_id).await?.ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))?;
        }
        self.catmat_item_repo.update(
            id, payload.class_id, payload.unit_of_measure_id, payload.code.as_deref(),
            payload.description.as_deref(), payload.supplementary_description.as_deref(),
            payload.is_sustainable, payload.specification.as_deref(), payload.estimated_value,
            payload.search_links.as_deref(), payload.photo_url.as_deref(),
            payload.is_permanent, payload.shelf_life_days, payload.requires_batch_control, payload.is_active,
        ).await.map_err(ServiceError::from)
    }

    pub async fn delete_catmat_item(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.catmat_item_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_catmat_items(&self, limit: i64, offset: i64, search: Option<String>, class_id: Option<Uuid>, is_sustainable: Option<bool>, is_permanent: Option<bool>, is_active: Option<bool>) -> Result<(Vec<CatmatItemWithDetailsDto>, i64), ServiceError> {
        self.catmat_item_repo.list(limit, offset, search, class_id, is_sustainable, is_permanent, is_active).await.map_err(ServiceError::from)
    }

    // ============================
    // CATSER Groups
    // ============================

    pub async fn create_catser_group(&self, payload: CreateCatserGroupPayload) -> Result<CatserGroupDto, ServiceError> {
        if self.catser_group_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!("Grupo CATSER com código '{}' já existe", payload.code)));
        }
        self.catser_group_repo.create(&payload.code, &payload.name, payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn get_catser_group(&self, id: Uuid) -> Result<CatserGroupDto, ServiceError> {
        self.catser_group_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Grupo CATSER não encontrado".to_string()))
    }

    pub async fn update_catser_group(&self, id: Uuid, payload: UpdateCatserGroupPayload) -> Result<CatserGroupDto, ServiceError> {
        let _ = self.get_catser_group(id).await?;
        if let Some(ref code) = payload.code {
            if self.catser_group_repo.exists_by_code_excluding(code, id).await? {
                return Err(ServiceError::Conflict(format!("Grupo CATSER com código '{}' já existe", code)));
            }
        }
        self.catser_group_repo.update(id, payload.code.as_deref(), payload.name.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_catser_group(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.catser_group_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_catser_groups(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatserGroupDto>, i64), ServiceError> {
        self.catser_group_repo.list(limit, offset, search, is_active).await.map_err(ServiceError::from)
    }

    pub async fn get_catser_tree(&self) -> Result<Vec<CatserGroupTreeNode>, ServiceError> {
        self.catser_group_repo.get_tree().await.map_err(ServiceError::from)
    }

    // ============================
    // CATSER Classes
    // ============================

    pub async fn create_catser_class(&self, payload: CreateCatserClassPayload) -> Result<CatserClassDto, ServiceError> {
        let _ = self.get_catser_group(payload.group_id).await?;
        if self.catser_class_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!("Classe CATSER com código '{}' já existe", payload.code)));
        }
        self.catser_class_repo.create(payload.group_id, &payload.code, &payload.name, payload.budget_classification_id, payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn get_catser_class(&self, id: Uuid) -> Result<CatserClassWithDetailsDto, ServiceError> {
        self.catser_class_repo.find_with_details_by_id(id).await?.ok_or(ServiceError::NotFound("Classe CATSER não encontrada".to_string()))
    }

    pub async fn update_catser_class(&self, id: Uuid, payload: UpdateCatserClassPayload) -> Result<CatserClassDto, ServiceError> {
        let _ = self.catser_class_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Classe CATSER não encontrada".to_string()))?;
        if let Some(ref code) = payload.code {
            if self.catser_class_repo.exists_by_code_excluding(code, id).await? {
                return Err(ServiceError::Conflict(format!("Classe CATSER com código '{}' já existe", code)));
            }
        }
        if let Some(group_id) = payload.group_id {
            let _ = self.get_catser_group(group_id).await?;
        }
        self.catser_class_repo.update(id, payload.group_id, payload.code.as_deref(), payload.name.as_deref(), payload.budget_classification_id, payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_catser_class(&self, id: Uuid) -> Result<bool, ServiceError> {
        if self.catser_class_repo.has_items(id).await? {
            return Err(ServiceError::Conflict("Não é possível excluir: a classe possui serviços vinculados".to_string()));
        }
        self.catser_class_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_catser_classes(&self, limit: i64, offset: i64, search: Option<String>, group_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatserClassWithDetailsDto>, i64), ServiceError> {
        self.catser_class_repo.list(limit, offset, search, group_id, is_active).await.map_err(ServiceError::from)
    }

    // ============================
    // CATSER Items (Serviços)
    // ============================

    pub async fn create_catser_item(&self, payload: CreateCatserItemPayload) -> Result<CatserItemDto, ServiceError> {
        let _ = self.catser_class_repo.find_by_id(payload.class_id).await?.ok_or(ServiceError::NotFound("Classe CATSER não encontrada".to_string()))?;
        let _ = self.unit_repo.find_by_id(payload.unit_of_measure_id).await?.ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))?;
        if self.catser_item_repo.exists_by_code(&payload.code).await? {
            return Err(ServiceError::Conflict(format!("Serviço CATSER com código '{}' já existe", payload.code)));
        }
        self.catser_item_repo.create(
            payload.class_id, payload.unit_of_measure_id, &payload.code, &payload.description,
            payload.supplementary_description.as_deref(), payload.specification.as_deref(),
            payload.estimated_value, payload.search_links.as_deref(), payload.is_active,
        ).await.map_err(ServiceError::from)
    }

    pub async fn get_catser_item(&self, id: Uuid) -> Result<CatserItemWithDetailsDto, ServiceError> {
        self.catser_item_repo.find_with_details_by_id(id).await?.ok_or(ServiceError::NotFound("Serviço CATSER não encontrado".to_string()))
    }

    pub async fn update_catser_item(&self, id: Uuid, payload: UpdateCatserItemPayload) -> Result<CatserItemDto, ServiceError> {
        let _ = self.catser_item_repo.find_by_id(id).await?.ok_or(ServiceError::NotFound("Serviço CATSER não encontrado".to_string()))?;
        if let Some(ref code) = payload.code {
            if self.catser_item_repo.exists_by_code_excluding(code, id).await? {
                return Err(ServiceError::Conflict(format!("Serviço CATSER com código '{}' já existe", code)));
            }
        }
        if let Some(class_id) = payload.class_id {
            let _ = self.catser_class_repo.find_by_id(class_id).await?.ok_or(ServiceError::NotFound("Classe CATSER não encontrada".to_string()))?;
        }
        if let Some(unit_id) = payload.unit_of_measure_id {
            let _ = self.unit_repo.find_by_id(unit_id).await?.ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))?;
        }
        self.catser_item_repo.update(
            id, payload.class_id, payload.unit_of_measure_id, payload.code.as_deref(),
            payload.description.as_deref(), payload.supplementary_description.as_deref(),
            payload.specification.as_deref(), payload.estimated_value,
            payload.search_links.as_deref(), payload.is_active,
        ).await.map_err(ServiceError::from)
    }

    pub async fn delete_catser_item(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.catser_item_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_catser_items(&self, limit: i64, offset: i64, search: Option<String>, class_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatserItemWithDetailsDto>, i64), ServiceError> {
        self.catser_item_repo.list(limit, offset, search, class_id, is_active).await.map_err(ServiceError::from)
    }
}
