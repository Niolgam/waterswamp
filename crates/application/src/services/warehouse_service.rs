use crate::errors::ServiceError;
use domain::models::{
    CreateMaterialGroupPayload, CreateMaterialPayload, MaterialDto, MaterialGroupDto,
    MaterialWithGroupDto, PaginatedMaterialGroups, PaginatedMaterials, UpdateMaterialGroupPayload,
    UpdateMaterialPayload,
};
use domain::ports::{MaterialGroupRepositoryPort, MaterialRepositoryPort};
use std::sync::Arc;
use uuid::Uuid;

pub struct WarehouseService {
    material_group_repo: Arc<dyn MaterialGroupRepositoryPort>,
    material_repo: Arc<dyn MaterialRepositoryPort>,
}

impl WarehouseService {
    pub fn new(
        material_group_repo: Arc<dyn MaterialGroupRepositoryPort>,
        material_repo: Arc<dyn MaterialRepositoryPort>,
    ) -> Self {
        Self {
            material_group_repo,
            material_repo,
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
}
