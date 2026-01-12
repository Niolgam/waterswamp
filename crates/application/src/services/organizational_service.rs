use domain::models::organizational::*;
use domain::ports::{
    OrganizationRepositoryPort, OrganizationalUnitCategoryRepositoryPort,
    OrganizationalUnitRepositoryPort, OrganizationalUnitTypeRepositoryPort,
    RepositoryError, SystemSettingsRepositoryPort,
};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Service Errors
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Repository error: {0}")]
    Repository(String),
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => ServiceError::NotFound("Resource not found".to_string()),
            RepositoryError::Database(msg) => ServiceError::Repository(msg),
            RepositoryError::Conflict(msg) => ServiceError::Conflict(msg),
        }
    }
}

impl ServiceError {
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            ServiceError::NotFound(_) => http::StatusCode::NOT_FOUND,
            ServiceError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
            ServiceError::Conflict(_) => http::StatusCode::CONFLICT,
            ServiceError::Repository(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<&ServiceError> for http::StatusCode {
    fn from(err: &ServiceError) -> Self {
        err.status_code()
    }
}

// ============================================================================
// System Settings Service
// ============================================================================

pub struct SystemSettingsService {
    repo: Arc<dyn SystemSettingsRepositoryPort>,
}

impl SystemSettingsService {
    pub fn new(repo: Arc<dyn SystemSettingsRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, key: &str) -> Result<SystemSettingDto, ServiceError> {
        self.repo
            .get(key)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Setting '{}' not found", key)))
    }

    pub async fn list(
        &self,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SystemSettingDto>, i64), ServiceError> {
        Ok(self.repo.list(category, limit, offset).await?)
    }

    pub async fn create(
        &self,
        payload: CreateSystemSettingPayload,
    ) -> Result<SystemSettingDto, ServiceError> {
        // Validate value_type
        if !["string", "number", "boolean", "json"].contains(&payload.value_type.as_str()) {
            return Err(ServiceError::BadRequest(
                "Invalid value_type. Must be: string, number, boolean, or json".to_string(),
            ));
        }

        Ok(self.repo.create(payload).await?)
    }

    pub async fn update(
        &self,
        key: &str,
        payload: UpdateSystemSettingPayload,
        updated_by: Option<Uuid>,
    ) -> Result<SystemSettingDto, ServiceError> {
        // Ensure setting exists
        let _ = self.get(key).await?;

        Ok(self.repo.update(key, payload, updated_by).await?)
    }

    pub async fn delete(&self, key: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete(key).await?)
    }

    pub async fn get_value<T>(&self, key: &str) -> Result<T, ServiceError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.repo
            .get_value::<T>(key)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Setting '{}' not found", key)))
    }

    pub async fn get_bool(&self, key: &str) -> Result<bool, ServiceError> {
        self.get_value::<bool>(key).await
    }

    pub async fn get_string(&self, key: &str) -> Result<String, ServiceError> {
        self.get_value::<String>(key).await
    }

    pub async fn get_i64(&self, key: &str) -> Result<i64, ServiceError> {
        self.get_value::<i64>(key).await
    }

    pub async fn get_f64(&self, key: &str) -> Result<f64, ServiceError> {
        self.get_value::<f64>(key).await
    }
}

// ============================================================================
// Organization Service
// ============================================================================

pub struct OrganizationService {
    repo: Arc<dyn OrganizationRepositoryPort>,
}

impl OrganizationService {
    pub fn new(repo: Arc<dyn OrganizationRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, id: Uuid) -> Result<OrganizationDto, ServiceError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Organization {} not found", id)))
    }

    pub async fn get_by_cnpj(&self, cnpj: &str) -> Result<OrganizationDto, ServiceError> {
        self.repo
            .find_by_cnpj(cnpj)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Organization with CNPJ {} not found", cnpj)))
    }

    pub async fn get_by_siorg_code(&self, siorg_code: i32) -> Result<OrganizationDto, ServiceError> {
        self.repo
            .find_by_siorg_code(siorg_code)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFound(format!(
                    "Organization with SIORG code {} not found",
                    siorg_code
                ))
            })
    }

    pub async fn get_main(&self) -> Result<OrganizationDto, ServiceError> {
        self.repo
            .find_main()
            .await?
            .ok_or_else(|| ServiceError::NotFound("Main organization not found".to_string()))
    }

    pub async fn list(
        &self,
        is_active: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationDto>, i64), ServiceError> {
        Ok(self.repo.list(is_active, limit, offset).await?)
    }

    pub async fn create(
        &self,
        payload: CreateOrganizationPayload,
    ) -> Result<OrganizationDto, ServiceError> {
        // Validate CNPJ format (14 digits)
        if !payload.cnpj.chars().all(|c| c.is_ascii_digit()) || payload.cnpj.len() != 14 {
            return Err(ServiceError::BadRequest(
                "CNPJ must be 14 digits without punctuation".to_string(),
            ));
        }

        // Check if CNPJ already exists
        if self.repo.find_by_cnpj(&payload.cnpj).await?.is_some() {
            return Err(ServiceError::Conflict(format!(
                "CNPJ {} already exists",
                payload.cnpj
            )));
        }

        // Check if SIORG code already exists
        if self.repo.find_by_siorg_code(payload.siorg_code).await?.is_some() {
            return Err(ServiceError::Conflict(format!(
                "SIORG code {} already exists",
                payload.siorg_code
            )));
        }

        Ok(self.repo.create(payload).await?)
    }

    pub async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationPayload,
    ) -> Result<OrganizationDto, ServiceError> {
        // Ensure organization exists
        let _ = self.get(id).await?;

        Ok(self.repo.update(id, payload).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        Ok(self.repo.delete(id).await?)
    }
}

