use super::siorg_client::{SiorgClient, SiorgOrganization, SiorgUnit};
use domain::models::organizational::*;
use domain::ports::*;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

// ============================================================================
// SIORG Sync Service
// ============================================================================

pub struct SiorgSyncService {
    siorg_client: Arc<SiorgClient>,
    organization_repo: Arc<dyn OrganizationRepositoryPort>,
    unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
    category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
    type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
}

impl SiorgSyncService {
    pub fn new(
        siorg_client: Arc<SiorgClient>,
        organization_repo: Arc<dyn OrganizationRepositoryPort>,
        unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
        category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
        type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
    ) -> Self {
        Self {
            siorg_client,
            organization_repo,
            unit_repo,
            category_repo,
            type_repo,
        }
    }

    // ========================================================================
    // Organization Sync
    // ========================================================================

    /// Synchronize a single organization from SIORG
    pub async fn sync_organization(
        &self,
        siorg_code: i32,
    ) -> Result<OrganizationDto, SyncError> {
        info!("Syncing organization with SIORG code {}", siorg_code);

        // Fetch from SIORG
        let siorg_org = self
            .siorg_client
            .get_organization(siorg_code)
            .await
            .map_err(|e| SyncError::ApiError(e.to_string()))?
            .ok_or_else(|| SyncError::NotFoundInSiorg(siorg_code))?;

        // Check if organization exists locally
        if let Some(local_org) = self
            .organization_repo
            .find_by_siorg_code(siorg_code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            // Update existing
            self.update_organization_from_siorg(local_org.id, &siorg_org)
                .await
        } else {
            // Create new
            self.create_organization_from_siorg(&siorg_org).await
        }
    }

