use domain::{
    errors::RepositoryError,
    models::catalog::*,
    ports::catalog::*,
};
use crate::errors::ServiceError;
use std::sync::Arc;
use uuid::Uuid;

// ============================
// Catalog Service
// ============================

pub struct CatalogService {
    unit_repo: Arc<dyn UnitOfMeasureRepositoryPort>,
    group_repo: Arc<dyn CatalogGroupRepositoryPort>,
    item_repo: Arc<dyn CatalogItemRepositoryPort>,
    conversion_repo: Arc<dyn UnitConversionRepositoryPort>,
}

impl CatalogService {
    pub fn new(
        unit_repo: Arc<dyn UnitOfMeasureRepositoryPort>,
        group_repo: Arc<dyn CatalogGroupRepositoryPort>,
        item_repo: Arc<dyn CatalogItemRepositoryPort>,
        conversion_repo: Arc<dyn UnitConversionRepositoryPort>,
    ) -> Self {
        Self {
            unit_repo,
            group_repo,
            item_repo,
            conversion_repo,
        }
    }

    // ============================
    // Unit of Measure Operations
    // ============================

    pub async fn create_unit_of_measure(
        &self,
        payload: CreateUnitOfMeasurePayload,
    ) -> Result<UnitOfMeasureDto, ServiceError> {
        // Validate symbol uniqueness
        if self.unit_repo.exists_by_symbol(&payload.symbol).await? {
            return Err(ServiceError::Conflict(format!(
                "Unidade com símbolo '{}' já existe",
                payload.symbol
            )));
        }

        self.unit_repo
            .create(
                &payload.name,
                &payload.symbol,
                payload.description.as_deref(),
                payload.is_base_unit,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_unit_of_measure(&self, id: Uuid) -> Result<UnitOfMeasureDto, ServiceError> {
        self.unit_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))
    }

    pub async fn update_unit_of_measure(
        &self,
        id: Uuid,
        payload: UpdateUnitOfMeasurePayload,
    ) -> Result<UnitOfMeasureDto, ServiceError> {
        // Check if exists
        let _ = self.get_unit_of_measure(id).await?;

        // Validate symbol uniqueness if changing
        if let Some(ref symbol) = payload.symbol {
            if self.unit_repo.exists_by_symbol_excluding(symbol, id).await? {
                return Err(ServiceError::Conflict(format!(
                    "Unidade com símbolo '{}' já existe",
                    symbol
                )));
            }
        }

        self.unit_repo
            .update(
                id,
                payload.name.as_deref(),
                payload.symbol.as_deref(),
                payload.description.as_deref(),
                payload.is_base_unit,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_unit_of_measure(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.unit_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_units_of_measure(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<UnitOfMeasureDto>, i64), ServiceError> {
        self.unit_repo
            .list(limit, offset, search)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Catalog Group Operations
    // ============================

    pub async fn create_catalog_group(
        &self,
        payload: CreateCatalogGroupPayload,
    ) -> Result<CatalogGroupDto, ServiceError> {
        // Validate code uniqueness in level
        if self
            .group_repo
            .exists_by_code_in_level(&payload.code, payload.parent_id)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Código '{}' já existe neste nível",
                payload.code
            )));
        }

        // Validate parent exists if specified
        if let Some(parent_id) = payload.parent_id {
            let _parent = self
                .group_repo
                .find_by_id(parent_id)
                .await?
                .ok_or(ServiceError::NotFound("Grupo pai não encontrado".to_string()))?;

            // Check if parent has items (leaf node validation)
            if self.group_repo.has_items(parent_id).await? {
                return Err(ServiceError::BadRequest(
                    "Não é possível criar subgrupos: o grupo pai já possui itens vinculados. Mova os itens primeiro.".to_string()
                ));
            }
        }

        // The database triggers will validate:
        // - Item type inheritance
        // - Budget classification compatibility
        self.group_repo
            .create(
                payload.parent_id,
                &payload.name,
                &payload.code,
                payload.item_type,
                payload.budget_classification_id,
                payload.is_active,
            )
            .await
            .map_err(|e| {
                // Map database trigger errors to friendly messages
                if let RepositoryError::Database(ref msg) = e {
                    if msg.contains("Conflito:") || msg.contains("Conflito Orçamentário:") {
                        return ServiceError::BadRequest(msg.clone());
                    }
                }
                ServiceError::from(e)
            })
    }