// ============================================================================
// Organizational Unit Category Service
// ============================================================================

pub struct OrganizationalUnitCategoryService {
    repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
}

impl OrganizationalUnitCategoryService {
    pub fn new(repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, id: Uuid) -> Result<OrganizationalUnitCategoryDto, ServiceError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Category {} not found", id)))
    }

    pub async fn get_by_name(
        &self,
        name: &str,
    ) -> Result<OrganizationalUnitCategoryDto, ServiceError> {
        self.repo
            .find_by_name(name)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Category '{}' not found", name)))
    }

    pub async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitCategoryDto>, i64), ServiceError> {
        Ok(self.repo.list(is_active, is_siorg_managed, limit, offset).await?)
    }

    pub async fn create(
        &self,
        payload: CreateOrganizationalUnitCategoryPayload,
    ) -> Result<OrganizationalUnitCategoryDto, ServiceError> {
        // Check if name already exists
        if self.repo.find_by_name(&payload.name).await?.is_some() {
            return Err(ServiceError::Conflict(format!(
                "Category '{}' already exists",
                payload.name
            )));
        }

        // Check if SIORG code already exists (if provided)
        if let Some(siorg_code) = payload.siorg_code {
            if self.repo.find_by_siorg_code(siorg_code).await?.is_some() {
                return Err(ServiceError::Conflict(format!(
                    "SIORG code {} already exists",
                    siorg_code
                )));
            }
        }

        Ok(self.repo.create(payload).await?)
    }

    pub async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationalUnitCategoryPayload,
    ) -> Result<OrganizationalUnitCategoryDto, ServiceError> {
        // Ensure category exists
        let _ = self.get(id).await?;

        // If updating name, check uniqueness
        if let Some(ref new_name) = payload.name {
            if let Some(existing) = self.repo.find_by_name(new_name).await? {
                if existing.id != id {
                    return Err(ServiceError::Conflict(format!(
                        "Category '{}' already exists",
                        new_name
                    )));
                }
            }
        }

        Ok(self.repo.update(id, payload).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        Ok(self.repo.delete(id).await?)
    }
}

// ============================================================================
// Organizational Unit Type Service
// ============================================================================

pub struct OrganizationalUnitTypeService {
    repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
}

impl OrganizationalUnitTypeService {
    pub fn new(repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, id: Uuid) -> Result<OrganizationalUnitTypeDto, ServiceError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Unit type {} not found", id)))
    }

    pub async fn get_by_code(&self, code: &str) -> Result<OrganizationalUnitTypeDto, ServiceError> {
        self.repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Unit type '{}' not found", code)))
    }

    pub async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitTypeDto>, i64), ServiceError> {
        Ok(self.repo.list(is_active, is_siorg_managed, limit, offset).await?)
    }

    pub async fn create(
        &self,
        payload: CreateOrganizationalUnitTypePayload,
    ) -> Result<OrganizationalUnitTypeDto, ServiceError> {
        // Check if code already exists
        if self.repo.find_by_code(&payload.code).await?.is_some() {
            return Err(ServiceError::Conflict(format!(
                "Unit type code '{}' already exists",
                payload.code
            )));
        }

        // Check if SIORG code already exists (if provided)
        if let Some(siorg_code) = payload.siorg_code {
            if self.repo.find_by_siorg_code(siorg_code).await?.is_some() {
                return Err(ServiceError::Conflict(format!(
                    "SIORG code {} already exists",
                    siorg_code
                )));
            }
        }

        Ok(self.repo.create(payload).await?)
    }

    pub async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationalUnitTypePayload,
    ) -> Result<OrganizationalUnitTypeDto, ServiceError> {
        // Ensure type exists
        let _ = self.get(id).await?;

        Ok(self.repo.update(id, payload).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        Ok(self.repo.delete(id).await?)
    }
}

// ============================================================================
// Organizational Unit Service
// ============================================================================

pub struct OrganizationalUnitService {
    unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
    org_repo: Arc<dyn OrganizationRepositoryPort>,
    category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
    type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
    settings_repo: Arc<dyn SystemSettingsRepositoryPort>,
}

