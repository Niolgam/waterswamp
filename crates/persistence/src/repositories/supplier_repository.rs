use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::supplier::*,
    ports::supplier::*,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct SupplierRepository {
    pool: PgPool,
}

impl SupplierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SupplierRepositoryPort for SupplierRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SupplierDto>, RepositoryError> {
        sqlx::query_as::<_, SupplierDto>("SELECT * FROM suppliers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<SupplierWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, SupplierWithDetailsDto>(
            r#"SELECT s.id, s.supplier_type, s.legal_name, s.trade_name, s.document_number,
                      s.representative_name, s.address, s.neighborhood, s.is_international_neighborhood,
                      s.city_id, c.name AS city_name, st.abbreviation AS state_abbreviation,
                      s.zip_code, s.email, s.phone, s.is_active,
                      s.created_at, s.updated_at
               FROM suppliers s
               LEFT JOIN cities c ON c.id = s.city_id
               LEFT JOIN states st ON st.id = c.state_id
               WHERE s.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_document_number(&self, document_number: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM suppliers WHERE document_number = $1) AS exists")
            .bind(document_number)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn exists_by_document_number_excluding(&self, document_number: &str, id: Uuid) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM suppliers WHERE document_number = $1 AND id != $2) AS exists")
            .bind(document_number)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn create(
        &self,
        supplier_type: &SupplierType,
        legal_name: &str,
        trade_name: Option<&str>,
        document_number: &str,
        representative_name: Option<&str>,
        address: Option<&str>,
        neighborhood: Option<&str>,
        is_international_neighborhood: bool,
        city_id: Option<Uuid>,
        zip_code: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<SupplierDto, RepositoryError> {
        sqlx::query_as::<_, SupplierDto>(
            r#"INSERT INTO suppliers (supplier_type, legal_name, trade_name, document_number,
                                      representative_name, address, neighborhood, is_international_neighborhood,
                                      city_id, zip_code, email, phone, created_by, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
               RETURNING *"#,
        )
        .bind(supplier_type)
        .bind(legal_name)
        .bind(trade_name)
        .bind(document_number)
        .bind(representative_name)
        .bind(address)
        .bind(neighborhood)
        .bind(is_international_neighborhood)
        .bind(city_id)
        .bind(zip_code)
        .bind(email)
        .bind(phone)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        supplier_type: Option<&SupplierType>,
        legal_name: Option<&str>,
        trade_name: Option<&str>,
        document_number: Option<&str>,
        representative_name: Option<&str>,
        address: Option<&str>,
        neighborhood: Option<&str>,
        is_international_neighborhood: Option<bool>,
        city_id: Option<Uuid>,
        zip_code: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        is_active: Option<bool>,
        updated_by: Option<Uuid>,
    ) -> Result<SupplierDto, RepositoryError> {
        sqlx::query_as::<_, SupplierDto>(
            r#"UPDATE suppliers SET
                supplier_type = COALESCE($2, supplier_type),
                legal_name = COALESCE($3, legal_name),
                trade_name = COALESCE($4, trade_name),
                document_number = COALESCE($5, document_number),
                representative_name = COALESCE($6, representative_name),
                address = COALESCE($7, address),
                neighborhood = COALESCE($8, neighborhood),
                is_international_neighborhood = COALESCE($9, is_international_neighborhood),
                city_id = COALESCE($10, city_id),
                zip_code = COALESCE($11, zip_code),
                email = COALESCE($12, email),
                phone = COALESCE($13, phone),
                is_active = COALESCE($14, is_active),
                updated_by = COALESCE($15, updated_by)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(supplier_type)
        .bind(legal_name)
        .bind(trade_name)
        .bind(document_number)
        .bind(representative_name)
        .bind(address)
        .bind(neighborhood)
        .bind(is_international_neighborhood)
        .bind(city_id)
        .bind(zip_code)
        .bind(email)
        .bind(phone)
        .bind(is_active)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM suppliers WHERE id = $1")
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
        supplier_type: Option<SupplierType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<SupplierWithDetailsDto>, i64), RepositoryError> {
        let mut where_clauses = Vec::new();
        let mut param_index = 1u32;

        if search.is_some() {
            where_clauses.push(format!(
                "(s.legal_name ILIKE ${p} OR s.trade_name ILIKE ${p} OR s.document_number ILIKE ${p})",
                p = param_index
            ));
            param_index += 1;
        }
        if supplier_type.is_some() {
            where_clauses.push(format!("s.supplier_type = ${}", param_index));
            param_index += 1;
        }
        if is_active.is_some() {
            where_clauses.push(format!("s.is_active = ${}", param_index));
            param_index += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) AS total FROM suppliers s {}", where_sql);
        let list_sql = format!(
            r#"SELECT s.id, s.supplier_type, s.legal_name, s.trade_name, s.document_number,
                      s.representative_name, s.address, s.neighborhood, s.is_international_neighborhood,
                      s.city_id, c.name AS city_name, st.abbreviation AS state_abbreviation,
                      s.zip_code, s.email, s.phone, s.is_active,
                      s.created_at, s.updated_at
               FROM suppliers s
               LEFT JOIN cities c ON c.id = s.city_id
               LEFT JOIN states st ON st.id = c.state_id
               {}
               ORDER BY s.legal_name ASC
               LIMIT ${} OFFSET ${}"#,
            where_sql, param_index, param_index + 1
        );

        // Build count query
        let mut count_query = sqlx::query(&count_sql);
        let mut list_query = sqlx::query_as::<_, SupplierWithDetailsDto>(&list_sql);

        if let Some(ref s) = search {
            let pattern = format!("%{}%", s);
            count_query = count_query.bind(pattern.clone());
            list_query = list_query.bind(pattern);
        }
        if let Some(ref st) = supplier_type {
            count_query = count_query.bind(st);
            list_query = list_query.bind(st);
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
