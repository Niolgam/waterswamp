use domain::{
    errors::RepositoryError,
    models::catalog::*,
    ports::catalog::*,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// ============================
// Unit of Measure Repository
// ============================

pub struct UnitOfMeasureRepository {
    pool: PgPool,
}

impl UnitOfMeasureRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl UnitOfMeasureRepositoryPort for UnitOfMeasureRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOfMeasureDto>, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>(
            "SELECT * FROM units_of_measure WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_symbol(&self, symbol: &str) -> Result<Option<UnitOfMeasureDto>, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>(
            "SELECT * FROM units_of_measure WHERE symbol = $1"
        )
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_symbol(&self, symbol: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM units_of_measure WHERE symbol = $1"
        )
        .bind(symbol)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_symbol_excluding(&self, symbol: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM units_of_measure WHERE symbol = $1 AND id != $2"
        )
        .bind(symbol)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        name: &str,
        symbol: &str,
        description: Option<&str>,
        is_base_unit: bool,
    ) -> Result<UnitOfMeasureDto, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>(
            r#"
            INSERT INTO units_of_measure (name, symbol, description, is_base_unit)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(name)
        .bind(symbol)
        .bind(description)
        .bind(is_base_unit)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        symbol: Option<&str>,
        description: Option<&str>,
        is_base_unit: Option<bool>,
    ) -> Result<UnitOfMeasureDto, RepositoryError> {
        let current = self.find_by_id(id).await?
            .ok_or(RepositoryError::NotFound)?;

        sqlx::query_as::<_, UnitOfMeasureDto>(
            r#"
            UPDATE units_of_measure
            SET name = COALESCE($2, name),
                symbol = COALESCE($3, symbol),
                description = CASE WHEN $4::TEXT IS NOT NULL THEN $4 ELSE description END,
                is_base_unit = COALESCE($5, is_base_unit),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(symbol)
        .bind(description)
        .bind(is_base_unit)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM units_of_measure WHERE id = $1")
        .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<UnitOfMeasureDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let units = sqlx::query_as::<_, UnitOfMeasureDto>(
            r#"
            SELECT * FROM units_of_measure
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR symbol ILIKE $1)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM units_of_measure
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR symbol ILIKE $1)
            "#
        )
        .bind(search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok((units, total))
    }
}

// ============================
// Catalog Group Repository
// ============================

pub struct CatalogGroupRepository {
    pool: PgPool,
}

