use async_trait::async_trait;
use domain::errors::RepositoryError;
use domain::models::{BudgetClassificationDto, BudgetClassificationWithParentDto};
use domain::ports::BudgetClassificationRepositoryPort;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct BudgetClassificationRepository {
    pool: PgPool,
}

impl BudgetClassificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_err(e: sqlx::Error) -> RepositoryError {
        if let Some(db_err) = e.as_database_error() {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                }
                if code == "23503" {
                    return RepositoryError::Database(
                        "Foreign key constraint violation".to_string(),
                    );
                }
            }
        }
        RepositoryError::Database(e.to_string())
    }
}

#[async_trait]
impl BudgetClassificationRepositoryPort for BudgetClassificationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BudgetClassificationDto>, RepositoryError> {
        sqlx::query_as::<_, BudgetClassificationDto>(
            r#"
            SELECT id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
            FROM budget_classifications
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_with_parent_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<BudgetClassificationWithParentDto>, RepositoryError> {
        sqlx::query_as::<_, BudgetClassificationWithParentDto>(
            r#"
            SELECT
                bc.id, bc.parent_id, bc.code_part, bc.full_code, bc.name, bc.level, bc.is_active,
                p.name as parent_name, p.full_code as parent_full_code,
                bc.created_at, bc.updated_at
            FROM budget_classifications bc
            LEFT JOIN budget_classifications p ON bc.parent_id = p.id
            WHERE bc.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn find_by_full_code(
        &self,
        full_code: &str,
    ) -> Result<Option<BudgetClassificationDto>, RepositoryError> {
        sqlx::query_as::<_, BudgetClassificationDto>(
            r#"
            SELECT id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
            FROM budget_classifications
            WHERE full_code = $1
            "#,
        )
        .bind(full_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn exists_by_full_code(&self, full_code: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM budget_classifications WHERE full_code = $1"
        )
        .bind(full_code)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn exists_by_full_code_excluding(
        &self,
        full_code: &str,
        exclude_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM budget_classifications WHERE full_code = $1 AND id != $2"
        )
        .bind(full_code)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        parent_id: Option<Uuid>,
        code_part: &str,
        name: &str,
        is_active: bool,
    ) -> Result<BudgetClassificationDto, RepositoryError> {
        sqlx::query_as::<_, BudgetClassificationDto>(
            r#"
            INSERT INTO budget_classifications (parent_id, code_part, name, is_active)
            VALUES ($1, $2, $3, $4)
            RETURNING id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
            "#,
        )
        .bind(parent_id)
        .bind(code_part)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(Self::map_err)
    }

    async fn update(
        &self,
        id: Uuid,
        parent_id: Option<Option<Uuid>>,
        code_part: Option<&str>,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<BudgetClassificationDto, RepositoryError> {
        let mut query_parts = vec![];
        let mut bind_index = 1;

        if parent_id.is_some() {
            query_parts.push(format!("parent_id = ${}", bind_index));
            bind_index += 1;
        }
        if code_part.is_some() {
            query_parts.push(format!("code_part = ${}", bind_index));
            bind_index += 1;
        }
        if name.is_some() {
            query_parts.push(format!("name = ${}", bind_index));
            bind_index += 1;
        }
        if is_active.is_some() {
            query_parts.push(format!("is_active = ${}", bind_index));
            bind_index += 1;
        }

        if query_parts.is_empty() {
            return self.find_by_id(id).await?.ok_or(RepositoryError::NotFound);
        }

        let query_str = format!(
            r#"
            UPDATE budget_classifications
            SET {}
            WHERE id = ${}
            RETURNING id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
            "#,
            query_parts.join(", "),
            bind_index
        );

        let mut query = sqlx::query_as::<_, BudgetClassificationDto>(&query_str);

        if let Some(parent_id_val) = parent_id {
            query = query.bind(parent_id_val);
        }
        if let Some(code_part_val) = code_part {
            query = query.bind(code_part_val);
        }
        if let Some(name_val) = name {
            query = query.bind(name_val);
        }
        if let Some(is_active_val) = is_active {
            query = query.bind(is_active_val);
        }
        query = query.bind(id);

        query.fetch_one(&self.pool).await.map_err(Self::map_err)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM budget_classifications WHERE id = $1")
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
        level: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<(Vec<BudgetClassificationWithParentDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let mut conditions = Vec::new();
        let mut bind_index = 1;

        if search_pattern.is_some() {
            conditions.push(format!(
                "(bc.name ILIKE ${} OR bc.full_code ILIKE ${})",
                bind_index, bind_index
            ));
            bind_index += 1;
        }
        if parent_id.is_some() {
            conditions.push(format!("bc.parent_id = ${}", bind_index));
            bind_index += 1;
        }
        if level.is_some() {
            conditions.push(format!("bc.level = ${}", bind_index));
            bind_index += 1;
        }
        if is_active.is_some() {
            conditions.push(format!("bc.is_active = ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            r#"
            SELECT
                bc.id, bc.parent_id, bc.code_part, bc.full_code, bc.name, bc.level, bc.is_active,
                p.name as parent_name, p.full_code as parent_full_code,
                bc.created_at, bc.updated_at
            FROM budget_classifications bc
            LEFT JOIN budget_classifications p ON bc.parent_id = p.id
            {}
            ORDER BY bc.full_code LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut query = sqlx::query_as::<_, BudgetClassificationWithParentDto>(&query_str);

        if let Some(ref pattern) = search_pattern {
            query = query.bind(pattern);
        }
        if let Some(parent_id_val) = parent_id {
            query = query.bind(parent_id_val);
        }
        if let Some(level_val) = level {
            query = query.bind(level_val);
        }
        if let Some(is_active_val) = is_active {
            query = query.bind(is_active_val);
        }
        query = query.bind(limit).bind(offset);

        let items = query.fetch_all(&self.pool).await.map_err(Self::map_err)?;

        // Count query
        let count_query_str = format!(
            "SELECT COUNT(*) FROM budget_classifications bc LEFT JOIN budget_classifications p ON bc.parent_id = p.id {}",
            where_clause
        );

        let mut count_query = sqlx::query_scalar(&count_query_str);

        if let Some(ref pattern) = search_pattern {
            count_query = count_query.bind(pattern);
        }
        if let Some(parent_id_val) = parent_id {
            count_query = count_query.bind(parent_id_val);
        }
        if let Some(level_val) = level {
            count_query = count_query.bind(level_val);
        }
        if let Some(is_active_val) = is_active {
            count_query = count_query.bind(is_active_val);
        }

        let total: i64 = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(Self::map_err)?;

        Ok((items, total))
    }

    async fn find_children(&self, parent_id: Option<Uuid>) -> Result<Vec<BudgetClassificationDto>, RepositoryError> {
        let query = if parent_id.is_some() {
            sqlx::query_as::<_, BudgetClassificationDto>(
                r#"
                SELECT id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
                FROM budget_classifications
                WHERE parent_id = $1
                ORDER BY full_code
                "#,
            )
            .bind(parent_id)
        } else {
            sqlx::query_as::<_, BudgetClassificationDto>(
                r#"
                SELECT id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
                FROM budget_classifications
                WHERE parent_id IS NULL
                ORDER BY full_code
                "#,
            )
        };

        query.fetch_all(&self.pool).await.map_err(Self::map_err)
    }

    async fn find_by_level(&self, level: i32) -> Result<Vec<BudgetClassificationDto>, RepositoryError> {
        sqlx::query_as::<_, BudgetClassificationDto>(
            r#"
            SELECT id, parent_id, code_part, full_code, name, level, is_active, created_at, updated_at
            FROM budget_classifications
            WHERE level = $1
            ORDER BY full_code
            "#,
        )
        .bind(level)
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)
    }
}
