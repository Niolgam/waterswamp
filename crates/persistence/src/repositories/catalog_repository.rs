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

    async fn find_by_code(&self, code: &str) -> Result<Option<CatmatGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatGroupDto>("SELECT * FROM catmat_groups WHERE code = $1")
            .bind(code)
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

    async fn create(&self, code: &str, name: &str, is_active: bool, verification_status: &str) -> Result<CatmatGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatmatGroupDto>(
            "INSERT INTO catmat_groups (code, name, is_active, verification_status) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(code)
        .bind(name)
        .bind(is_active)
        .bind(verification_status)
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

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catmat_groups SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
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
                c.is_active AS class_is_active,
                c.created_at AS class_created_at, c.updated_at AS class_updated_at,
                (SELECT COUNT(*) FROM catmat_pdms WHERE class_id = c.id) AS pdm_count
            FROM catmat_groups g
            LEFT JOIN catmat_classes c ON c.group_id = g.id
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
                    is_active: r.get("class_is_active"),
                    pdm_count: r.get("pdm_count"),
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
                (SELECT COUNT(*) FROM catmat_pdms WHERE class_id = c.id) AS pdm_count
            FROM catmat_classes c
            JOIN catmat_groups g ON c.group_id = g.id
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
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            pdm_count: r.get("pdm_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CatmatClassDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatClassDto>("SELECT * FROM catmat_classes WHERE code = $1")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
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

    async fn create(&self, group_id: Uuid, code: &str, name: &str, is_active: bool, verification_status: &str) -> Result<CatmatClassDto, RepositoryError> {
        sqlx::query_as::<_, CatmatClassDto>(
            "INSERT INTO catmat_classes (group_id, code, name, is_active, verification_status) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, group_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatmatClassDto, RepositoryError> {
        sqlx::query_as::<_, CatmatClassDto>(
            r#"UPDATE catmat_classes SET group_id = COALESCE($2, group_id), code = COALESCE($3, code),
            name = COALESCE($4, name), is_active = COALESCE($5, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catmat_classes SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catmat_classes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_pdms(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_pdms WHERE class_id = $1")
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
                (SELECT COUNT(*) FROM catmat_pdms WHERE class_id = c.id) AS pdm_count
            FROM catmat_classes c
            JOIN catmat_groups g ON c.group_id = g.id
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
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            pdm_count: r.get("pdm_count"),
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
// CATMAT PDM Repository
// ============================

pub struct CatmatPdmRepository {
    pool: PgPool,
}

impl CatmatPdmRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatmatPdmRepositoryPort for CatmatPdmRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatmatPdmDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatPdmDto>("SELECT * FROM catmat_pdms WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatmatPdmWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT p.*, cc.name AS class_name, cc.code AS class_code,
                cg.id AS group_id, cg.name AS group_name, cg.code AS group_code,
                (SELECT COUNT(*) FROM catmat_items WHERE pdm_id = p.id) AS item_count
            FROM catmat_pdms p
            JOIN catmat_classes cc ON p.class_id = cc.id
            JOIN catmat_groups cg ON cc.group_id = cg.id
            WHERE p.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatmatPdmWithDetailsDto {
            id: r.get("id"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            code: r.get("code"),
            description: r.get("description"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CatmatPdmDto>, RepositoryError> {
        sqlx::query_as::<_, CatmatPdmDto>("SELECT * FROM catmat_pdms WHERE code = $1")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_pdms WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_pdms WHERE code = $1 AND id != $2")
            .bind(code)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, class_id: Uuid, code: &str, description: &str, is_active: bool, verification_status: &str) -> Result<CatmatPdmDto, RepositoryError> {
        sqlx::query_as::<_, CatmatPdmDto>(
            "INSERT INTO catmat_pdms (class_id, code, description, is_active, verification_status) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(class_id)
        .bind(code)
        .bind(description)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, class_id: Option<Uuid>, code: Option<&str>, description: Option<&str>, is_active: Option<bool>) -> Result<CatmatPdmDto, RepositoryError> {
        let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE catmat_pdms SET ");
        let mut separated = builder.separated(", ");

        if let Some(v) = class_id {
            separated.push("class_id = ");
            separated.push_bind_unseparated(v);
        }
        if let Some(v) = code {
            separated.push("code = ");
            separated.push_bind_unseparated(v.to_string());
        }
        if let Some(v) = description {
            separated.push("description = ");
            separated.push_bind_unseparated(v.to_string());
        }
        if let Some(v) = is_active {
            separated.push("is_active = ");
            separated.push_bind_unseparated(v);
        }

        separated.push("updated_at = NOW()");

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" RETURNING *");

        builder
            .build_query_as::<CatmatPdmDto>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catmat_pdms SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catmat_pdms WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_items(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catmat_items WHERE pdm_id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, class_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatmatPdmWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT p.*, cc.name AS class_name, cc.code AS class_code,
                cg.id AS group_id, cg.name AS group_name, cg.code AS group_code,
                (SELECT COUNT(*) FROM catmat_items WHERE pdm_id = p.id) AS item_count
            FROM catmat_pdms p
            JOIN catmat_classes cc ON p.class_id = cc.id
            JOIN catmat_groups cg ON cc.group_id = cg.id
            WHERE ($1::TEXT IS NULL OR p.description ILIKE $1 OR p.code ILIKE $1)
              AND ($2::UUID IS NULL OR p.class_id = $2)
              AND ($3::BOOLEAN IS NULL OR p.is_active = $3)
            ORDER BY p.code LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(class_id)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let pdms = records.into_iter().map(|r| CatmatPdmWithDetailsDto {
            id: r.get("id"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            code: r.get("code"),
            description: r.get("description"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catmat_pdms p
            WHERE ($1::TEXT IS NULL OR p.description ILIKE $1 OR p.code ILIKE $1)
              AND ($2::UUID IS NULL OR p.class_id = $2)
              AND ($3::BOOLEAN IS NULL OR p.is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(class_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((pdms, total))
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
            r#"SELECT i.*, p.description AS pdm_description, p.code AS pdm_code,
                cc.id AS class_id, cc.name AS class_name, cc.code AS class_code,
                cg.id AS group_id, cg.name AS group_name, cg.code AS group_code,
                u.name AS unit_name, u.symbol AS unit_symbol,
                bc.name AS bc_name, bc.full_code AS bc_full_code
            FROM catmat_items i
            JOIN catmat_pdms p ON i.pdm_id = p.id
            JOIN catmat_classes cc ON p.class_id = cc.id
            JOIN catmat_groups cg ON cc.group_id = cg.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            LEFT JOIN budget_classifications bc ON i.budget_classification_id = bc.id
            WHERE i.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatmatItemWithDetailsDto {
            id: r.get("id"),
            pdm_id: r.get("pdm_id"),
            pdm_description: r.get("pdm_description"),
            pdm_code: r.get("pdm_code"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            unit_of_measure_id: r.get("unit_of_measure_id"),
            unit_name: r.get("unit_name"),
            unit_symbol: r.get("unit_symbol"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.try_get("bc_name").ok().flatten(),
            budget_classification_full_code: r.try_get("bc_full_code").ok().flatten(),
            code: r.get("code"),
            description: r.get("description"),
            is_sustainable: r.get("is_sustainable"),
            code_ncm: r.get("code_ncm"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
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
        &self, pdm_id: Uuid, unit_of_measure_id: Uuid, budget_classification_id: Option<Uuid>,
        code: &str, description: &str,
        is_sustainable: bool, code_ncm: Option<&str>, is_active: bool, verification_status: &str,
    ) -> Result<CatmatItemDto, RepositoryError> {
        sqlx::query_as::<_, CatmatItemDto>(
            r#"INSERT INTO catmat_items (pdm_id, unit_of_measure_id, budget_classification_id, code, description,
                is_sustainable, code_ncm, is_active, verification_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
        )
        .bind(pdm_id)
        .bind(unit_of_measure_id)
        .bind(budget_classification_id)
        .bind(code)
        .bind(description)
        .bind(is_sustainable)
        .bind(code_ncm)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self, id: Uuid, pdm_id: Option<Uuid>, unit_of_measure_id: Option<Uuid>,
        budget_classification_id: Option<Uuid>,
        code: Option<&str>, description: Option<&str>, is_sustainable: Option<bool>,
        code_ncm: Option<&str>, is_active: Option<bool>,
    ) -> Result<CatmatItemDto, RepositoryError> {
        let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE catmat_items SET ");
        let mut separated = builder.separated(", ");

        if let Some(v) = pdm_id {
            separated.push("pdm_id = ");
            separated.push_bind_unseparated(v);
        }
        if let Some(v) = unit_of_measure_id {
            separated.push("unit_of_measure_id = ");
            separated.push_bind_unseparated(v);
        }
        if let Some(v) = budget_classification_id {
            separated.push("budget_classification_id = ");
            separated.push_bind_unseparated(v);
        }
        if let Some(v) = code {
            separated.push("code = ");
            separated.push_bind_unseparated(v.to_string());
        }
        if let Some(v) = description {
            separated.push("description = ");
            separated.push_bind_unseparated(v.to_string());
        }
        if let Some(v) = is_sustainable {
            separated.push("is_sustainable = ");
            separated.push_bind_unseparated(v);
        }
        if let Some(v) = code_ncm {
            separated.push("code_ncm = ");
            separated.push_bind_unseparated(v.to_string());
        }
        if let Some(v) = is_active {
            separated.push("is_active = ");
            separated.push_bind_unseparated(v);
        }

        separated.push("updated_at = NOW()");

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" RETURNING *");

        builder
            .build_query_as::<CatmatItemDto>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catmat_items SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
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
        &self, limit: i64, offset: i64, search: Option<String>, pdm_id: Option<Uuid>,
        is_sustainable: Option<bool>, is_active: Option<bool>,
    ) -> Result<(Vec<CatmatItemWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT i.*, p.description AS pdm_description, p.code AS pdm_code,
                cc.id AS class_id, cc.name AS class_name, cc.code AS class_code,
                cg.id AS group_id, cg.name AS group_name, cg.code AS group_code,
                u.name AS unit_name, u.symbol AS unit_symbol,
                bc.name AS bc_name, bc.full_code AS bc_full_code
            FROM catmat_items i
            JOIN catmat_pdms p ON i.pdm_id = p.id
            JOIN catmat_classes cc ON p.class_id = cc.id
            JOIN catmat_groups cg ON cc.group_id = cg.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            LEFT JOIN budget_classifications bc ON i.budget_classification_id = bc.id
            WHERE ($1::TEXT IS NULL OR i.description ILIKE $1 OR i.code ILIKE $1)
              AND ($2::UUID IS NULL OR i.pdm_id = $2)
              AND ($3::BOOLEAN IS NULL OR i.is_sustainable = $3)
              AND ($4::BOOLEAN IS NULL OR i.is_active = $4)
            ORDER BY i.code LIMIT $5 OFFSET $6"#,
        )
        .bind(&search_pattern)
        .bind(pdm_id)
        .bind(is_sustainable)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = records.into_iter().map(|r| CatmatItemWithDetailsDto {
            id: r.get("id"),
            pdm_id: r.get("pdm_id"),
            pdm_description: r.get("pdm_description"),
            pdm_code: r.get("pdm_code"),
            class_id: r.get("class_id"),
            class_name: r.get("class_name"),
            class_code: r.get("class_code"),
            group_id: r.get("group_id"),
            group_name: r.get("group_name"),
            group_code: r.get("group_code"),
            unit_of_measure_id: r.get("unit_of_measure_id"),
            unit_name: r.get("unit_name"),
            unit_symbol: r.get("unit_symbol"),
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.try_get("bc_name").ok().flatten(),
            budget_classification_full_code: r.try_get("bc_full_code").ok().flatten(),
            code: r.get("code"),
            description: r.get("description"),
            is_sustainable: r.get("is_sustainable"),
            code_ncm: r.get("code_ncm"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catmat_items i
            WHERE ($1::TEXT IS NULL OR i.description ILIKE $1 OR i.code ILIKE $1)
              AND ($2::UUID IS NULL OR i.pdm_id = $2)
              AND ($3::BOOLEAN IS NULL OR i.is_sustainable = $3)
              AND ($4::BOOLEAN IS NULL OR i.is_active = $4)"#,
        )
        .bind(&search_pattern)
        .bind(pdm_id)
        .bind(is_sustainable)
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

    async fn find_by_code(&self, code: &str) -> Result<Option<CatserGroupDto>, RepositoryError> {
        sqlx::query_as::<_, CatserGroupDto>("SELECT * FROM catser_groups WHERE code = $1")
            .bind(code)
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

    async fn create(&self, divisao_id: Option<Uuid>, code: &str, name: &str, is_active: bool, verification_status: &str) -> Result<CatserGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatserGroupDto>(
            "INSERT INTO catser_groups (divisao_id, code, name, is_active, verification_status) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(divisao_id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, divisao_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatserGroupDto, RepositoryError> {
        sqlx::query_as::<_, CatserGroupDto>(
            r#"UPDATE catser_groups SET divisao_id = CASE WHEN $2::UUID IS NOT NULL THEN $2 ELSE divisao_id END, code = COALESCE($3, code), name = COALESCE($4, name),
            is_active = COALESCE($5, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(divisao_id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catser_groups SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catser_groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, divisao_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatserGroupDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let groups = sqlx::query_as::<_, CatserGroupDto>(
            r#"SELECT * FROM catser_groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
              AND ($2::UUID IS NULL OR divisao_id = $2)
              AND ($3::BOOLEAN IS NULL OR is_active = $3)
            ORDER BY code LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(divisao_id)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catser_groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR code ILIKE $1)
              AND ($2::UUID IS NULL OR divisao_id = $2)
              AND ($3::BOOLEAN IS NULL OR is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(divisao_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((groups, total))
    }

    async fn get_tree(&self) -> Result<Vec<CatserGroupTreeNode>, RepositoryError> {
        let records = sqlx::query(
            r#"SELECT g.id, g.divisao_id, g.code, g.name, g.is_active, g.created_at, g.updated_at,
                c.id AS class_id, c.code AS class_code, c.name AS class_name,
                c.is_active AS class_is_active,
                c.created_at AS class_created_at, c.updated_at AS class_updated_at,
                (SELECT COUNT(*) FROM catser_items WHERE class_id = c.id) AS item_count
            FROM catser_groups g
            LEFT JOIN catser_classes c ON c.group_id = g.id
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
                divisao_id: r.get("divisao_id"),
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
                (SELECT COUNT(*) FROM catser_items WHERE class_id = c.id) AS item_count
            FROM catser_classes c
            JOIN catser_groups g ON c.group_id = g.id
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
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            item_count: r.get("item_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<CatserClassDto>, RepositoryError> {
        sqlx::query_as::<_, CatserClassDto>("SELECT * FROM catser_classes WHERE code = $1")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
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

    async fn create(&self, group_id: Uuid, code: &str, name: &str, is_active: bool, verification_status: &str) -> Result<CatserClassDto, RepositoryError> {
        sqlx::query_as::<_, CatserClassDto>(
            "INSERT INTO catser_classes (group_id, code, name, is_active, verification_status) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, group_id: Option<Uuid>, code: Option<&str>, name: Option<&str>, is_active: Option<bool>) -> Result<CatserClassDto, RepositoryError> {
        sqlx::query_as::<_, CatserClassDto>(
            r#"UPDATE catser_classes SET group_id = COALESCE($2, group_id), code = COALESCE($3, code),
            name = COALESCE($4, name), is_active = COALESCE($5, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catser_classes SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
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
                (SELECT COUNT(*) FROM catser_items WHERE class_id = c.id) AS item_count
            FROM catser_classes c
            JOIN catser_groups g ON c.group_id = g.id
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
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
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
                u.name AS unit_name, u.symbol AS unit_symbol,
                bc.name AS bc_name, bc.full_code AS bc_full_code
            FROM catser_items i
            JOIN catser_classes c ON i.class_id = c.id
            JOIN catser_groups g ON c.group_id = g.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            LEFT JOIN budget_classifications bc ON i.budget_classification_id = bc.id
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
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.try_get("bc_name").ok().flatten(),
            budget_classification_full_code: r.try_get("bc_full_code").ok().flatten(),
            code: r.get("code"),
            code_cpc: r.get("code_cpc"),
            description: r.get("description"),
            supplementary_description: r.get("supplementary_description"),
            specification: r.get("specification"),
            search_links: r.get("search_links"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
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
        &self, class_id: Uuid, unit_of_measure_id: Uuid, budget_classification_id: Option<Uuid>,
        code: &str, code_cpc: Option<&str>, description: &str,
        supplementary_description: Option<&str>, specification: Option<&str>,
        search_links: Option<&str>, is_active: bool, verification_status: &str,
    ) -> Result<CatserItemDto, RepositoryError> {
        sqlx::query_as::<_, CatserItemDto>(
            r#"INSERT INTO catser_items (class_id, unit_of_measure_id, budget_classification_id, code, code_cpc, description,
                supplementary_description, specification, search_links, is_active, verification_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"#,
        )
        .bind(class_id)
        .bind(unit_of_measure_id)
        .bind(budget_classification_id)
        .bind(code)
        .bind(code_cpc)
        .bind(description)
        .bind(supplementary_description)
        .bind(specification)
        .bind(search_links)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self, id: Uuid, class_id: Option<Uuid>, unit_of_measure_id: Option<Uuid>,
        budget_classification_id: Option<Uuid>,
        code: Option<&str>, code_cpc: Option<&str>, description: Option<&str>,
        supplementary_description: Option<&str>,
        specification: Option<&str>, search_links: Option<&str>, is_active: Option<bool>,
    ) -> Result<CatserItemDto, RepositoryError> {
        sqlx::query_as::<_, CatserItemDto>(
            r#"UPDATE catser_items SET
                class_id = COALESCE($2, class_id), unit_of_measure_id = COALESCE($3, unit_of_measure_id),
                budget_classification_id = CASE WHEN $4::UUID IS NOT NULL THEN $4 ELSE budget_classification_id END,
                code = COALESCE($5, code),
                code_cpc = CASE WHEN $6::TEXT IS NOT NULL THEN $6 ELSE code_cpc END,
                description = COALESCE($7, description),
                supplementary_description = CASE WHEN $8::TEXT IS NOT NULL THEN $8 ELSE supplementary_description END,
                specification = CASE WHEN $9::TEXT IS NOT NULL THEN $9 ELSE specification END,
                search_links = CASE WHEN $10::TEXT IS NOT NULL THEN $10 ELSE search_links END,
                is_active = COALESCE($11, is_active), updated_at = NOW()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(class_id)
        .bind(unit_of_measure_id)
        .bind(budget_classification_id)
        .bind(code)
        .bind(code_cpc)
        .bind(description)
        .bind(supplementary_description)
        .bind(specification)
        .bind(search_links)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catser_items SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
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
                u.name AS unit_name, u.symbol AS unit_symbol,
                bc.name AS bc_name, bc.full_code AS bc_full_code
            FROM catser_items i
            JOIN catser_classes c ON i.class_id = c.id
            JOIN catser_groups g ON c.group_id = g.id
            JOIN units_of_measure u ON i.unit_of_measure_id = u.id
            LEFT JOIN budget_classifications bc ON i.budget_classification_id = bc.id
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
            budget_classification_id: r.get("budget_classification_id"),
            budget_classification_name: r.try_get("bc_name").ok().flatten(),
            budget_classification_full_code: r.try_get("bc_full_code").ok().flatten(),
            code: r.get("code"),
            code_cpc: r.get("code_cpc"),
            description: r.get("description"),
            supplementary_description: r.get("supplementary_description"),
            specification: r.get("specification"),
            search_links: r.get("search_links"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
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
// CATSER Seção Repository
// ============================

pub struct CatserSecaoRepository {
    pool: PgPool,
}

impl CatserSecaoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatserSecaoRepositoryPort for CatserSecaoRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserSecaoDto>, RepositoryError> {
        sqlx::query_as::<_, CatserSecaoDto>("SELECT * FROM catser_secoes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatserSecaoWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT s.*,
                (SELECT COUNT(*) FROM catser_divisoes WHERE secao_id = s.id) AS divisao_count
            FROM catser_secoes s
            WHERE s.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatserSecaoWithDetailsDto {
            id: r.get("id"),
            name: r.get("name"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            divisao_count: r.get("divisao_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn create(&self, name: &str, is_active: bool, verification_status: &str) -> Result<CatserSecaoDto, RepositoryError> {
        sqlx::query_as::<_, CatserSecaoDto>(
            "INSERT INTO catser_secoes (name, is_active, verification_status) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(name)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<CatserSecaoDto, RepositoryError> {
        sqlx::query_as::<_, CatserSecaoDto>(
            r#"UPDATE catser_secoes SET name = COALESCE($2, name),
            is_active = COALESCE($3, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catser_secoes SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catser_secoes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_divisoes(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_divisoes WHERE secao_id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, is_active: Option<bool>) -> Result<(Vec<CatserSecaoWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT s.*,
                (SELECT COUNT(*) FROM catser_divisoes WHERE secao_id = s.id) AS divisao_count
            FROM catser_secoes s
            WHERE ($1::TEXT IS NULL OR s.name ILIKE $1)
              AND ($2::BOOLEAN IS NULL OR s.is_active = $2)
            ORDER BY s.name LIMIT $3 OFFSET $4"#,
        )
        .bind(&search_pattern)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let secoes = records.into_iter().map(|r| CatserSecaoWithDetailsDto {
            id: r.get("id"),
            name: r.get("name"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            divisao_count: r.get("divisao_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catser_secoes s
            WHERE ($1::TEXT IS NULL OR s.name ILIKE $1)
              AND ($2::BOOLEAN IS NULL OR s.is_active = $2)"#,
        )
        .bind(&search_pattern)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((secoes, total))
    }

    async fn get_tree(&self) -> Result<Vec<CatserSecaoTreeNode>, RepositoryError> {
        let records = sqlx::query(
            r#"SELECT s.id, s.name, s.is_active, s.created_at, s.updated_at,
                d.id AS divisao_id, d.secao_id AS divisao_secao_id, d.name AS divisao_name,
                d.is_active AS divisao_is_active,
                d.created_at AS divisao_created_at, d.updated_at AS divisao_updated_at,
                g.id AS group_id, g.divisao_id AS group_divisao_id, g.code AS group_code,
                g.name AS group_name, g.is_active AS group_is_active,
                g.created_at AS group_created_at, g.updated_at AS group_updated_at,
                (SELECT COUNT(*) FROM catser_classes WHERE group_id = g.id) AS class_count
            FROM catser_secoes s
            LEFT JOIN catser_divisoes d ON d.secao_id = s.id
            LEFT JOIN catser_groups g ON g.divisao_id = d.id
            ORDER BY s.name, d.name, g.code"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let mut secoes_map: std::collections::BTreeMap<Uuid, CatserSecaoTreeNode> = std::collections::BTreeMap::new();
        let mut divisoes_map: std::collections::BTreeMap<Uuid, CatserDivisaoTreeNode> = std::collections::BTreeMap::new();

        for r in &records {
            let secao_id: Uuid = r.get("id");
            secoes_map.entry(secao_id).or_insert_with(|| CatserSecaoTreeNode {
                id: secao_id,
                name: r.get("name"),
                is_active: r.get("is_active"),
                divisoes: Vec::new(),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });

            if let Some(divisao_id) = r.try_get::<Option<Uuid>, _>("divisao_id").unwrap_or(None) {
                let divisao_entry = divisoes_map.entry(divisao_id).or_insert_with(|| CatserDivisaoTreeNode {
                    id: divisao_id,
                    secao_id,
                    name: r.get("divisao_name"),
                    is_active: r.get("divisao_is_active"),
                    grupos: Vec::new(),
                    created_at: r.get("divisao_created_at"),
                    updated_at: r.get("divisao_updated_at"),
                });

                if let Some(group_id) = r.try_get::<Option<Uuid>, _>("group_id").unwrap_or(None) {
                    // Avoid duplicates
                    if !divisao_entry.grupos.iter().any(|g| g.id == group_id) {
                        divisao_entry.grupos.push(CatserGroupTreeNode {
                            id: group_id,
                            divisao_id: r.get("group_divisao_id"),
                            code: r.get("group_code"),
                            name: r.get("group_name"),
                            is_active: r.get("group_is_active"),
                            classes: Vec::new(),
                            created_at: r.get("group_created_at"),
                            updated_at: r.get("group_updated_at"),
                        });
                    }
                }
            }
        }

        // Assign divisoes to their secoes
        for (divisao_id, divisao_node) in divisoes_map {
            if let Some(secao_node) = secoes_map.get_mut(&divisao_node.secao_id) {
                secao_node.divisoes.push(divisao_node);
            }
        }

        Ok(secoes_map.into_values().collect())
    }
}

// ============================
// CATSER Divisão Repository
// ============================

pub struct CatserDivisaoRepository {
    pool: PgPool,
}

impl CatserDivisaoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatserDivisaoRepositoryPort for CatserDivisaoRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CatserDivisaoDto>, RepositoryError> {
        sqlx::query_as::<_, CatserDivisaoDto>("SELECT * FROM catser_divisoes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<CatserDivisaoWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"SELECT d.*, s.name AS secao_name,
                (SELECT COUNT(*) FROM catser_groups WHERE divisao_id = d.id) AS grupo_count
            FROM catser_divisoes d
            JOIN catser_secoes s ON d.secao_id = s.id
            WHERE d.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| CatserDivisaoWithDetailsDto {
            id: r.get("id"),
            secao_id: r.get("secao_id"),
            secao_name: r.get("secao_name"),
            name: r.get("name"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            grupo_count: r.get("grupo_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn create(&self, secao_id: Uuid, name: &str, is_active: bool, verification_status: &str) -> Result<CatserDivisaoDto, RepositoryError> {
        sqlx::query_as::<_, CatserDivisaoDto>(
            "INSERT INTO catser_divisoes (secao_id, name, is_active, verification_status) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(secao_id)
        .bind(name)
        .bind(is_active)
        .bind(verification_status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, secao_id: Option<Uuid>, name: Option<&str>, is_active: Option<bool>) -> Result<CatserDivisaoDto, RepositoryError> {
        sqlx::query_as::<_, CatserDivisaoDto>(
            r#"UPDATE catser_divisoes SET secao_id = COALESCE($2, secao_id), name = COALESCE($3, name),
            is_active = COALESCE($4, is_active), updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(secao_id)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_verification_status(&self, id: Uuid, verification_status: &str) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE catser_divisoes SET verification_status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(verification_status)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM catser_divisoes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_grupos(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catser_groups WHERE divisao_id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, secao_id: Option<Uuid>, is_active: Option<bool>) -> Result<(Vec<CatserDivisaoWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let records = sqlx::query(
            r#"SELECT d.*, s.name AS secao_name,
                (SELECT COUNT(*) FROM catser_groups WHERE divisao_id = d.id) AS grupo_count
            FROM catser_divisoes d
            JOIN catser_secoes s ON d.secao_id = s.id
            WHERE ($1::TEXT IS NULL OR d.name ILIKE $1)
              AND ($2::UUID IS NULL OR d.secao_id = $2)
              AND ($3::BOOLEAN IS NULL OR d.is_active = $3)
            ORDER BY d.name LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(secao_id)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let divisoes = records.into_iter().map(|r| CatserDivisaoWithDetailsDto {
            id: r.get("id"),
            secao_id: r.get("secao_id"),
            secao_name: r.get("secao_name"),
            name: r.get("name"),
            is_active: r.get("is_active"),
            verification_status: r.get("verification_status"),
            grupo_count: r.get("grupo_count"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM catser_divisoes d
            WHERE ($1::TEXT IS NULL OR d.name ILIKE $1)
              AND ($2::UUID IS NULL OR d.secao_id = $2)
              AND ($3::BOOLEAN IS NULL OR d.is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(secao_id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((divisoes, total))
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