    async fn create_organization_from_siorg(
        &self,
        siorg_org: &SiorgOrganization,
    ) -> Result<OrganizationDto, SyncError> {
        let cnpj = siorg_org
            .cnpj
            .as_ref()
            .ok_or_else(|| SyncError::MissingRequiredField("CNPJ".to_string()))?
            .clone();

        let ug_code = siorg_org
            .codigo_ug
            .ok_or_else(|| SyncError::MissingRequiredField("UG Code".to_string()))?;

        let payload = CreateOrganizationPayload {
            acronym: siorg_org.sigla.clone(),
            name: siorg_org.nome.clone(),
            cnpj,
            ug_code,
            siorg_code: siorg_org.codigo_siorg,
            address: None,
            city: None,
            state: None,
            zip_code: None,
            phone: None,
            email: None,
            website: None,
            logo_url: None,
            is_active: siorg_org.ativo,
        };

        self.organization_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    async fn update_organization_from_siorg(
        &self,
        org_id: Uuid,
        siorg_org: &SiorgOrganization,
    ) -> Result<OrganizationDto, SyncError> {
        let payload = UpdateOrganizationPayload {
            acronym: Some(siorg_org.sigla.clone()),
            name: Some(siorg_org.nome.clone()),
            address: None,
            city: None,
            state: None,
            zip_code: None,
            phone: None,
            email: None,
            website: None,
            logo_url: None,
            is_active: Some(siorg_org.ativo),
        };

        self.organization_repo
            .update(org_id, payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    /// Synchronize an organization using its local database UUID (auto-discovers siorg_code)
    pub async fn sync_organization_by_id(
        &self,
        org_id: Uuid,
    ) -> Result<OrganizationDto, SyncError> {
        let org = self
            .organization_repo
            .find_by_id(org_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Organization {} not found locally", org_id))
            })?;

        info!(
            "Syncing organization {} (siorg_code={}) from local DB lookup",
            org_id, org.siorg_code
        );

        self.sync_organization(org.siorg_code).await
    }

    /// Synchronize all units of an organization using its local database UUID (auto-discovers siorg_code)
    pub async fn sync_organization_units_by_id(
        &self,
        org_id: Uuid,
    ) -> Result<SyncSummary, SyncError> {
        let org = self
            .organization_repo
            .find_by_id(org_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Organization {} not found locally", org_id))
            })?;

        info!(
            "Bulk syncing units for organization {} (siorg_code={}) from local DB lookup",
            org_id, org.siorg_code
        );

        self.sync_organization_units(org.siorg_code).await
    }

    // ========================================================================
    // Unit Sync
    // ========================================================================

    /// Synchronize a single organizational unit from SIORG
    pub async fn sync_unit(&self, siorg_code: i32) -> Result<OrganizationalUnitDto, SyncError> {
        info!("Syncing unit with SIORG code {}", siorg_code);

        // Fetch from SIORG
        let siorg_unit = self
            .siorg_client
            .get_unit(siorg_code)
            .await
            .map_err(|e| SyncError::ApiError(e.to_string()))?
            .ok_or_else(|| SyncError::NotFoundInSiorg(siorg_code))?;

        // Check if unit exists locally
        if let Some(local_unit) = self
            .unit_repo
            .find_by_siorg_code(siorg_code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            // Update existing
            self.update_unit_from_siorg(local_unit.id, &siorg_unit)
                .await
        } else {
            // Create new
            self.create_unit_from_siorg(&siorg_unit).await
        }
    }

    async fn create_unit_from_siorg(
        &self,
        siorg_unit: &SiorgUnit,
    ) -> Result<OrganizationalUnitDto, SyncError> {
        // Find organization (required)
        let organization = self
            .organization_repo
            .find_main()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::MissingRequiredField("Main Organization".to_string()))?;

        // Find or create category
        let category = self
            .find_or_create_category("Default SIORG Category")
            .await?;

        // Find or create type based on tipo_unidade
        let unit_type = self
            .find_or_create_type(&siorg_unit.tipo_unidade)
            .await?;

        // Find parent if exists
        let parent_id = if let Some(parent_siorg_code) = siorg_unit.codigo_unidade_pai {
            self.unit_repo
                .find_by_siorg_code(parent_siorg_code)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?
                .map(|u| u.id)
        } else {
            None
        };

        // Parse activity area
        let activity_area = match siorg_unit.area_atuacao.to_uppercase().as_str() {
            "SUPPORT" | "MEIO" => ActivityArea::Support,
            "CORE" | "FIM" => ActivityArea::Core,
            _ => ActivityArea::Support, // Default
        };

        // Parse internal type
        let internal_type = match siorg_unit.tipo_unidade.to_uppercase().as_str() {
            "ADMINISTRATION" => InternalUnitType::Administration,
            "DEPARTMENT" => InternalUnitType::Department,
            "LABORATORY" => InternalUnitType::Laboratory,
            "COUNCIL" => InternalUnitType::Council,
            "COORDINATION" => InternalUnitType::Coordination,
            "CENTER" => InternalUnitType::Center,
            "DIVISION" => InternalUnitType::Division,
            _ => InternalUnitType::Sector, // Default
        };

        let payload = CreateOrganizationalUnitPayload {
            organization_id: organization.id,
            parent_id,
            category_id: category.id,
            unit_type_id: unit_type.id,
            internal_type,
            name: siorg_unit.nome.clone(),
            formal_name: None,
            acronym: None,
            siorg_code: Some(siorg_unit.codigo_siorg),
            activity_area,
            contact_info: ContactInfo::default(),
            is_active: siorg_unit.ativo,
        };

        self.unit_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    async fn update_unit_from_siorg(
        &self,
        unit_id: Uuid,
        siorg_unit: &SiorgUnit,
    ) -> Result<OrganizationalUnitDto, SyncError> {
        // Find parent if exists and changed
        let parent_id = if let Some(parent_siorg_code) = siorg_unit.codigo_unidade_pai {
            self.unit_repo
                .find_by_siorg_code(parent_siorg_code)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?
                .map(|u| u.id)
        } else {
            None
        };

        let payload = UpdateOrganizationalUnitPayload {
            parent_id,
            category_id: None,
            unit_type_id: None,
            internal_type: None,
            name: Some(siorg_unit.nome.clone()),
            formal_name: None,
            acronym: None,
            activity_area: None,
            contact_info: None,
            is_active: Some(siorg_unit.ativo),
            deactivation_reason: None,
        };

        self.unit_repo
            .update(unit_id, payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    // ========================================================================
    // Bulk Sync
    // ========================================================================

    /// Synchronize all units for an organization
    pub async fn sync_organization_units(
        &self,
        org_siorg_code: i32,
    ) -> Result<SyncSummary, SyncError> {
        info!(
            "Starting bulk sync for organization SIORG code {}",
            org_siorg_code
        );

        let mut summary = SyncSummary {
            total_processed: 0,
            created: 0,
            updated: 0,
            failed: 0,
            errors: Vec::new(),
        };

        let mut page = 1;
        let page_size = 100;

        loop {
            let response = self
                .siorg_client
                .list_units_by_organization(org_siorg_code, page, page_size)
                .await
                .map_err(|e| SyncError::ApiError(e.to_string()))?;

            let data_len = response.data.len();

            for siorg_unit in response.data {
                summary.total_processed += 1;

                match self.sync_unit(siorg_unit.codigo_siorg).await {
                    Ok(unit) => {
                        // Check if was created or updated
                        if unit.created_at == unit.updated_at {
                            summary.created += 1;
                        } else {
                            summary.updated += 1;
                        }
                    }
                    Err(e) => {
                        summary.failed += 1;
                        summary.errors.push(format!(
                            "Unit {}: {}",
                            siorg_unit.codigo_siorg,
                            e.to_string()
                        ));
                        error!("Failed to sync unit {}: {}", siorg_unit.codigo_siorg, e);
                    }
                }
            }

            // Check if there are more pages
            if data_len < page_size as usize {
                break;
            }

            page += 1;
        }

        info!("Bulk sync completed: {:?}", summary);
        Ok(summary)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    async fn find_or_create_category(
        &self,
        name: &str,
    ) -> Result<OrganizationalUnitCategoryDto, SyncError> {
        // Try to find by name
        let (categories, _) = self
            .category_repo
            .list(None, Some(true), 10, 0)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if let Some(category) = categories.into_iter().find(|c| c.name == name) {
            return Ok(category);
        }

        // Create new
        let payload = CreateOrganizationalUnitCategoryPayload {
            name: name.to_string(),
            description: Some("Auto-created from SIORG sync".to_string()),
            siorg_code: None,
            display_order: 0,
            is_active: true,
        };

        self.category_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    async fn find_or_create_type(
        &self,
        code: &str,
    ) -> Result<OrganizationalUnitTypeDto, SyncError> {
        // Try to find by code
        if let Some(unit_type) = self
            .type_repo
            .find_by_code(code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            return Ok(unit_type);
        }

        // Create new
        let payload = CreateOrganizationalUnitTypePayload {
            code: code.to_string(),
            name: code.to_string(),
            description: Some("Auto-created from SIORG sync".to_string()),
            siorg_code: None,
            is_active: true,
        };

        self.type_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    /// Check SIORG API health
    pub async fn check_health(&self) -> Result<bool, SyncError> {
        self.siorg_client
            .health_check()
            .await
            .map_err(|e| SyncError::ApiError(e.to_string()))
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("SIORG API error: {0}")]
    ApiError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Entity not found in SIORG: {0}")]
    NotFoundInSiorg(i32),

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

// ============================================================================
// Sync Summary
// ============================================================================

#[derive(Debug, Clone)]
pub struct SyncSummary {
    pub total_processed: i32,
    pub created: i32,
    pub updated: i32,
    pub failed: i32,
    pub errors: Vec<String>,
}
