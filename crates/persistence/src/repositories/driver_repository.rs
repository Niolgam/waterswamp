use async_trait::async_trait;
use chrono::NaiveDate;
use domain::{
    errors::RepositoryError,
    models::driver::*,
    ports::driver::*,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct DriverRepository {
    pool: PgPool,
}

impl DriverRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriverRepositoryPort for DriverRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DriverDto>, RepositoryError> {
        sqlx::query_as::<_, DriverDto>("SELECT * FROM drivers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_cpf(&self, cpf: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM drivers WHERE cpf = $1) AS exists")
            .bind(cpf)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn exists_by_cpf_excluding(&self, cpf: &str, id: Uuid) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM drivers WHERE cpf = $1 AND id != $2) AS exists")
            .bind(cpf)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn exists_by_cnh(&self, cnh_number: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM drivers WHERE cnh_number = $1) AS exists")
            .bind(cnh_number)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn exists_by_cnh_excluding(&self, cnh_number: &str, id: Uuid) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM drivers WHERE cnh_number = $1 AND id != $2) AS exists")
            .bind(cnh_number)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn create(
        &self,
        driver_type: &DriverType,
        full_name: &str,
        cpf: &str,
        cnh_number: &str,
        cnh_category: &str,
        cnh_expiration: NaiveDate,
        phone: Option<&str>,
        email: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<DriverDto, RepositoryError> {
        sqlx::query_as::<_, DriverDto>(
            r#"INSERT INTO drivers (driver_type, full_name, cpf, cnh_number, cnh_category,
                                    cnh_expiration, phone, email, created_by, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
               RETURNING *"#,
        )
        .bind(driver_type)
        .bind(full_name)
        .bind(cpf)
        .bind(cnh_number)
        .bind(cnh_category)
        .bind(cnh_expiration)
        .bind(phone)
        .bind(email)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        driver_type: Option<&DriverType>,
        full_name: Option<&str>,
        cpf: Option<&str>,
        cnh_number: Option<&str>,
        cnh_category: Option<&str>,
        cnh_expiration: Option<NaiveDate>,
        phone: Option<&str>,
        email: Option<&str>,
        is_active: Option<bool>,
        updated_by: Option<Uuid>,
    ) -> Result<DriverDto, RepositoryError> {
        sqlx::query_as::<_, DriverDto>(
            r#"UPDATE drivers SET
                driver_type = COALESCE($2, driver_type),
                full_name = COALESCE($3, full_name),
                cpf = COALESCE($4, cpf),
                cnh_number = COALESCE($5, cnh_number),
                cnh_category = COALESCE($6, cnh_category),
                cnh_expiration = COALESCE($7, cnh_expiration),
                phone = COALESCE($8, phone),
                email = COALESCE($9, email),
                is_active = COALESCE($10, is_active),
                updated_by = COALESCE($11, updated_by)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(driver_type)
        .bind(full_name)
        .bind(cpf)
        .bind(cnh_number)
        .bind(cnh_category)
        .bind(cnh_expiration)
        .bind(phone)
        .bind(email)
        .bind(is_active)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM drivers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        driver_type: Option<DriverType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<DriverDto>, i64), RepositoryError> {
        let mut where_clauses = Vec::new();
        let mut param_index = 1u32;

        if search.is_some() {
            where_clauses.push(format!(
                "(d.full_name ILIKE ${p} OR d.cpf ILIKE ${p} OR d.cnh_number ILIKE ${p})",
                p = param_index
            ));
            param_index += 1;
        }
        if driver_type.is_some() {
            where_clauses.push(format!("d.driver_type = ${}", param_index));
            param_index += 1;
        }
        if is_active.is_some() {
            where_clauses.push(format!("d.is_active = ${}", param_index));
            param_index += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) AS total FROM drivers d {}", where_sql);
        let list_sql = format!(
            r#"SELECT d.*
               FROM drivers d
               {}
               ORDER BY d.full_name ASC
               LIMIT ${} OFFSET ${}"#,
            where_sql, param_index, param_index + 1
        );

        let mut count_query = sqlx::query(&count_sql);
        let mut list_query = sqlx::query_as::<_, DriverDto>(&list_sql);

        if let Some(ref s) = search {
            let pattern = format!("%{}%", s);
            count_query = count_query.bind(pattern.clone());
            list_query = list_query.bind(pattern);
        }
        if let Some(ref dt) = driver_type {
            count_query = count_query.bind(dt);
            list_query = list_query.bind(dt);
        }
        if let Some(active) = is_active {
            count_query = count_query.bind(active);
            list_query = list_query.bind(active);
        }

        count_query = count_query.bind(limit);
        list_query = list_query.bind(limit).bind(offset);

        let total: i64 = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?
            .get("total");

        let items = list_query
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;

        Ok((items, total))
    }
}