impl OrganizationalUnitService {
    pub fn new(
        unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
        org_repo: Arc<dyn OrganizationRepositoryPort>,
        category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
        type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
        settings_repo: Arc<dyn SystemSettingsRepositoryPort>,
    ) -> Self {
        Self {
            unit_repo,
            org_repo,
            category_repo,
            type_repo,
            settings_repo,
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<OrganizationalUnitDto, ServiceError> {
        self.unit_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Organizational unit {} not found", id)))
    }

    pub async fn get_with_details(
        &self,
        id: Uuid,
    ) -> Result<OrganizationalUnitWithDetailsDto, ServiceError> {
        self.unit_repo
            .find_by_id_with_details(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Organizational unit {} not found", id)))
    }

    pub async fn get_by_siorg_code(
        &self,
        siorg_code: i32,
    ) -> Result<OrganizationalUnitDto, ServiceError> {
        self.unit_repo
            .find_by_siorg_code(siorg_code)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Unit with SIORG code {} not found", siorg_code))
            })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list(
        &self,
        organization_id: Option<Uuid>,
        parent_id: Option<Uuid>,
        category_id: Option<Uuid>,
        unit_type_id: Option<Uuid>,
        activity_area: Option<ActivityArea>,
        internal_type: Option<InternalUnitType>,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitWithDetailsDto>, i64), ServiceError> {
        Ok(self
            .unit_repo
            .list(
                organization_id,
                parent_id,
                category_id,
                unit_type_id,
                activity_area,
                internal_type,
                is_active,
                is_siorg_managed,
                search,
                limit,
                offset,
            )
            .await?)
    }

    pub async fn get_tree(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<OrganizationalUnitTreeNode>, ServiceError> {
        Ok(self.unit_repo.get_tree(organization_id).await?)
    }

    pub async fn get_children(
        &self,
        parent_id: Uuid,
    ) -> Result<Vec<OrganizationalUnitDto>, ServiceError> {
        Ok(self.unit_repo.get_children(parent_id).await?)
    }

    pub async fn get_path_to_root(
        &self,
        id: Uuid,
    ) -> Result<Vec<OrganizationalUnitDto>, ServiceError> {
        Ok(self.unit_repo.get_path_to_root(id).await?)
    }

    pub async fn create(
        &self,
        payload: CreateOrganizationalUnitPayload,
    ) -> Result<OrganizationalUnitDto, ServiceError> {
        // Validate organization exists
        self.org_repo
            .find_by_id(payload.organization_id)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Organization {} not found", payload.organization_id))
            })?;

        // Validate parent exists (if provided)
        if let Some(parent_id) = payload.parent_id {
            self.unit_repo
                .find_by_id(parent_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Parent unit {} not found", parent_id)))?;
        }

        // Validate category exists
        self.category_repo
            .find_by_id(payload.category_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Category {} not found", payload.category_id)))?;

        // Validate unit type exists
        self.type_repo
            .find_by_id(payload.unit_type_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Unit type {} not found", payload.unit_type_id)))?;

        // Check if SIORG code already exists (if provided)
        if let Some(siorg_code) = payload.siorg_code {
            if self.unit_repo.find_by_siorg_code(siorg_code).await?.is_some() {
                return Err(ServiceError::Conflict(format!(
                    "SIORG code {} already exists",
                    siorg_code
                )));
            }
        }

        // Check if allow_custom_units is enabled (if no SIORG code)
        if payload.siorg_code.is_none() {
            let allow_custom: bool = self
                .settings_repo
                .get_value("units.allow_custom_units")
                .await?
                .unwrap_or(true);

            if !allow_custom {
                return Err(ServiceError::BadRequest(
                    "Creating units without SIORG code is disabled. Set 'units.allow_custom_units' to true."
                        .to_string(),
                ));
            }
        }

        Ok(self.unit_repo.create(payload).await?)
    }

    pub async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationalUnitPayload,
    ) -> Result<OrganizationalUnitDto, ServiceError> {
        // Ensure unit exists
        let _ = self.get(id).await?;

        // Validate parent exists (if being updated)
        if let Some(parent_id) = payload.parent_id {
            // Cannot set self as parent
            if parent_id == id {
                return Err(ServiceError::BadRequest(
                    "Cannot set unit as its own parent".to_string(),
                ));
            }

            self.unit_repo
                .find_by_id(parent_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Parent unit {} not found", parent_id)))?;
        }

        // Validate category exists (if being updated)
        if let Some(category_id) = payload.category_id {
            self.category_repo
                .find_by_id(category_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Category {} not found", category_id)))?;
        }

        // Validate unit type exists (if being updated)
        if let Some(unit_type_id) = payload.unit_type_id {
            self.type_repo
                .find_by_id(unit_type_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Unit type {} not found", unit_type_id)))?;
        }

        Ok(self.unit_repo.update(id, payload).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        // Check if has children
        if self.unit_repo.has_children(id).await? {
            return Err(ServiceError::Conflict(
                "Cannot delete unit with children. Delete or reassign children first.".to_string(),
            ));
        }

        Ok(self.unit_repo.delete(id).await?)
    }

    pub async fn deactivate(&self, id: Uuid, reason: Option<String>) -> Result<(), ServiceError> {
        // Ensure unit exists
        let _ = self.get(id).await?;

        Ok(self.unit_repo.deactivate(id, reason).await?)
    }

    pub async fn activate(&self, id: Uuid) -> Result<(), ServiceError> {
        // Ensure unit exists
        let _ = self.get(id).await?;

        Ok(self.unit_repo.activate(id).await?)
    }
}