impl CatalogGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl CatalogGroupRepositoryPort for CatalogGroupRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatalogGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatalogGroupDto>(
            r#"SELECT id, parent_id, name, code, item_type as "item_type: ItemType", 
               budget_classification_id, is_active, created_at, updated_at
               FROM catalog_groups WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatalogGroupWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"
            SELECT 
                cg.id, cg.parent_id, cg.name, cg.code,
                cg.item_type as "item_type: ItemType",
                cg.budget_classification_id, cg.is_active,
                cg.created_at, cg.updated_at,
                bc.name as budget_classification_name,
                bc.full_code as budget_classification_code,
                pg.name as parent_name
            FROM catalog_groups cg
            JOIN budget_classifications bc ON cg.budget_classification_id = bc.id
            LEFT JOIN catalog_groups pg ON cg.parent_id = pg.id
            WHERE cg.id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(result.map(|r| CatalogGroupWithDetailsDto {
            id: r.get("id"),
            parent_id: r.get("parent_id"),
            name: r.get("name"),
            code: r.get("code"),
            item_type: r.get("item_type"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.get("budget_classification_name"),
            budget_classification_code: r.get("budget_classification_code"),
            parent_name: r.get("parent_name"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CatalogGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatalogGroupDto>(
            r#"SELECT id, parent_id, name, code, item_type as "item_type: ItemType",
               budget_classification_id, is_active, created_at, updated_at
               FROM catalog_groups WHERE code = $1"#
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_code_in_level(&self, code: &str, parent_id: Option<Uuid>) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_groups WHERE code = $1 AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL))"
        )
        .bind(code)
        .bind(parent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_code_in_level_excluding(
        &self,
        code: &str,
        parent_id: Option<Uuid>,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_groups WHERE code = $1 AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL)) AND id != $3"
        )
        .bind(code)
        .bind(parent_id)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn has_children(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_groups WHERE parent_id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_items WHERE group_id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn get_item_count(&self, id: Uuid) -> Result<i64, RepositoryError> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_items WHERE group_id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn create(
        &self,
        parent_id: Option<Uuid>,
        name: &str,
        code: &str,
        item_type: ItemType,
        budget_classification_id: Uuid,
        is_active: bool,
    ) -> Result<CatalogGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatalogGroupDto>(
            r#"
            INSERT INTO catalog_groups (parent_id, name, code, item_type, budget_classification_id, is_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, parent_id, name, code, item_type as "item_type: ItemType",
                      budget_classification_id, is_active, created_at, updated_at
            "#
        )
        .bind(parent_id)
        .bind(name)
        .bind(code)
        .bind(item_type as ItemType)
        .bind(budget_classification_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        parent_id: Option<Option<Uuid>>,
        name: Option<&str>,
        code: Option<&str>,
        item_type: Option<ItemType>,
        budget_classification_id: Option<Uuid>,
        is_active: Option<bool>,
    ) -> Result<CatalogGroupDto, RepositoryError> {
        let current = self.find_by_id(id).await?
            .ok_or(RepositoryError::NotFound)?;

        let new_parent_id = parent_id.unwrap_or(current.parent_id);
        let new_name = name.unwrap_or(&current.name);
        let new_code = code.unwrap_or(&current.code);
        let new_item_type = item_type.unwrap_or(current.item_type);
        let new_budget_classification_id = budget_classification_id.unwrap_or(current.budget_classification_id);
        let new_is_active = is_active.unwrap_or(current.is_active);

        sqlx::query_as::<_, CatalogGroupDto>(
            r#"
            UPDATE catalog_groups
            SET parent_id = $2, name = $3, code = $4, item_type = $5,
                budget_classification_id = $6, is_active = $7, updated_at = NOW()
            WHERE id = $1
            RETURNING id, parent_id, name, code, item_type as "item_type: ItemType",
                      budget_classification_id, is_active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(new_parent_id)
        .bind(new_name)
        .bind(new_code)
        .bind(new_item_type as ItemType)
        .bind(new_budget_classification_id)
        .bind(new_is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catalog_groups WHERE id = $1")
        .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        parent_id: Option<Uuid>,
        item_type: Option<ItemType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatalogGroupWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let records = sqlx::query(
            r#"
            SELECT 
                cg.id, cg.parent_id, cg.name, cg.code,
                cg.item_type as "item_type: ItemType",
                cg.budget_classification_id, cg.is_active,
                cg.created_at, cg.updated_at,
                bc.name as budget_classification_name,
                bc.full_code as budget_classification_code,
                pg.name as parent_name
            FROM catalog_groups cg
            JOIN budget_classifications bc ON cg.budget_classification_id = bc.id
            LEFT JOIN catalog_groups pg ON cg.parent_id = pg.id
            WHERE ($1::TEXT IS NULL OR cg.name ILIKE $1 OR cg.code ILIKE $1)
              AND ($2::UUID IS NULL OR cg.parent_id = $2 OR (cg.parent_id IS NULL AND $2 IS NULL))
              AND ($3::item_type_enum IS NULL OR cg.item_type = $3)
              AND ($4::BOOLEAN IS NULL OR cg.is_active = $4)
            ORDER BY cg.name
            LIMIT $5 OFFSET $6
            "#
        )
        .bind(search_pattern)
        .bind(parent_id)
        .bind(item_type as Option<ItemType>)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        let groups = records.into_iter().map(|r| CatalogGroupWithDetailsDto {
            id: r.id,
            parent_id: r.parent_id,
            name: r.name,
            code: r.code,
            item_type: r.item_type,
            budget_classification_id: r.budget_classification_id,
            budget_classification_name: r.budget_classification_name,
            budget_classification_code: r.budget_classification_code,
            parent_name: r.parent_name,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM catalog_groups cg
            WHERE ($1::TEXT IS NULL OR cg.name ILIKE $1 OR cg.code ILIKE $1)
              AND ($2::UUID IS NULL OR cg.parent_id = $2 OR (cg.parent_id IS NULL AND $2 IS NULL))
              AND ($3::item_type_enum IS NULL OR cg.item_type = $3)
              AND ($4::BOOLEAN IS NULL OR cg.is_active = $4)
            "#
        )
        .bind(search_pattern)
        .bind(parent_id)
        .bind(item_type as Option<ItemType>)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok((groups, total))
    }

    async fn find_children(&self, parent_id: Option<Uuid>) -> Result<Vec<CatalogGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatalogGroupDto>(
            r#"SELECT id, parent_id, name, code, item_type as "item_type: ItemType",
               budget_classification_id, is_active, created_at, updated_at
               FROM catalog_groups 
               WHERE parent_id = $1 OR (parent_id IS NULL AND $1 IS NULL)
               ORDER BY name"#
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn get_tree(&self) -> Result<Vec<CatalogGroupTreeNode>, RepositoryError> {
        // Fetch all groups with their details
        let all_groups = sqlx::query(
            r#"
            SELECT 
                cg.id, cg.parent_id, cg.name, cg.code,
                cg.item_type as "item_type: ItemType",
                cg.budget_classification_id, cg.is_active,
                cg.created_at, cg.updated_at,
                bc.name as budget_classification_name,
                bc.full_code as budget_classification_code,
                (SELECT COUNT(*) FROM catalog_items WHERE group_id = cg.id) as "item_count!"
            FROM catalog_groups cg
            JOIN budget_classifications bc ON cg.budget_classification_id = bc.id
            ORDER BY cg.name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        // Build hierarchical structure
        let mut nodes: Vec<CatalogGroupTreeNode> = all_groups.iter().map(|r| CatalogGroupTreeNode {
            id: r.id,
            parent_id: r.parent_id,
            name: r.name.clone(),
            code: r.code.clone(),
            item_type: r.item_type.clone(),
            budget_classification_id: r.budget_classification_id,
            budget_classification_name: r.budget_classification_name.clone(),
            budget_classification_code: r.budget_classification_code.clone(),
            is_active: r.is_active,
            children: Vec::new(),
            item_count: r.item_count,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        // Build tree structure (root nodes only)
        let root_nodes: Vec<CatalogGroupTreeNode> = nodes.iter()
            .filter(|n| n.parent_id.is_none())
            .cloned()
            .map(|mut node| {
                Self::build_tree_recursive(&mut node, &nodes);
                node
            })
            .collect();

        Ok(root_nodes)
    }
}

impl CatalogGroupRepository {
    fn build_tree_recursive(node: &mut CatalogGroupTreeNode, all_nodes: &[CatalogGroupTreeNode]) {
        node.children = all_nodes.iter()
            .filter(|n| n.parent_id == Some(node.id))
            .cloned()
            .map(|mut child| {
                Self::build_tree_recursive(&mut child, all_nodes);
                child
            })
            .collect();
    }
}

// ============================
// Catalog Item Repository
// ============================

pub struct CatalogItemRepository {
    pool: PgPool,
}

impl CatalogItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl CatalogItemRepositoryPort for CatalogItemRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatalogItemDto>, RepositoryError> {
        sqlx::query_as::<_, CatalogItemDto>(
            "SELECT * FROM catalog_items WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatalogItemWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"
            SELECT 
                ci.*, 
                cg.name as group_name, cg.code as group_code,
                um.name as unit_name, um.symbol as unit_symbol
            FROM catalog_items ci
            JOIN catalog_groups cg ON ci.group_id = cg.id
            JOIN units_of_measure um ON ci.unit_of_measure_id = um.id
            WHERE ci.id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(result.map(|r| CatalogItemWithDetailsDto {
            id: r.id,
            group_id: r.group_id,
            group_name: r.group_name,
            group_code: r.group_code,
            unit_of_measure_id: r.unit_of_measure_id,
            unit_name: r.unit_name,
            unit_symbol: r.unit_symbol,
            name: r.name,
            catmat_code: r.catmat_code,
            specification: r.specification,
            estimated_value: r.estimated_value,
            search_links: r.search_links,
            photo_url: r.photo_url,
            is_stockable: r.is_stockable,
            is_permanent: r.is_permanent,
            shelf_life_days: r.shelf_life_days,
            requires_batch_control: r.requires_batch_control,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn find_by_catmat_code(&self, catmat_code: &str) -> Result<Option<CatalogItemDto>, RepositoryError> {
        sqlx::query_as::<_, CatalogItemDto>(
            "SELECT * FROM catalog_items WHERE catmat_code = $1"
        )
        .bind(catmat_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_catmat_code(&self, catmat_code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_items WHERE catmat_code = $1"
        )
        .bind(catmat_code)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_catmat_code_excluding(&self, catmat_code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_items WHERE catmat_code = $1 AND id != $2"
        )
        .bind(catmat_code)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_name_in_group(&self, name: &str, group_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_items WHERE name = $1 AND group_id = $2"
        )
        .bind(name)
        .bind(group_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_name_in_group_excluding(&self, name: &str, group_id: Uuid, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM catalog_items WHERE name = $1 AND group_id = $2 AND id != $3"
        )
        .bind(name)
        .bind(group_id)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        group_id: Uuid,
        unit_of_measure_id: Uuid,
        name: &str,
        catmat_code: Option<&str>,
        specification: &str,
        estimated_value: rust_decimal::Decimal,
        search_links: Option<&str>,
        photo_url: Option<&str>,
        is_stockable: bool,
        is_permanent: bool,
        shelf_life_days: Option<i32>,
        requires_batch_control: bool,
        is_active: bool,
    ) -> Result<CatalogItemDto, RepositoryError> {
        sqlx::query_as::<_, CatalogItemDto>(
            r#"
            INSERT INTO catalog_items (
                group_id, unit_of_measure_id, name, catmat_code, specification,
                estimated_value, search_links, photo_url, is_stockable, is_permanent,
                shelf_life_days, requires_batch_control, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(group_id)
        .bind(unit_of_measure_id)
        .bind(name)
        .bind(catmat_code)
        .bind(specification)
        .bind(estimated_value)
        .bind(search_links)
        .bind(photo_url)
        .bind(is_stockable)
        .bind(is_permanent)
        .bind(shelf_life_days)
        .bind(requires_batch_control)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        group_id: Option<Uuid>,
        unit_of_measure_id: Option<Uuid>,
        name: Option<&str>,
        catmat_code: Option<&str>,
        specification: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>,
        search_links: Option<&str>,
        photo_url: Option<&str>,
        is_stockable: Option<bool>,
        is_permanent: Option<bool>,
        shelf_life_days: Option<i32>,
        requires_batch_control: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<CatalogItemDto, RepositoryError> {
        let current = self.find_by_id(id).await?
            .ok_or(RepositoryError::NotFound)?;

        sqlx::query_as::<_, CatalogItemDto>(
            r#"
            UPDATE catalog_items
            SET group_id = COALESCE($2, group_id),
                unit_of_measure_id = COALESCE($3, unit_of_measure_id),
                name = COALESCE($4, name),
                catmat_code = CASE WHEN $5::TEXT IS NOT NULL THEN $5 ELSE catmat_code END,
                specification = COALESCE($6, specification),
                estimated_value = COALESCE($7, estimated_value),
                search_links = CASE WHEN $8::TEXT IS NOT NULL THEN $8 ELSE search_links END,
                photo_url = CASE WHEN $9::TEXT IS NOT NULL THEN $9 ELSE photo_url END,
                is_stockable = COALESCE($10, is_stockable),
                is_permanent = COALESCE($11, is_permanent),
                shelf_life_days = CASE WHEN $12::INTEGER IS NOT NULL THEN $12 ELSE shelf_life_days END,
                requires_batch_control = COALESCE($13, requires_batch_control),
                is_active = COALESCE($14, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(group_id)
        .bind(unit_of_measure_id)
        .bind(name)
        .bind(catmat_code)
        .bind(specification)
        .bind(estimated_value)
        .bind(search_links)
        .bind(photo_url)
        .bind(is_stockable)
        .bind(is_permanent)
        .bind(shelf_life_days)
        .bind(requires_batch_control)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catalog_items WHERE id = $1")
        .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        group_id: Option<Uuid>,
        is_stockable: Option<bool>,
        is_permanent: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(Vec<CatalogItemWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let records = sqlx::query(
            r#"
            SELECT 
                ci.*, 
                cg.name as group_name, cg.code as group_code,
                um.name as unit_name, um.symbol as unit_symbol
            FROM catalog_items ci
            JOIN catalog_groups cg ON ci.group_id = cg.id
            JOIN units_of_measure um ON ci.unit_of_measure_id = um.id
            WHERE ($1::TEXT IS NULL OR ci.name ILIKE $1 OR ci.specification ILIKE $1 OR ci.catmat_code ILIKE $1)
              AND ($2::UUID IS NULL OR ci.group_id = $2)
              AND ($3::BOOLEAN IS NULL OR ci.is_stockable = $3)
              AND ($4::BOOLEAN IS NULL OR ci.is_permanent = $4)
              AND ($5::BOOLEAN IS NULL OR ci.is_active = $5)
            ORDER BY ci.name
            LIMIT $6 OFFSET $7
            "#
        )
        .bind(search_pattern)
        .bind(group_id)
        .bind(is_stockable)
        .bind(is_permanent)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        let items = records.into_iter().map(|r| CatalogItemWithDetailsDto {
            id: r.id,
            group_id: r.group_id,
            group_name: r.group_name,
            group_code: r.group_code,
            unit_of_measure_id: r.unit_of_measure_id,
            unit_name: r.unit_name,
            unit_symbol: r.unit_symbol,
            name: r.name,
            catmat_code: r.catmat_code,
            specification: r.specification,
            estimated_value: r.estimated_value,
            search_links: r.search_links,
            photo_url: r.photo_url,
            is_stockable: r.is_stockable,
            is_permanent: r.is_permanent,
            shelf_life_days: r.shelf_life_days,
            requires_batch_control: r.requires_batch_control,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM catalog_items ci
            WHERE ($1::TEXT IS NULL OR ci.name ILIKE $1 OR ci.specification ILIKE $1 OR ci.catmat_code ILIKE $1)
              AND ($2::UUID IS NULL OR ci.group_id = $2)
              AND ($3::BOOLEAN IS NULL OR ci.is_stockable = $3)
              AND ($4::BOOLEAN IS NULL OR ci.is_permanent = $4)
              AND ($5::BOOLEAN IS NULL OR ci.is_active = $5)
            "#
        )
        .bind(search_pattern)
        .bind(group_id)
        .bind(is_stockable)
        .bind(is_permanent)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok((items, total))
    }
}

// ============================
// Unit Conversion Repository
// ============================

pub struct UnitConversionRepository {
    pool: PgPool,
}

impl UnitConversionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl UnitConversionRepositoryPort for UnitConversionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitConversionDto>, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            "SELECT * FROM unit_conversions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<UnitConversionWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"
            SELECT 
                uc.*,
                from_unit.name as from_unit_name,
                from_unit.symbol as from_unit_symbol,
                to_unit.name as to_unit_name,
                to_unit.symbol as to_unit_symbol
            FROM unit_conversions uc
            JOIN units_of_measure from_unit ON uc.from_unit_id = from_unit.id
            JOIN units_of_measure to_unit ON uc.to_unit_id = to_unit.id
            WHERE uc.id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok(result.map(|r| UnitConversionWithDetailsDto {
            id: r.id,
            from_unit_id: r.from_unit_id,
            from_unit_name: r.from_unit_name,
            from_unit_symbol: r.from_unit_symbol,
            to_unit_id: r.to_unit_id,
            to_unit_name: r.to_unit_name,
            to_unit_symbol: r.to_unit_symbol,
            conversion_factor: r.conversion_factor,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn find_conversion(&self, from_unit_id: Uuid, to_unit_id: Uuid) -> Result<Option<UnitConversionDto>, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            "SELECT * FROM unit_conversions WHERE from_unit_id = $1 AND to_unit_id = $2"
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_conversion(&self, from_unit_id: Uuid, to_unit_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM unit_conversions WHERE from_unit_id = $1 AND to_unit_id = $2"
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        from_unit_id: Uuid,
        to_unit_id: Uuid,
        conversion_factor: rust_decimal::Decimal,
    ) -> Result<UnitConversionDto, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            r#"
            INSERT INTO unit_conversions (from_unit_id, to_unit_id, conversion_factor)
            VALUES ($1, $2, $3)
            RETURNING *
            "#
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .bind(conversion_factor)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        conversion_factor: rust_decimal::Decimal,
    ) -> Result<UnitConversionDto, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            r#"
            UPDATE unit_conversions
            SET conversion_factor = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(conversion_factor)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM unit_conversions WHERE id = $1")
        .bind(id)
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        from_unit_id: Option<Uuid>,
        to_unit_id: Option<Uuid>,
    ) -> Result<(Vec<UnitConversionWithDetailsDto>, i64), RepositoryError> {
        let records = sqlx::query(
            r#"
            SELECT 
                uc.*,
                from_unit.name as from_unit_name,
                from_unit.symbol as from_unit_symbol,
                to_unit.name as to_unit_name,
                to_unit.symbol as to_unit_symbol
            FROM unit_conversions uc
            JOIN units_of_measure from_unit ON uc.from_unit_id = from_unit.id
            JOIN units_of_measure to_unit ON uc.to_unit_id = to_unit.id
            WHERE ($1::UUID IS NULL OR uc.from_unit_id = $1)
              AND ($2::UUID IS NULL OR uc.to_unit_id = $2)
            ORDER BY from_unit.name, to_unit.name
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        let conversions = records.into_iter().map(|r| UnitConversionWithDetailsDto {
            id: r.id,
            from_unit_id: r.from_unit_id,
            from_unit_name: r.from_unit_name,
            from_unit_symbol: r.from_unit_symbol,
            to_unit_id: r.to_unit_id,
            to_unit_name: r.to_unit_name,
            to_unit_symbol: r.to_unit_symbol,
            conversion_factor: r.conversion_factor,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM unit_conversions uc
            WHERE ($1::UUID IS NULL OR uc.from_unit_id = $1)
              AND ($2::UUID IS NULL OR uc.to_unit_id = $2)
            "#
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;

        Ok((conversions, total))
    }
}
