use domain::{
    errors::RepositoryError,
    models::catalog::*,
    ports::catalog::*,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

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
}

#[async_trait]
impl UnitOfMeasureRepositoryPort for UnitOfMeasureRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOfMeasureDto>, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>("SELECT * FROM units_of_measure WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_by_symbol(&self, symbol: &str) -> Result<Option<UnitOfMeasureDto>, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>("SELECT * FROM units_of_measure WHERE symbol = $1")
            .bind(symbol)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_symbol(&self, symbol: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM units_of_measure WHERE symbol = $1")
            .bind(symbol)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_symbol_excluding(&self, symbol: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM units_of_measure WHERE symbol = $1 AND id != $2")
            .bind(symbol)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &str, symbol: &str, description: Option<&str>, is_base_unit: bool) -> Result<UnitOfMeasureDto, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>(
            "INSERT INTO units_of_measure (name, symbol, description, is_base_unit) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(name)
        .bind(symbol)
        .bind(description)
        .bind(is_base_unit)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, symbol: Option<&str>, description: Option<&str>, is_base_unit: Option<bool>) -> Result<UnitOfMeasureDto, RepositoryError> {
        sqlx::query_as::<_, UnitOfMeasureDto>(
            r#"UPDATE units_of_measure
            SET name = COALESCE($2, name), symbol = COALESCE($3, symbol),
                description = CASE WHEN $4::TEXT IS NOT NULL THEN $4 ELSE description END,
                is_base_unit = COALESCE($5, is_base_unit), updated_at = NOW()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(symbol)
        .bind(description)
        .bind(is_base_unit)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM units_of_measure WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<UnitOfMeasureDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let units = sqlx::query_as::<_, UnitOfMeasureDto>(
            r#"SELECT * FROM units_of_measure
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR symbol ILIKE $1)
            ORDER BY name LIMIT $2 OFFSET $3"#,
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM units_of_measure WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR symbol ILIKE $1)",
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((units, total))
    }
}

// ============================
// CATMAT Group Repository
// ============================

pub struct CatmatGroupRepository {
    pool: PgPool,
}

impl CatmatGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatmatGroupRepositoryPort for CatmatGroupRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatGroupDto>("SELECT * FROM catmat_groups WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_groups WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_groups WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, code: &str, name: &str, is_active: bool) -> Result<CatmatGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatmatGroupDto>(
            "INSERT INTO catmat_groups (code, name, is_active) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatmatGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatmatGroupDto>(
            r#"UPDATE catmat_groups SET code = COALESCE($2, code), name = COALESCE($3, name),
            is_active = COALESCE($4, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catmat_groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatmatGroupDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let groups = sqlx::query_as::<_, CatmatGroupDto>(
            r#"SELECT * FROM catmat_groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
              AND ($2::BOOLEAN IS NULL OR is_active = $2)
            ORDER BY code LIMIT $3 OFFSET $4"#,
        )
        .bind(&search_pattern)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catmat_groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
              AND ($2::BOOLEAN IS NULL OR is_active = $2)"#,
        )
        .bind(&search_pattern)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((groups, total))
    }

    async fn get_tree(&self) -> Result<Vec<CatmatGroupTreeNode>, RepositoryError> {
        let records = sqlx::query(
            r#"SELECT g.id, g.code, g.name, g.is_active, g.created_at, g.updated_at,
                c.id AS class_id, c.code AS class_code, c.name AS class_name,
                c.budget_classification_id, c.is_active AS class_is_active,
                c.created_at AS class_created_at, c.updated_at AS class_updated_at,
                bc.name AS budget_classification_name,
                (SELECT COUNT(*) FROM catmat_items WHERE class_id = c.id) AS item_count
            FROM catmat_groups g
            LEFT JOIN catmat_classes c ON c.group_id = g.id
            LEFT JOIN budget_classifications bc ON c.budget_classification_id = bc.id
            ORDER BY g.code, c.code"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let mut groups_map: std::collections::BTreeMap<Uuid, CatmatGroupTreeNode> = std::collections::BTreeMap::new();

        for r in &records {
            let group_id: Uuid = r.get("id");
            let entry = groups_map.entry(group_id).or_insert_with(|| CatmatGroupTreeNode {
                id: group_id,
                code: r.get("code"),
                name: r.get("name"),
                is_active: r.get("is_active"),
                classes: Vec::new(),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });

            if let Some(class_id) = r.try_get::<Option<Uuid>, _>("class_id").unwrap_or(None) {
                entry.classes.push(CatmatClassTreeNode {
                    id: class_id,
                    group_id,
                    code: r.get("class_code"),
                    name: r.get("class_name"),
                    budget_classification_id: r.get("budget_classification_id"),
                    budget_classification_name: r.get("budget_classification_name"),
                    is_active: r.get("class_is_active"),
                    item_count: r.get("item_count"),
                    created_at: r.get("class_created_at"),
                    updated_at: r.get("class_updated_at"),
                });
            }
        }

        Ok(groups_map.into_values().collect())
    }
}

// ============================
// CATMAT Class Repository
// ============================

pub struct CatmatClassRepository {
    pool: PgPool,
}

impl CatmatClassRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatmatClassRepositoryPort for CatmatClassRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatClassDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatClassDto>("SELECT * FROM catmat_classes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatmatClassWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT c.*, g.name AS group_name, g.code AS group_code,
                bc.name AS budget_classification_name, bc.full_code AS budget_classification_code,
                (SELECT COUNT(*) FROM catmat_items WHERE class_id = c.id) AS item_count
            FROM catmat_classes c
            JOIN catmat_groups g ON c.group_id = g.id
            LEFT JOIN budget_classifications bc ON c.budget_classification_id = bc.id
            WHERE c.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatmatClassWithDetailsDto {
            id: r.get("id"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            code: r.get("code"),
            name: r.get("name"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.get("budget_classification_name"),
            budget_classification_code: r.get("budget_classification_code"),
            is_active: r.get("is_active"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_classes WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_classes WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, group_id: Uuid, code: &str, name: &str, budget_classification_id: Option<Uuid>, is_active: bool) -> Result<CatmatClassDto, RepositoryError> {
        sqlx::query_as::<_, CatmatClassDto>(
            "INSERT INTO catmat_classes (group_id, code, name, budget_classification_id, is_active) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(budget_classification_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, group_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, budget_classification_id: Option<Uuid>, is_active: Option<bool>) -> Result<CatmatClassDto, RepositoryError> {
        sqlx::query_as::<_, CatmatClassDto>(
            r#"UPDATE catmat_classes SET group_id = COALESCE($2, group_id), code = COALESCE($3, code),
            name = COALESCE($4, name), budget_classification_id = COALESCE($5, budget_classification_id),
            is_active = COALESCE($6, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(budget_classification_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catmat_classes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_items WHERE class_id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, group_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatmatClassWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT c.*, g.name AS group_name, g.code AS group_code,
                bc.name AS budget_classification_name, bc.full_code AS budget_classification_code,
                (SELECT COUNT(*) FROM catmat_items WHERE class_id = c.id) AS item_count
            FROM catmat_classes c
            JOIN catmat_groups g ON c.group_id = g.id
            LEFT JOIN budget_classifications bc ON c.budget_classification_id = bc.id
            WHERE ($1::TEXT IS NULL OR c.name ILIKE $1 OR c.code ILIKE $1)
              AND ($2::UUID IS NULL OR c.group_id = $2)
              AND ($3::BOOLEAN IS NULL OR c.is_active = $3)
            ORDER BY c.code LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(group_id)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let classes = records.into_iter().map(|r| CatmatClassWithDetailsDto {
            id: r.get("id"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            code: r.get("code"),
            name: r.get("name"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.get("budget_classification_name"),
            budget_classification_code: r.get("budget_classification_code"),
            is_active: r.get("is_active"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catmat_classes c
            WHERE ($1::TEXT IS NULL OR c.name ILIKE $1 OR c.code ILIKE $1)
              AND ($2::UUID IS NULL OR c.group_id = $2)
              AND ($3::BOOLEAN IS NULL OR c.is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(group_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((classes, total))
    }
}

// ============================
// CATMAT Item Repository
// ============================

pub struct CatmatItemRepository {
    pool: PgPool,
}

impl CatmatItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatmatItemRepositoryPort for CatmatItemRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatItemDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatItemDto>("SELECT * FROM catmat_items WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatmatItemWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT i.*, c.name AS class_name, c.code AS class_code,
                g.id AS group_id, g.name AS group_name, g.code AS group_code,
                u.name AS unit_name, u.symbol AS unit_symbol
            FROM catmat_items i
            JOIN catmat_classes c ON i.class_id = c.id
            JOIN catmat_groups g ON c.group_id = g.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            WHERE i.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatmatItemWithDetailsDto {
            id: r.get("id"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            unit_of_measure_id: r.get("unit_of_measure_id"),
            unit_name: r.get("unit_name"),
            unit_symbol: r.get("unit_symbol"),
            code: r.get("code"),
            description: r.get("description"),
            supplementary_description: r.get("supplementary_description"),
            is_sustainable: r.get("is_sustainable"),
            specification: r.get("specification"),
            estimated_value: r.get("estimated_value"),
            search_links: r.get("search_links"),
            photo_url: r.get("photo_url"),
            is_permanent: r.get("is_permanent"),
            shelf_life_days: r.get("shelf_life_days"),
            requires_batch_control: r.get("requires_batch_control"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CatmatItemDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatItemDto>("SELECT * FROM catmat_items WHERE code = $1")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_items WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_items WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(
        &self, class_id: Uuid, unit_of_measure_id: Uuid, code: &str, description: &str,
        supplementary_description: Option<&str>, is_sustainable: bool, specification: Option<&str>,
        estimated_value: rust_decimal::Decimal, search_links: Option<&str>, photo_url: Option<&str>,
        is_permanent: bool, shelf_life_days: Option<i32>, requires_batch_control: bool, is_active: bool,
    ) -> Result<CatmatItemDto, RepositoryError> {
        sqlx::query_as::<_, CatmatItemDto>(
            r#"INSERT INTO catmat_items (class_id, unit_of_measure_id, code, description,
                supplementary_description, is_sustainable, specification, estimated_value,
                search_links, photo_url, is_permanent, shelf_life_days, requires_batch_control, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING *"#,
        )
        .bind(class_id)
        .bind(unit_of_measure_id)
        .bind(code)
        .bind(description)
        .bind(supplementary_description)
        .bind(is_sustainable)
        .bind(specification)
        .bind(estimated_value)
        .bind(search_links)
        .bind(photo_url)
        .bind(is_permanent)
        .bind(shelf_life_days)
        .bind(requires_batch_control)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self, id: Uuid, class_id: Option<Uuid>, unit_of_measure_id: Option<Uuid>,
        code: Option<&str>, description: Option<&str>, supplementary_description: Option<&str>,
        is_sustainable: Option<bool>, specification: Option<&str>,
        estimated_value: Option<rust_decimal::Decimal>, search_links: Option<&str>,
        photo_url: Option<&str>, is_permanent: Option<bool>, shelf_life_days: Option<i32>,
        requires_batch_control: Option<bool>, is_active: Option<bool>,
    ) -> Result<CatmatItemDto, RepositoryError> {
        sqlx::query_as::<_, CatmatItemDto>(
            r#"UPDATE catmat_items SET
                class_id = COALESCE($2, class_id), unit_of_measure_id = COALESCE($3, unit_of_measure_id),
                code = COALESCE($4, code), description = COALESCE($5, description),
                supplementary_description = CASE WHEN $6::TEXT IS NOT NULL THEN $6 ELSE supplementary_description END,
                is_sustainable = COALESCE($7, is_sustainable),
                specification = CASE WHEN $8::TEXT IS NOT NULL THEN $8 ELSE specification END,
                estimated_value = COALESCE($9, estimated_value),
                search_links = CASE WHEN $10::TEXT IS NOT NULL THEN $10 ELSE search_links END,
                photo_url = CASE WHEN $11::TEXT IS NOT NULL THEN $11 ELSE photo_url END,
                is_permanent = COALESCE($12, is_permanent),
                shelf_life_days = CASE WHEN $13::INTEGER IS NOT NULL THEN $13 ELSE shelf_life_days END,
                requires_batch_control = COALESCE($14, requires_batch_control),
                is_active = COALESCE($15, is_active), updated_at = NOW()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(class_id)
        .bind(unit_of_measure_id)
        .bind(code)
        .bind(description)
        .bind(supplementary_description)
        .bind(is_sustainable)
        .bind(specification)
        .bind(estimated_value)
        .bind(search_links)
        .bind(photo_url)
        .bind(is_permanent)
        .bind(shelf_life_days)
        .bind(requires_batch_control)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catmat_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self, limit: i64, offset: i64, search: Option<String>, class_id: Option<Uuid>,
        is_sustainable: Option<bool>, is_permanent: Option<bool>, is_active: Option<bool>,
    ) -> Result<(Vec<CatmatItemWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT i.*, c.name AS class_name, c.code AS class_code,
                g.id AS group_id, g.name AS group_name, g.code AS group_code,
                u.name AS unit_name, u.symbol AS unit_symbol
            FROM catmat_items i
            JOIN catmat_classes c ON i.class_id = c.id
            JOIN catmat_groups g ON c.group_id = g.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            WHERE ($1::TEXT IS NULL OR i.description ILIKE $1 OR i.code ILIKE $1 OR i.specification ILIKE $1)
              AND ($2::UUID IS NULL OR i.class_id = $2)
              AND ($3::BOOLEAN IS NULL OR i.is_sustainable = $3)
              AND ($4::BOOLEAN IS NULL OR i.is_permanent = $4)
              AND ($5::BOOLEAN IS NULL OR i.is_active = $5)
            ORDER BY i.code LIMIT $6 OFFSET $7"#,
        )
        .bind(&search_pattern)
        .bind(class_id)
        .bind(is_sustainable)
        .bind(is_permanent)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = records.into_iter().map(|r| CatmatItemWithDetailsDto {
            id: r.get("id"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            unit_of_measure_id: r.get("unit_of_measure_id"),
            unit_name: r.get("unit_name"),
            unit_symbol: r.get("unit_symbol"),
            code: r.get("code"),
            description: r.get("description"),
            supplementary_description: r.get("supplementary_description"),
            is_sustainable: r.get("is_sustainable"),
            specification: r.get("specification"),
            estimated_value: r.get("estimated_value"),
            search_links: r.get("search_links"),
            photo_url: r.get("photo_url"),
            is_permanent: r.get("is_permanent"),
            shelf_life_days: r.get("shelf_life_days"),
            requires_batch_control: r.get("requires_batch_control"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catmat_items i
            WHERE ($1::TEXT IS NULL OR i.description ILIKE $1 OR i.code ILIKE $1 OR i.specification ILIKE $1)
              AND ($2::UUID IS NULL OR i.class_id = $2)
              AND ($3::BOOLEAN IS NULL OR i.is_sustainable = $3)
              AND ($4::BOOLEAN IS NULL OR i.is_permanent = $4)
              AND ($5::BOOLEAN IS NULL OR i.is_active = $5)"#,
        )
        .bind(&search_pattern)
        .bind(class_id)
        .bind(is_sustainable)
        .bind(is_permanent)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// CATSER Group Repository
// ============================

pub struct CatserGroupRepository {
    pool: PgPool,
}

impl CatserGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatserGroupRepositoryPort for CatserGroupRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatserGroupDto>("SELECT * FROM catser_groups WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_groups WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_groups WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, code: &str, name: &str, is_active: bool) -> Result<CatserGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatserGroupDto>(
            "INSERT INTO catser_groups (code, name, is_active) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatserGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatserGroupDto>(
            r#"UPDATE catser_groups SET code = COALESCE($2, code), name = COALESCE($3, name),
            is_active = COALESCE($4, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catser_groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatserGroupDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let groups = sqlx::query_as::<_, CatserGroupDto>(
            r#"SELECT * FROM catser_groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
              AND ($2::BOOLEAN IS NULL OR is_active = $2)
            ORDER BY code LIMIT $3 OFFSET $4"#,
        )
        .bind(&search_pattern)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catser_groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
              AND ($2::BOOLEAN IS NULL OR is_active = $2)"#,
        )
        .bind(&search_pattern)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((groups, total))
    }

    async fn get_tree(&self) -> Result<Vec<CatserGroupTreeNode>, RepositoryError> {
        let records = sqlx::query(
            r#"SELECT g.id, g.code, g.name, g.is_active, g.created_at, g.updated_at,
                c.id AS class_id, c.code AS class_code, c.name AS class_name,
                c.budget_classification_id, c.is_active AS class_is_active,
                c.created_at AS class_created_at, c.updated_at AS class_updated_at,
                bc.name AS budget_classification_name,
                (SELECT COUNT(*) FROM catser_items WHERE class_id = c.id) AS item_count
            FROM catser_groups g
            LEFT JOIN catser_classes c ON c.group_id = g.id
            LEFT JOIN budget_classifications bc ON c.budget_classification_id = bc.id
            ORDER BY g.code, c.code"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let mut groups_map: std::collections::BTreeMap<Uuid, CatserGroupTreeNode> = std::collections::BTreeMap::new();

        for r in &records {
            let group_id: Uuid = r.get("id");
            let entry = groups_map.entry(group_id).or_insert_with(|| CatserGroupTreeNode {
                id: group_id,
                code: r.get("code"),
                name: r.get("name"),
                is_active: r.get("is_active"),
                classes: Vec::new(),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });

            if let Some(class_id) = r.try_get::<Option<Uuid>, _>("class_id").unwrap_or(None) {
                entry.classes.push(CatserClassTreeNode {
                    id: class_id,
                    group_id,
                    code: r.get("class_code"),
                    name: r.get("class_name"),
                    budget_classification_id: r.get("budget_classification_id"),
                    budget_classification_name: r.get("budget_classification_name"),
                    is_active: r.get("class_is_active"),
                    item_count: r.get("item_count"),
                    created_at: r.get("class_created_at"),
                    updated_at: r.get("class_updated_at"),
                });
            }
        }

        Ok(groups_map.into_values().collect())
    }
}

// ============================
// CATSER Class Repository
// ============================

pub struct CatserClassRepository {
    pool: PgPool,
}

impl CatserClassRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatserClassRepositoryPort for CatserClassRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserClassDto>, RepositoryError> {
        sqlx::query_as::<_, CatserClassDto>("SELECT * FROM catser_classes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatserClassWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT c.*, g.name AS group_name, g.code AS group_code,
                bc.name AS budget_classification_name, bc.full_code AS budget_classification_code,
                (SELECT COUNT(*) FROM catser_items WHERE class_id = c.id) AS item_count
            FROM catser_classes c
            JOIN catser_groups g ON c.group_id = g.id
            LEFT JOIN budget_classifications bc ON c.budget_classification_id = bc.id
            WHERE c.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatserClassWithDetailsDto {
            id: r.get("id"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            code: r.get("code"),
            name: r.get("name"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.get("budget_classification_name"),
            budget_classification_code: r.get("budget_classification_code"),
            is_active: r.get("is_active"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_classes WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_classes WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, group_id: Uuid, code: &str, name: &str, budget_classification_id: Option<Uuid>, is_active: bool) -> Result<CatserClassDto, RepositoryError> {
        sqlx::query_as::<_, CatserClassDto>(
            "INSERT INTO catser_classes (group_id, code, name, budget_classification_id, is_active) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(budget_classification_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, group_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, budget_classification_id: Option<Uuid>, is_active: Option<bool>) -> Result<CatserClassDto, RepositoryError> {
        sqlx::query_as::<_, CatserClassDto>(
            r#"UPDATE catser_classes SET group_id = COALESCE($2, group_id), code = COALESCE($3, code),
            name = COALESCE($4, name), budget_classification_id = COALESCE($5, budget_classification_id),
            is_active = COALESCE($6, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(budget_classification_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catser_classes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_items WHERE class_id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, group_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatserClassWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT c.*, g.name AS group_name, g.code AS group_code,
                bc.name AS budget_classification_name, bc.full_code AS budget_classification_code,
                (SELECT COUNT(*) FROM catser_items WHERE class_id = c.id) AS item_count
            FROM catser_classes c
            JOIN catser_groups g ON c.group_id = g.id
            LEFT JOIN budget_classifications bc ON c.budget_classification_id = bc.id
            WHERE ($1::TEXT IS NULL OR c.name ILIKE $1 OR c.code ILIKE $1)
              AND ($2::UUID IS NULL OR c.group_id = $2)
              AND ($3::BOOLEAN IS NULL OR c.is_active = $3)
            ORDER BY c.code LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(group_id)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let classes = records.into_iter().map(|r| CatserClassWithDetailsDto {
            id: r.get("id"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            code: r.get("code"),
            name: r.get("name"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.get("budget_classification_name"),
            budget_classification_code: r.get("budget_classification_code"),
            is_active: r.get("is_active"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catser_classes c
            WHERE ($1::TEXT IS NULL OR c.name ILIKE $1 OR c.code ILIKE $1)
              AND ($2::UUID IS NULL OR c.group_id = $2)
              AND ($3::BOOLEAN IS NULL OR c.is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(group_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((classes, total))
    }
}

// ============================
// CATSER Item Repository
// ============================

pub struct CatserItemRepository {
    pool: PgPool,
}

impl CatserItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatserItemRepositoryPort for CatserItemRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserItemDto>, RepositoryError> {
        sqlx::query_as::<_, CatserItemDto>("SELECT * FROM catser_items WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatserItemWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT i.*, c.name AS class_name, c.code AS class_code,
                g.id AS group_id, g.name AS group_name, g.code AS group_code,
                u.name AS unit_name, u.symbol AS unit_symbol
            FROM catser_items i
            JOIN catser_classes c ON i.class_id = c.id
            JOIN catser_groups g ON c.group_id = g.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            WHERE i.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatserItemWithDetailsDto {
            id: r.get("id"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            unit_of_measure_id: r.get("unit_of_measure_id"),
            unit_name: r.get("unit_name"),
            unit_symbol: r.get("unit_symbol"),
            code: r.get("code"),
            description: r.get("description"),
            supplementary_description: r.get("supplementary_description"),
            specification: r.get("specification"),
            estimated_value: r.get("estimated_value"),
            search_links: r.get("search_links"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CatserItemDto>, RepositoryError> {
        sqlx::query_as::<_, CatserItemDto>("SELECT * FROM catser_items WHERE code = $1")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_items WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_items WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(
        &self, class_id: Uuid, unit_of_measure_id: Uuid, code: &str, description: &str,
        supplementary_description: Option<&str>, specification: Option<&str>,
        estimated_value: rust_decimal::Decimal, search_links: Option<&str>, is_active: bool,
    ) -> Result<CatserItemDto, RepositoryError> {
        sqlx::query_as::<_, CatserItemDto>(
            r#"INSERT INTO catser_items (class_id, unit_of_measure_id, code, description,
                supplementary_description, specification, estimated_value, search_links, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
        )
        .bind(class_id)
        .bind(unit_of_measure_id)
        .bind(code)
        .bind(description)
        .bind(supplementary_description)
        .bind(specification)
        .bind(estimated_value)
        .bind(search_links)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self, id: Uuid, class_id: Option<Uuid>, unit_of_measure_id: Option<Uuid>,
        code: Option<&str>, description: Option<&str>, supplementary_description: Option<&str>,
        specification: Option<&str>, estimated_value: Option<rust_decimal::Decimal>,
        search_links: Option<&str>, is_active: Option<bool>,
    ) -> Result<CatserItemDto, RepositoryError> {
        sqlx::query_as::<_, CatserItemDto>(
            r#"UPDATE catser_items SET
                class_id = COALESCE($2, class_id), unit_of_measure_id = COALESCE($3, unit_of_measure_id),
                code = COALESCE($4, code), description = COALESCE($5, description),
                supplementary_description = CASE WHEN $6::TEXT IS NOT NULL THEN $6 ELSE supplementary_description END,
                specification = CASE WHEN $7::TEXT IS NOT NULL THEN $7 ELSE specification END,
                estimated_value = COALESCE($8, estimated_value),
                search_links = CASE WHEN $9::TEXT IS NOT NULL THEN $9 ELSE search_links END,
                is_active = COALESCE($10, is_active), updated_at = NOW()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(class_id)
        .bind(unit_of_measure_id)
        .bind(code)
        .bind(description)
        .bind(supplementary_description)
        .bind(specification)
        .bind(estimated_value)
        .bind(search_links)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catser_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self, limit: i64, offset: i64, search: Option<String>, class_id: Option<Uuid>, is_active: Option<bool>,
    ) -> Result<(Vec<CatserItemWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT i.*, c.name AS class_name, c.code AS class_code,
                g.id AS group_id, g.name AS group_name, g.code AS group_code,
                u.name AS unit_name, u.symbol AS unit_symbol
            FROM catser_items i
            JOIN catser_classes c ON i.class_id = c.id
            JOIN catser_groups g ON c.group_id = g.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            WHERE ($1::TEXT IS NULL OR i.description ILIKE $1 OR i.code ILIKE $1 OR i.specification ILIKE $1)
              AND ($2::UUID IS NULL OR i.class_id = $2)
              AND ($3::BOOLEAN IS NULL OR i.is_active = $3)
            ORDER BY i.code LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(class_id)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = records.into_iter().map(|r| CatserItemWithDetailsDto {
            id: r.get("id"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            unit_of_measure_id: r.get("unit_of_measure_id"),
            unit_name: r.get("unit_name"),
            unit_symbol: r.get("unit_symbol"),
            code: r.get("code"),
            description: r.get("description"),
            supplementary_description: r.get("supplementary_description"),
            specification: r.get("specification"),
            estimated_value: r.get("estimated_value"),
            search_links: r.get("search_links"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catser_items i
            WHERE ($1::TEXT IS NULL OR i.description ILIKE $1 OR i.code ILIKE $1 OR i.specification ILIKE $1)
              AND ($2::UUID IS NULL OR i.class_id = $2)
              AND ($3::BOOLEAN IS NULL OR i.is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(class_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

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
}

#[async_trait]
impl UnitConversionRepositoryPort for UnitConversionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitConversionDto>, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>("SELECT * FROM unit_conversions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<UnitConversionWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT uc.*, from_unit.name as from_unit_name, from_unit.symbol as from_unit_symbol,
                to_unit.name as to_unit_name, to_unit.symbol as to_unit_symbol
            FROM unit_conversions uc
            JOIN units_of_measure from_unit ON uc.from_unit_id = from_unit.id
            JOIN units_of_measure to_unit ON uc.to_unit_id = to_unit.id
            WHERE uc.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| UnitConversionWithDetailsDto {
            id: r.get("id"),
            from_unit_id: r.get("from_unit_id"),
            from_unit_name: r.get("from_unit_name"),
            from_unit_symbol: r.get("from_unit_symbol"),
            to_unit_id: r.get("to_unit_id"),
            to_unit_name: r.get("to_unit_name"),
            to_unit_symbol: r.get("to_unit_symbol"),
            conversion_factor: r.get("conversion_factor"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_conversion(&self, from_unit_id: Uuid, to_unit_id: Uuid) -> Result<Option<UnitConversionDto>, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            "SELECT * FROM unit_conversions WHERE from_unit_id = $1 AND to_unit_id = $2",
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_conversion(&self, from_unit_id: Uuid, to_unit_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM unit_conversions WHERE from_unit_id = $1 AND to_unit_id = $2",
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, from_unit_id: Uuid, to_unit_id: Uuid, conversion_factor: rust_decimal::Decimal) -> Result<UnitConversionDto, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            "INSERT INTO unit_conversions (from_unit_id, to_unit_id, conversion_factor) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .bind(conversion_factor)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, conversion_factor: rust_decimal::Decimal) -> Result<UnitConversionDto, RepositoryError> {
        sqlx::query_as::<_, UnitConversionDto>(
            "UPDATE unit_conversions SET conversion_factor = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(conversion_factor)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM unit_conversions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, from_unit_id: Option<Uuid>, to_unit_id: Option<Uuid>) -> Result<(Vec<UnitConversionWithDetailsDto>, i64), RepositoryError> {
        let records = sqlx::query(
            r#"SELECT uc.*, from_unit.name as from_unit_name, from_unit.symbol as from_unit_symbol,
                to_unit.name as to_unit_name, to_unit.symbol as to_unit_symbol
            FROM unit_conversions uc
            JOIN units_of_measure from_unit ON uc.from_unit_id = from_unit.id
            JOIN units_of_measure to_unit ON uc.to_unit_id = to_unit.id
            WHERE ($1::UUID IS NULL OR uc.from_unit_id = $1)
              AND ($2::UUID IS NULL OR uc.to_unit_id = $2)
            ORDER BY from_unit.name, to_unit.name
            LIMIT $3 OFFSET $4"#,
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let conversions = records.into_iter().map(|r| UnitConversionWithDetailsDto {
            id: r.get("id"),
            from_unit_id: r.get("from_unit_id"),
            from_unit_name: r.get("from_unit_name"),
            from_unit_symbol: r.get("from_unit_symbol"),
            to_unit_id: r.get("to_unit_id"),
            to_unit_name: r.get("to_unit_name"),
            to_unit_symbol: r.get("to_unit_symbol"),
            conversion_factor: r.get("conversion_factor"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM unit_conversions uc
            WHERE ($1::UUID IS NULL OR uc.from_unit_id = $1)
              AND ($2::UUID IS NULL OR uc.to_unit_id = $2)"#,
        )
        .bind(from_unit_id)
        .bind(to_unit_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((conversions, total))
    }
}
