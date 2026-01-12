use domain::models::organizational::*;
use domain::errors::RepositoryError;
use domain::ports::{
    OrganizationRepositoryPort, OrganizationalUnitCategoryRepositoryPort,
    OrganizationalUnitRepositoryPort, OrganizationalUnitTypeRepositoryPort,
    SystemSettingsRepositoryPort,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// System Settings Repository
// ============================================================================

pub struct SystemSettingsRepository {
    pool: Arc<PgPool>,
}

impl SystemSettingsRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SystemSettingsRepositoryPort for SystemSettingsRepository {
    async fn get(&self, key: &str) -> Result<Option<SystemSettingDto>, RepositoryError> {
        let result = sqlx::query_as!(
            SystemSettingDto,
            r#"
            SELECT
                key,
                value,
                value_type,
                description,
                category,
                is_sensitive,
                updated_at,
                updated_by
            FROM system_settings
            WHERE key = $1
            "#,
            key
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn list(
        &self,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SystemSettingDto>, i64), RepositoryError> {
        let settings = sqlx::query_as!(
            SystemSettingDto,
            r#"
            SELECT
                key,
                value,
                value_type,
                description,
                category,
                is_sensitive,
                updated_at,
                updated_by
            FROM system_settings
            WHERE ($1::TEXT IS NULL OR category = $1)
            ORDER BY category, key
            LIMIT $2 OFFSET $3
            "#,
            category,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM system_settings
            WHERE ($1::TEXT IS NULL OR category = $1)
            "#,
            category
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok((settings, total))
    }

    async fn create(
        &self,
        payload: CreateSystemSettingPayload,
    ) -> Result<SystemSettingDto, RepositoryError> {
        let result = sqlx::query_as!(
            SystemSettingDto,
            r#"
            INSERT INTO system_settings (key, value, value_type, description, category, is_sensitive)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING key, value, value_type, description, category, is_sensitive, updated_at, updated_by
            "#,
            payload.key,
            payload.value,
            payload.value_type,
            payload.description,
            payload.category,
            payload.is_sensitive
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update(
        &self,
        key: &str,
        payload: UpdateSystemSettingPayload,
        updated_by: Option<Uuid>,
    ) -> Result<SystemSettingDto, RepositoryError> {
        let result = sqlx::query_as!(
            SystemSettingDto,
            r#"
            UPDATE system_settings
            SET
                value = COALESCE($2, value),
                description = COALESCE($3, description),
                category = COALESCE($4, category),
                is_sensitive = COALESCE($5, is_sensitive),
                updated_by = $6,
                updated_at = NOW()
            WHERE key = $1
            RETURNING key, value, value_type, description, category, is_sensitive, updated_at, updated_by
            "#,
            key,
            payload.value,
            payload.description,
            payload.category,
            payload.is_sensitive,
            updated_by
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, key: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM system_settings WHERE key = $1", key)
            .execute(&*self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

// ============================================================================
// Organization Repository
// ============================================================================

pub struct OrganizationRepository {
    pool: Arc<PgPool>,
}

impl OrganizationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OrganizationRepositoryPort for OrganizationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationDto,
            r#"
            SELECT
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_cnpj(&self, cnpj: &str) -> Result<Option<OrganizationDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationDto,
            r#"
            SELECT
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            FROM organizations
            WHERE cnpj = $1
            "#,
            cnpj
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_siorg_code(
        &self,
        siorg_code: i32,
    ) -> Result<Option<OrganizationDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationDto,
            r#"
            SELECT
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            FROM organizations
            WHERE siorg_code = $1
            "#,
            siorg_code
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_main(&self) -> Result<Option<OrganizationDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationDto,
            r#"
            SELECT
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            FROM organizations
            WHERE is_main = TRUE
            LIMIT 1
            "#
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn list(
        &self,
        is_active: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationDto>, i64), RepositoryError> {
        let organizations = sqlx::query_as!(
            OrganizationDto,
            r#"
            SELECT
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            FROM organizations
            WHERE ($1::BOOLEAN IS NULL OR is_active = $1)
            ORDER BY is_main DESC, name
            LIMIT $2 OFFSET $3
            "#,
            is_active,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM organizations
            WHERE ($1::BOOLEAN IS NULL OR is_active = $1)
            "#,
            is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok((organizations, total))
    }

    async fn create(
        &self,
        payload: CreateOrganizationPayload,
    ) -> Result<OrganizationDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationDto,
            r#"
            INSERT INTO organizations (
                acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            "#,
            payload.acronym,
            payload.name,
            payload.cnpj,
            payload.ug_code,
            payload.siorg_code,
            payload.address,
            payload.city,
            payload.state,
            payload.zip_code,
            payload.phone,
            payload.email,
            payload.website,
            payload.logo_url,
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationPayload,
    ) -> Result<OrganizationDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationDto,
            r#"
            UPDATE organizations
            SET
                acronym = COALESCE($2, acronym),
                name = COALESCE($3, name),
                address = COALESCE($4, address),
                city = COALESCE($5, city),
                state = COALESCE($6, state),
                zip_code = COALESCE($7, zip_code),
                phone = COALESCE($8, phone),
                email = COALESCE($9, email),
                website = COALESCE($10, website),
                logo_url = COALESCE($11, logo_url),
                is_active = COALESCE($12, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, acronym, name, cnpj, ug_code, siorg_code,
                address, city, state, zip_code, phone, email, website, logo_url,
                is_main, is_active, siorg_synced_at, siorg_raw_data,
                created_at, updated_at
            "#,
            id,
            payload.acronym,
            payload.name,
            payload.address,
            payload.city,
            payload.state,
            payload.zip_code,
            payload.phone,
            payload.email,
            payload.website,
            payload.logo_url,
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM organizations WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

// ============================================================================
// Organizational Unit Category Repository
// ============================================================================

pub struct OrganizationalUnitCategoryRepository {
    pool: Arc<PgPool>,
}

impl OrganizationalUnitCategoryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OrganizationalUnitCategoryRepositoryPort for OrganizationalUnitCategoryRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitCategoryDto,
            r#"
            SELECT
                id, name, description, siorg_code, siorg_name, is_siorg_managed,
                display_order, is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_categories
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitCategoryDto,
            r#"
            SELECT
                id, name, description, siorg_code, siorg_name, is_siorg_managed,
                display_order, is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_categories
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_siorg_code(
        &self,
        siorg_code: i32,
    ) -> Result<Option<OrganizationalUnitCategoryDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitCategoryDto,
            r#"
            SELECT
                id, name, description, siorg_code, siorg_name, is_siorg_managed,
                display_order, is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_categories
            WHERE siorg_code = $1
            "#,
            siorg_code
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitCategoryDto>, i64), RepositoryError> {
        let categories = sqlx::query_as!(
            OrganizationalUnitCategoryDto,
            r#"
            SELECT
                id, name, description, siorg_code, siorg_name, is_siorg_managed,
                display_order, is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_categories
            WHERE ($1::BOOLEAN IS NULL OR is_active = $1)
              AND ($2::BOOLEAN IS NULL OR is_siorg_managed = $2)
            ORDER BY display_order, name
            LIMIT $3 OFFSET $4
            "#,
            is_active,
            is_siorg_managed,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM organizational_unit_categories
            WHERE ($1::BOOLEAN IS NULL OR is_active = $1)
              AND ($2::BOOLEAN IS NULL OR is_siorg_managed = $2)
            "#,
            is_active,
            is_siorg_managed
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok((categories, total))
    }

    async fn create(
        &self,
        payload: CreateOrganizationalUnitCategoryPayload,
    ) -> Result<OrganizationalUnitCategoryDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitCategoryDto,
            r#"
            INSERT INTO organizational_unit_categories (
                name, description, siorg_code, display_order, is_active
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id, name, description, siorg_code, siorg_name, is_siorg_managed,
                display_order, is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            "#,
            payload.name,
            payload.description,
            payload.siorg_code,
            payload.display_order,
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationalUnitCategoryPayload,
    ) -> Result<OrganizationalUnitCategoryDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitCategoryDto,
            r#"
            UPDATE organizational_unit_categories
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                display_order = COALESCE($4, display_order),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, name, description, siorg_code, siorg_name, is_siorg_managed,
                display_order, is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            "#,
            id,
            payload.name,
            payload.description,
            payload.display_order,
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM organizational_unit_categories WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

// ============================================================================
// Organizational Unit Type Repository
// ============================================================================

pub struct OrganizationalUnitTypeRepository {
    pool: Arc<PgPool>,
}

impl OrganizationalUnitTypeRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OrganizationalUnitTypeRepositoryPort for OrganizationalUnitTypeRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitTypeDto,
            r#"
            SELECT
                id, code, name, description, siorg_code, siorg_name, is_siorg_managed,
                is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_types
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_code(
        &self,
        code: &str,
    ) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitTypeDto,
            r#"
            SELECT
                id, code, name, description, siorg_code, siorg_name, is_siorg_managed,
                is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_types
            WHERE code = $1
            "#,
            code
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_siorg_code(
        &self,
        siorg_code: i32,
    ) -> Result<Option<OrganizationalUnitTypeDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitTypeDto,
            r#"
            SELECT
                id, code, name, description, siorg_code, siorg_name, is_siorg_managed,
                is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_types
            WHERE siorg_code = $1
            "#,
            siorg_code
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn list(
        &self,
        is_active: Option<bool>,
        is_siorg_managed: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<OrganizationalUnitTypeDto>, i64), RepositoryError> {
        let types = sqlx::query_as!(
            OrganizationalUnitTypeDto,
            r#"
            SELECT
                id, code, name, description, siorg_code, siorg_name, is_siorg_managed,
                is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_unit_types
            WHERE ($1::BOOLEAN IS NULL OR is_active = $1)
              AND ($2::BOOLEAN IS NULL OR is_siorg_managed = $2)
            ORDER BY name
            LIMIT $3 OFFSET $4
            "#,
            is_active,
            is_siorg_managed,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM organizational_unit_types
            WHERE ($1::BOOLEAN IS NULL OR is_active = $1)
              AND ($2::BOOLEAN IS NULL OR is_siorg_managed = $2)
            "#,
            is_active,
            is_siorg_managed
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok((types, total))
    }

    async fn create(
        &self,
        payload: CreateOrganizationalUnitTypePayload,
    ) -> Result<OrganizationalUnitTypeDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitTypeDto,
            r#"
            INSERT INTO organizational_unit_types (
                code, name, description, siorg_code, is_active
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id, code, name, description, siorg_code, siorg_name, is_siorg_managed,
                is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            "#,
            payload.code,
            payload.name,
            payload.description,
            payload.siorg_code,
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationalUnitTypePayload,
    ) -> Result<OrganizationalUnitTypeDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitTypeDto,
            r#"
            UPDATE organizational_unit_types
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                is_active = COALESCE($4, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, code, name, description, siorg_code, siorg_name, is_siorg_managed,
                is_active, siorg_synced_at,
                siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            "#,
            id,
            payload.name,
            payload.description,
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM organizational_unit_types WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

// ============================================================================
// Organizational Unit Repository
// ============================================================================

pub struct OrganizationalUnitRepository {
    pool: Arc<PgPool>,
}

impl OrganizationalUnitRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn build_tree_recursive(
        node: &mut OrganizationalUnitTreeNode,
        all_nodes: &[OrganizationalUnitTreeNode],
    ) {
        node.children = all_nodes
            .iter()
            .filter(|n| n.unit.parent_id == Some(node.unit.id))
            .cloned()
            .map(|mut child| {
                Self::build_tree_recursive(&mut child, all_nodes);
                child
            })
            .collect();
    }
}

#[async_trait::async_trait]
impl OrganizationalUnitRepositoryPort for OrganizationalUnitRepository {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationalUnitDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitDto,
            r#"
            SELECT
                id, organization_id, parent_id, category_id, unit_type_id,
                internal_type as "internal_type: InternalUnitType",
                name, formal_name, acronym,
                siorg_code, siorg_parent_code, siorg_url, siorg_last_version, is_siorg_managed,
                activity_area as "activity_area: ActivityArea",
                contact_info as "contact_info: ContactInfo",
                level, path_ids, path_names, is_active, deactivated_at, deactivation_reason,
                siorg_synced_at, siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_units
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_id_with_details(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationalUnitWithDetailsDto>, RepositoryError> {
        let result = sqlx::query!(
            r#"
            SELECT
                ou.id, ou.organization_id, ou.parent_id, ou.category_id, ou.unit_type_id,
                ou.internal_type as "internal_type: InternalUnitType",
                ou.name, ou.formal_name, ou.acronym,
                ou.siorg_code, ou.siorg_parent_code, ou.siorg_url, ou.siorg_last_version, ou.is_siorg_managed,
                ou.activity_area as "activity_area: ActivityArea",
                ou.contact_info as "contact_info: ContactInfo",
                ou.level, ou.path_ids, ou.path_names, ou.is_active, ou.deactivated_at, ou.deactivation_reason,
                ou.siorg_synced_at, ou.siorg_sync_status as "siorg_sync_status: SyncStatus",
                ou.siorg_raw_data, ou.created_at, ou.updated_at,
                org.name as organization_name, org.acronym as organization_acronym,
                p.name as parent_name, p.acronym as parent_acronym,
                cat.name as category_name, ut.name as unit_type_name
            FROM organizational_units ou
            INNER JOIN organizations org ON ou.organization_id = org.id
            LEFT JOIN organizational_units p ON ou.parent_id = p.id
            INNER JOIN organizational_unit_categories cat ON ou.category_id = cat.id
            INNER JOIN organizational_unit_types ut ON ou.unit_type_id = ut.id
            WHERE ou.id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.map(|r| OrganizationalUnitWithDetailsDto {
            unit: OrganizationalUnitDto {
                id: r.id,
                organization_id: r.organization_id,
                parent_id: r.parent_id,
                category_id: r.category_id,
                unit_type_id: r.unit_type_id,
                internal_type: r.internal_type,
                name: r.name,
                formal_name: r.formal_name,
                acronym: r.acronym,
                siorg_code: r.siorg_code,
                siorg_parent_code: r.siorg_parent_code,
                siorg_url: r.siorg_url,
                siorg_last_version: r.siorg_last_version,
                is_siorg_managed: r.is_siorg_managed,
                activity_area: r.activity_area,
                contact_info: r.contact_info,
                level: r.level,
                path_ids: r.path_ids,
                path_names: r.path_names,
                is_active: r.is_active,
                deactivated_at: r.deactivated_at,
                deactivation_reason: r.deactivation_reason,
                siorg_synced_at: r.siorg_synced_at,
                siorg_sync_status: r.siorg_sync_status,
                siorg_raw_data: r.siorg_raw_data,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            organization_name: r.organization_name,
            organization_acronym: r.organization_acronym,
            parent_name: r.parent_name,
            parent_acronym: r.parent_acronym,
            category_name: r.category_name,
            unit_type_name: r.unit_type_name,
        }))
    }

    async fn find_by_siorg_code(
        &self,
        siorg_code: i32,
    ) -> Result<Option<OrganizationalUnitDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitDto,
            r#"
            SELECT
                id, organization_id, parent_id, category_id, unit_type_id,
                internal_type as "internal_type: InternalUnitType",
                name, formal_name, acronym,
                siorg_code, siorg_parent_code, siorg_url, siorg_last_version, is_siorg_managed,
                activity_area as "activity_area: ActivityArea",
                contact_info as "contact_info: ContactInfo",
                level, path_ids, path_names, is_active, deactivated_at, deactivation_reason,
                siorg_synced_at, siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_units
            WHERE siorg_code = $1
            "#,
            siorg_code
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn list(
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
    ) -> Result<(Vec<OrganizationalUnitWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let rows = sqlx::query!(
            r#"
            SELECT
                ou.id, ou.organization_id, ou.parent_id, ou.category_id, ou.unit_type_id,
                ou.internal_type as "internal_type: InternalUnitType",
                ou.name, ou.formal_name, ou.acronym,
                ou.siorg_code, ou.siorg_parent_code, ou.siorg_url, ou.siorg_last_version, ou.is_siorg_managed,
                ou.activity_area as "activity_area: ActivityArea",
                ou.contact_info as "contact_info: ContactInfo",
                ou.level, ou.path_ids, ou.path_names, ou.is_active, ou.deactivated_at, ou.deactivation_reason,
                ou.siorg_synced_at, ou.siorg_sync_status as "siorg_sync_status: SyncStatus",
                ou.siorg_raw_data, ou.created_at, ou.updated_at,
                org.name as organization_name, org.acronym as organization_acronym,
                p.name as parent_name, p.acronym as parent_acronym,
                cat.name as category_name, ut.name as unit_type_name
            FROM organizational_units ou
            INNER JOIN organizations org ON ou.organization_id = org.id
            LEFT JOIN organizational_units p ON ou.parent_id = p.id
            INNER JOIN organizational_unit_categories cat ON ou.category_id = cat.id
            INNER JOIN organizational_unit_types ut ON ou.unit_type_id = ut.id
            WHERE ($1::UUID IS NULL OR ou.organization_id = $1)
              AND ($2::UUID IS NULL OR ou.parent_id = $2)
              AND ($3::UUID IS NULL OR ou.category_id = $3)
              AND ($4::UUID IS NULL OR ou.unit_type_id = $4)
              AND ($5::activity_area_enum IS NULL OR ou.activity_area = $5)
              AND ($6::internal_unit_type_enum IS NULL OR ou.internal_type = $6)
              AND ($7::BOOLEAN IS NULL OR ou.is_active = $7)
              AND ($8::BOOLEAN IS NULL OR ou.is_siorg_managed = $8)
              AND ($9::TEXT IS NULL OR ou.name ILIKE $9 OR ou.acronym ILIKE $9 OR ou.formal_name ILIKE $9)
            ORDER BY ou.level, ou.name
            LIMIT $10 OFFSET $11
            "#,
            organization_id,
            parent_id,
            category_id,
            unit_type_id,
            activity_area as Option<ActivityArea>,
            internal_type as Option<InternalUnitType>,
            is_active,
            is_siorg_managed,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT as "count!"
            FROM organizational_units ou
            WHERE ($1::UUID IS NULL OR ou.organization_id = $1)
              AND ($2::UUID IS NULL OR ou.parent_id = $2)
              AND ($3::UUID IS NULL OR ou.category_id = $3)
              AND ($4::UUID IS NULL OR ou.unit_type_id = $4)
              AND ($5::activity_area_enum IS NULL OR ou.activity_area = $5)
              AND ($6::internal_unit_type_enum IS NULL OR ou.internal_type = $6)
              AND ($7::BOOLEAN IS NULL OR ou.is_active = $7)
              AND ($8::BOOLEAN IS NULL OR ou.is_siorg_managed = $8)
              AND ($9::TEXT IS NULL OR ou.name ILIKE $9 OR ou.acronym ILIKE $9 OR ou.formal_name ILIKE $9)
            "#,
            organization_id,
            parent_id,
            category_id,
            unit_type_id,
            activity_area as Option<ActivityArea>,
            internal_type as Option<InternalUnitType>,
            is_active,
            is_siorg_managed,
            search_pattern
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let units = rows
            .into_iter()
            .map(|r| OrganizationalUnitWithDetailsDto {
                unit: OrganizationalUnitDto {
                    id: r.id,
                    organization_id: r.organization_id,
                    parent_id: r.parent_id,
                    category_id: r.category_id,
                    unit_type_id: r.unit_type_id,
                    internal_type: r.internal_type,
                    name: r.name,
                    formal_name: r.formal_name,
                    acronym: r.acronym,
                    siorg_code: r.siorg_code,
                    siorg_parent_code: r.siorg_parent_code,
                    siorg_url: r.siorg_url,
                    siorg_last_version: r.siorg_last_version,
                    is_siorg_managed: r.is_siorg_managed,
                    activity_area: r.activity_area,
                    contact_info: r.contact_info,
                    level: r.level,
                    path_ids: r.path_ids,
                    path_names: r.path_names,
                    is_active: r.is_active,
                    deactivated_at: r.deactivated_at,
                    deactivation_reason: r.deactivation_reason,
                    siorg_synced_at: r.siorg_synced_at,
                    siorg_sync_status: r.siorg_sync_status,
                    siorg_raw_data: r.siorg_raw_data,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                },
                organization_name: r.organization_name,
                organization_acronym: r.organization_acronym,
                parent_name: r.parent_name,
                parent_acronym: r.parent_acronym,
                category_name: r.category_name,
                unit_type_name: r.unit_type_name,
            })
            .collect();

        Ok((units, total))
    }

    async fn get_tree(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<OrganizationalUnitTreeNode>, RepositoryError> {
        let all_units = sqlx::query_as!(
            OrganizationalUnitDto,
            r#"
            SELECT
                id, organization_id, parent_id, category_id, unit_type_id,
                internal_type as "internal_type: InternalUnitType",
                name, formal_name, acronym,
                siorg_code, siorg_parent_code, siorg_url, siorg_last_version, is_siorg_managed,
                activity_area as "activity_area: ActivityArea",
                contact_info as "contact_info: ContactInfo",
                level, path_ids, path_names, is_active, deactivated_at, deactivation_reason,
                siorg_synced_at, siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_units
            WHERE ($1::UUID IS NULL OR organization_id = $1)
            ORDER BY level, name
            "#,
            organization_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut nodes: Vec<OrganizationalUnitTreeNode> = all_units
            .into_iter()
            .map(|unit| OrganizationalUnitTreeNode {
                unit,
                children: Vec::new(),
                child_count: 0,
            })
            .collect();

        // Build tree recursively
        let root_nodes: Vec<OrganizationalUnitTreeNode> = nodes
            .iter()
            .filter(|n| n.unit.parent_id.is_none())
            .cloned()
            .map(|mut node| {
                Self::build_tree_recursive(&mut node, &nodes);
                node.child_count = node.children.len() as i64;
                node
            })
            .collect();

        Ok(root_nodes)
    }

    async fn get_children(
        &self,
        parent_id: Uuid,
    ) -> Result<Vec<OrganizationalUnitDto>, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitDto,
            r#"
            SELECT
                id, organization_id, parent_id, category_id, unit_type_id,
                internal_type as "internal_type: InternalUnitType",
                name, formal_name, acronym,
                siorg_code, siorg_parent_code, siorg_url, siorg_last_version, is_siorg_managed,
                activity_area as "activity_area: ActivityArea",
                contact_info as "contact_info: ContactInfo",
                level, path_ids, path_names, is_active, deactivated_at, deactivation_reason,
                siorg_synced_at, siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            FROM organizational_units
            WHERE parent_id = $1
            ORDER BY name
            "#,
            parent_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn has_children(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*)::BIGINT FROM organizational_units WHERE parent_id = $1",
            id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(count.unwrap_or(0) > 0)
    }

    async fn get_path_to_root(
        &self,
        id: Uuid,
    ) -> Result<Vec<OrganizationalUnitDto>, RepositoryError> {
        let unit = self.find_by_id(id).await?;

        let unit = match unit {
            Some(u) => u,
            None => return Err(RepositoryError::NotFound),
        };

        let mut path = Vec::new();
        for path_id in unit.path_ids.iter() {
            if let Some(u) = self.find_by_id(*path_id).await? {
                path.push(u);
            }
        }

        Ok(path)
    }

    async fn create(
        &self,
        payload: CreateOrganizationalUnitPayload,
    ) -> Result<OrganizationalUnitDto, RepositoryError> {
        let result = sqlx::query_as!(
            OrganizationalUnitDto,
            r#"
            INSERT INTO organizational_units (
                organization_id, parent_id, category_id, unit_type_id, internal_type,
                name, formal_name, acronym, siorg_code, activity_area, contact_info, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, organization_id, parent_id, category_id, unit_type_id,
                internal_type as "internal_type: InternalUnitType",
                name, formal_name, acronym,
                siorg_code, siorg_parent_code, siorg_url, siorg_last_version, is_siorg_managed,
                activity_area as "activity_area: ActivityArea",
                contact_info as "contact_info: ContactInfo",
                level, path_ids, path_names, is_active, deactivated_at, deactivation_reason,
                siorg_synced_at, siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            "#,
            payload.organization_id,
            payload.parent_id,
            payload.category_id,
            payload.unit_type_id,
            payload.internal_type as InternalUnitType,
            payload.name,
            payload.formal_name,
            payload.acronym,
            payload.siorg_code,
            payload.activity_area as ActivityArea,
            serde_json::to_value(&payload.contact_info).unwrap(),
            payload.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update(
        &self,
        id: Uuid,
        payload: UpdateOrganizationalUnitPayload,
    ) -> Result<OrganizationalUnitDto, RepositoryError> {
        let contact_info_json = payload.contact_info.as_ref().map(|c| serde_json::to_value(c).unwrap());

        let result = sqlx::query_as!(
            OrganizationalUnitDto,
            r#"
            UPDATE organizational_units
            SET
                parent_id = COALESCE($2, parent_id),
                category_id = COALESCE($3, category_id),
                unit_type_id = COALESCE($4, unit_type_id),
                internal_type = COALESCE($5, internal_type),
                name = COALESCE($6, name),
                formal_name = COALESCE($7, formal_name),
                acronym = COALESCE($8, acronym),
                activity_area = COALESCE($9, activity_area),
                contact_info = COALESCE($10, contact_info),
                is_active = COALESCE($11, is_active),
                deactivation_reason = COALESCE($12, deactivation_reason),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, organization_id, parent_id, category_id, unit_type_id,
                internal_type as "internal_type: InternalUnitType",
                name, formal_name, acronym,
                siorg_code, siorg_parent_code, siorg_url, siorg_last_version, is_siorg_managed,
                activity_area as "activity_area: ActivityArea",
                contact_info as "contact_info: ContactInfo",
                level, path_ids, path_names, is_active, deactivated_at, deactivation_reason,
                siorg_synced_at, siorg_sync_status as "siorg_sync_status: SyncStatus",
                siorg_raw_data, created_at, updated_at
            "#,
            id,
            payload.parent_id,
            payload.category_id,
            payload.unit_type_id,
            payload.internal_type as Option<InternalUnitType>,
            payload.name,
            payload.formal_name,
            payload.acronym,
            payload.activity_area as Option<ActivityArea>,
            contact_info_json,
            payload.is_active,
            payload.deactivation_reason
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM organizational_units WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn deactivate(&self, id: Uuid, reason: Option<String>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE organizational_units
            SET is_active = FALSE, deactivated_at = NOW(), deactivation_reason = $2
            WHERE id = $1
            "#,
            id,
            reason
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn activate(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE organizational_units
            SET is_active = TRUE, deactivated_at = NULL, deactivation_reason = NULL
            WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