    pub async fn get_catalog_group(&self, id: Uuid) -> Result<CatalogGroupWithDetailsDto, ServiceError> {
        self.group_repo
            .find_with_details_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Grupo de catálogo não encontrado".to_string()))
    }

    pub async fn update_catalog_group(
        &self,
        id: Uuid,
        payload: UpdateCatalogGroupPayload,
    ) -> Result<CatalogGroupDto, ServiceError> {
        // Check if exists
        let current = self
            .group_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Grupo de catálogo não encontrado".to_string()))?;

        // Validate code uniqueness if changing
        if let Some(ref code) = payload.code {
            let parent_id = payload.parent_id.flatten().or(current.parent_id);
            if self
                .group_repo
                .exists_by_code_in_level_excluding(code, parent_id, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Código '{}' já existe neste nível",
                    code
                )));
            }
        }

        // Validate parent if changing
        if let Some(Some(new_parent_id)) = payload.parent_id {
            // Can't set itself as parent
            if new_parent_id == id {
                return Err(ServiceError::BadRequest(
                    "Um grupo não pode ser pai de si mesmo".to_string()
                ));
            }

            // Check if parent exists
            let _parent = self
                .group_repo
                .find_by_id(new_parent_id)
                .await?
                .ok_or(ServiceError::NotFound("Grupo pai não encontrado".to_string()))?;

            // Check if parent has items
            if self.group_repo.has_items(new_parent_id).await? {
                return Err(ServiceError::BadRequest(
                    "Não é possível mover para este grupo: o grupo destino já possui itens vinculados".to_string()
                ));
            }
        }

        self.group_repo
            .update(
                id,
                payload.parent_id,
                payload.name.as_deref(),
                payload.code.as_deref(),
                payload.item_type,
                payload.budget_classification_id,
                payload.is_active,
            )
            .await
            .map_err(|e| {
                if let RepositoryError::Database(ref msg) = e {
                    if msg.contains("Conflito:") || msg.contains("Conflito Orçamentário:") {
                        return ServiceError::BadRequest(msg.clone());
                    }
                }
                ServiceError::from(e)
            })
    }

    pub async fn delete_catalog_group(&self, id: Uuid) -> Result<bool, ServiceError> {
        // Check if has children
        if self.group_repo.has_children(id).await? {
            return Err(ServiceError::Conflict(
                "Não é possível excluir: o grupo possui subgrupos".to_string()
            ));
        }

        // Check if has items
        if self.group_repo.has_items(id).await? {
            return Err(ServiceError::Conflict(
                "Não é possível excluir: o grupo possui itens vinculados".to_string()
            ));
        }

        self.group_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_catalog_groups(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        parent_id: Option<Uuid>,
        item_type: Option<ItemType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatalogGroupWithDetailsDto>, i64), ServiceError> {
        self.group_repo
            .list(limit, offset, search, parent_id, item_type, is_active)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_catalog_group_tree(&self) -> Result<Vec<CatalogGroupTreeNode>, ServiceError> {
        self.group_repo
            .get_tree()
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Catalog Item Operations
    // ============================

    pub async fn create_catalog_item(
        &self,
        payload: CreateCatalogItemPayload,
    ) -> Result<CatalogItemDto, ServiceError> {
        // Validate group exists and is a leaf node
        let _ = self
            .group_repo
            .find_by_id(payload.group_id)
            .await?
            .ok_or(ServiceError::NotFound("Grupo de catálogo não encontrado".to_string()))?;

        // Check if group is a leaf node (no children)
        if self.group_repo.has_children(payload.group_id).await? {
            return Err(ServiceError::BadRequest(
                "Itens só podem ser vinculados a grupos folha (sem subgrupos)".to_string()
            ));
        }

        // Validate unit exists
        let _ = self
            .unit_repo
            .find_by_id(payload.unit_of_measure_id)
            .await?
            .ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))?;

        // Validate name uniqueness in group
        if self
            .item_repo
            .exists_by_name_in_group(&payload.name, payload.group_id)
            .await?
        {
            return Err(ServiceError::Conflict(format!(
                "Item '{}' já existe neste grupo",
                payload.name
            )));
        }

        // Validate CATMAT code uniqueness if provided
        if let Some(ref catmat_code) = payload.catmat_code {
            if self.item_repo.exists_by_catmat_code(catmat_code).await? {
                return Err(ServiceError::Conflict(format!(
                    "Código CATMAT '{}' já está em uso",
                    catmat_code
                )));
            }
        }

        // Validate shelf_life_days for stockable items
        if payload.is_stockable && payload.shelf_life_days.is_some() {
            // Additional validation: perishable items should have reasonable shelf life
            if let Some(days) = payload.shelf_life_days {
                if days < 1 {
                    return Err(ServiceError::BadRequest(
                        "Validade deve ser maior que 0 dias".to_string()
                    ));
                }
            }
        }

        self.item_repo
            .create(
                payload.group_id,
                payload.unit_of_measure_id,
                &payload.name,
                payload.catmat_code.as_deref(),
                &payload.specification,
                payload.estimated_value,
                payload.search_links.as_deref(),
                payload.photo_url.as_deref(),
                payload.is_stockable,
                payload.is_permanent,
                payload.shelf_life_days,
                payload.requires_batch_control,
                payload.is_active,
            )
            .await
            .map_err(|e| {
                if let RepositoryError::Database(ref msg) = e {
                    if msg.contains("grupos folha") || msg.contains("subgrupos") {
                        return ServiceError::BadRequest(msg.clone());
                    }
                }
                ServiceError::from(e)
            })
    }

    pub async fn get_catalog_item(&self, id: Uuid) -> Result<CatalogItemWithDetailsDto, ServiceError> {
        self.item_repo
            .find_with_details_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Item de catálogo não encontrado".to_string()))
    }

    pub async fn update_catalog_item(
        &self,
        id: Uuid,
        payload: UpdateCatalogItemPayload,
    ) -> Result<CatalogItemDto, ServiceError> {
        // Check if exists
        let current = self
            .item_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Item de catálogo não encontrado".to_string()))?;

        // Validate group if changing
        if let Some(new_group_id) = payload.group_id {
            let _ = self
                .group_repo
                .find_by_id(new_group_id)
                .await?
                .ok_or(ServiceError::NotFound("Grupo de catálogo não encontrado".to_string()))?;

            // Check if new group is a leaf node
            if self.group_repo.has_children(new_group_id).await? {
                return Err(ServiceError::BadRequest(
                    "Itens só podem ser vinculados a grupos folha (sem subgrupos)".to_string()
                ));
            }
        }

        // Validate unit if changing
        if let Some(new_unit_id) = payload.unit_of_measure_id {
            let _ = self
                .unit_repo
                .find_by_id(new_unit_id)
                .await?
                .ok_or(ServiceError::NotFound("Unidade de medida não encontrada".to_string()))?;
        }

        // Validate name uniqueness if changing
        if let Some(ref name) = payload.name {
            let group_id = payload.group_id.unwrap_or(current.group_id);
            if self
                .item_repo
                .exists_by_name_in_group_excluding(name, group_id, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Item '{}' já existe neste grupo",
                    name
                )));
            }
        }

        // Validate CATMAT code uniqueness if changing
        if let Some(ref catmat_code) = payload.catmat_code {
            if self
                .item_repo
                .exists_by_catmat_code_excluding(catmat_code, id)
                .await?
            {
                return Err(ServiceError::Conflict(format!(
                    "Código CATMAT '{}' já está em uso",
                    catmat_code
                )));
            }
        }

        self.item_repo
            .update(
                id,
                payload.group_id,
                payload.unit_of_measure_id,
                payload.name.as_deref(),
                payload.catmat_code.as_deref(),
                payload.specification.as_deref(),
                payload.estimated_value,
                payload.search_links.as_deref(),
                payload.photo_url.as_deref(),
                payload.is_stockable,
                payload.is_permanent,
                payload.shelf_life_days,
                payload.requires_batch_control,
                payload.is_active,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_catalog_item(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.item_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_catalog_items(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        group_id: Option<Uuid>,
        is_stockable: Option<bool>,
        is_permanent: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatalogItemWithDetailsDto>, i64), ServiceError> {
        self.item_repo
            .list(limit, offset, search, group_id, is_stockable, is_permanent, is_active)
            .await
            .map_err(ServiceError::from)
    }

    // ============================
    // Unit Conversion Operations
    // ============================

    pub async fn create_unit_conversion(
        &self,
        payload: CreateUnitConversionPayload,
    ) -> Result<UnitConversionDto, ServiceError> {
        // Validate units exist
        let _ = self
            .unit_repo
            .find_by_id(payload.from_unit_id)
            .await?
            .ok_or(ServiceError::NotFound("Unidade de origem não encontrada".to_string()))?;

        let _ = self
            .unit_repo
            .find_by_id(payload.to_unit_id)
            .await?
            .ok_or(ServiceError::NotFound("Unidade de destino não encontrada".to_string()))?;

        // Can't convert to self
        if payload.from_unit_id == payload.to_unit_id {
            return Err(ServiceError::BadRequest(
                "Não é possível criar conversão para a mesma unidade".to_string()
            ));
        }

        // Check if conversion already exists
        if self
            .conversion_repo
            .exists_conversion(payload.from_unit_id, payload.to_unit_id)
            .await?
        {
            return Err(ServiceError::Conflict(
                "Conversão já existe entre essas unidades".to_string()
            ));
        }

        // Validate conversion factor
        if payload.conversion_factor <= rust_decimal::Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Fator de conversão deve ser maior que zero".to_string()
            ));
        }

        self.conversion_repo
            .create(
                payload.from_unit_id,
                payload.to_unit_id,
                payload.conversion_factor,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_unit_conversion(&self, id: Uuid) -> Result<UnitConversionWithDetailsDto, ServiceError> {
        self.conversion_repo
            .find_with_details_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Conversão de unidade não encontrada".to_string()))
    }

    pub async fn update_unit_conversion(
        &self,
        id: Uuid,
        payload: UpdateUnitConversionPayload,
    ) -> Result<UnitConversionDto, ServiceError> {
        // Check if exists
        let _ = self
            .conversion_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Conversão de unidade não encontrada".to_string()))?;

        // Validate conversion factor
        if payload.conversion_factor <= rust_decimal::Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "Fator de conversão deve ser maior que zero".to_string()
            ));
        }

        self.conversion_repo
            .update(id, payload.conversion_factor)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_unit_conversion(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.conversion_repo
            .delete(id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_unit_conversions(
        &self,
        limit: i64,
        offset: i64,
        from_unit_id: Option<Uuid>,
        to_unit_id: Option<Uuid>,
    ) -> Result<(Vec<UnitConversionWithDetailsDto>, i64), ServiceError> {
        self.conversion_repo
            .list(limit, offset, from_unit_id, to_unit_id)
            .await
            .map_err(ServiceError::from)
    }
}
