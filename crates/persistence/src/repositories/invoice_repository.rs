use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use domain::{
    errors::RepositoryError,
    models::invoice::*,
    ports::invoice::*,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

pub struct InvoiceRepository {
    pool: PgPool,
}

impl InvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvoiceRepositoryPort for InvoiceRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InvoiceDto>, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>("SELECT * FROM invoices WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<InvoiceWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, InvoiceWithDetailsDto>(
            r#"SELECT i.id, i.invoice_number, i.series, i.access_key, i.issue_date,
                      i.supplier_id, s.legal_name AS supplier_name,
                      i.warehouse_id, w.name AS warehouse_name,
                      i.total_products, i.total_freight, i.total_discount, i.total_value,
                      i.status,
                      i.received_at, i.received_by,
                      i.checked_at, i.checked_by,
                      i.posted_at, i.posted_by,
                      i.commitment_number, i.purchase_order_number, i.contract_number,
                      i.notes, i.rejection_reason, i.pdf_url, i.xml_url,
                      i.created_at, i.updated_at
               FROM invoices i
               LEFT JOIN suppliers s ON s.id = i.supplier_id
               LEFT JOIN warehouses w ON w.id = i.warehouse_id
               WHERE i.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn exists_by_access_key(&self, access_key: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM invoices WHERE access_key = $1) AS exists",
        )
        .bind(access_key)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn exists_by_access_key_excluding(
        &self,
        access_key: &str,
        id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM invoices WHERE access_key = $1 AND id != $2) AS exists",
        )
        .bind(access_key)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.get::<bool, _>("exists"))
    }

    async fn create(
        &self,
        invoice_number: &str,
        series: Option<&str>,
        access_key: Option<&str>,
        issue_date: DateTime<Utc>,
        supplier_id: Uuid,
        warehouse_id: Uuid,
        total_freight: Decimal,
        total_discount: Decimal,
        commitment_number: Option<&str>,
        purchase_order_number: Option<&str>,
        contract_number: Option<&str>,
        notes: Option<&str>,
        pdf_url: Option<&str>,
        xml_url: Option<&str>,
        _created_by: Option<Uuid>,
    ) -> Result<InvoiceDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>(
            r#"INSERT INTO invoices (
                invoice_number, series, access_key, issue_date,
                supplier_id, warehouse_id,
                total_freight, total_discount,
                commitment_number, purchase_order_number, contract_number,
                notes, pdf_url, xml_url
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(invoice_number)
        .bind(series)
        .bind(access_key)
        .bind(issue_date)
        .bind(supplier_id)
        .bind(warehouse_id)
        .bind(total_freight)
        .bind(total_discount)
        .bind(commitment_number)
        .bind(purchase_order_number)
        .bind(contract_number)
        .bind(notes)
        .bind(pdf_url)
        .bind(xml_url)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        invoice_number: Option<&str>,
        series: Option<&str>,
        access_key: Option<&str>,
        issue_date: Option<DateTime<Utc>>,
        supplier_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
        total_freight: Option<Decimal>,
        total_discount: Option<Decimal>,
        commitment_number: Option<&str>,
        purchase_order_number: Option<&str>,
        contract_number: Option<&str>,
        notes: Option<&str>,
        pdf_url: Option<&str>,
        xml_url: Option<&str>,
        _updated_by: Option<Uuid>,
    ) -> Result<InvoiceDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>(
            r#"UPDATE invoices SET
                invoice_number = COALESCE($2, invoice_number),
                series = COALESCE($3, series),
                access_key = COALESCE($4, access_key),
                issue_date = COALESCE($5, issue_date),
                supplier_id = COALESCE($6, supplier_id),
                warehouse_id = COALESCE($7, warehouse_id),
                total_freight = COALESCE($8, total_freight),
                total_discount = COALESCE($9, total_discount),
                commitment_number = COALESCE($10, commitment_number),
                purchase_order_number = COALESCE($11, purchase_order_number),
                contract_number = COALESCE($12, contract_number),
                notes = COALESCE($13, notes),
                pdf_url = COALESCE($14, pdf_url),
                xml_url = COALESCE($15, xml_url)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(invoice_number)
        .bind(series)
        .bind(access_key)
        .bind(issue_date)
        .bind(supplier_id)
        .bind(warehouse_id)
        .bind(total_freight)
        .bind(total_discount)
        .bind(commitment_number)
        .bind(purchase_order_number)
        .bind(contract_number)
        .bind(notes)
        .bind(pdf_url)
        .bind(xml_url)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn transition_to_checking(
        &self,
        id: Uuid,
        received_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>(
            r#"UPDATE invoices SET
                status = 'CHECKING',
                received_at = NOW(),
                received_by = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(received_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn transition_to_checked(
        &self,
        id: Uuid,
        checked_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>(
            r#"UPDATE invoices SET
                status = 'CHECKED',
                checked_at = NOW(),
                checked_by = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(checked_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn transition_to_posted(
        &self,
        id: Uuid,
        posted_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError> {
        // The trigger fn_auto_post_invoice() will automatically create stock movements
        sqlx::query_as::<_, InvoiceDto>(
            r#"UPDATE invoices SET
                status = 'POSTED',
                posted_at = NOW(),
                posted_by = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(posted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn transition_to_rejected(
        &self,
        id: Uuid,
        rejection_reason: &str,
        rejected_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>(
            r#"UPDATE invoices SET
                status = 'REJECTED',
                rejection_reason = $2,
                checked_by = $3,
                checked_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(rejection_reason)
        .bind(rejected_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn transition_to_cancelled(
        &self,
        id: Uuid,
        _cancelled_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError> {
        sqlx::query_as::<_, InvoiceDto>(
            r#"UPDATE invoices SET status = 'CANCELLED' WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM invoices WHERE id = $1")
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
        status: Option<InvoiceStatus>,
        supplier_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<InvoiceWithDetailsDto>, i64), RepositoryError> {
        let mut where_clauses = Vec::new();
        let mut param_index = 1u32;

        if search.is_some() {
            where_clauses.push(format!(
                "(i.invoice_number ILIKE ${p} OR i.access_key ILIKE ${p} OR s.legal_name ILIKE ${p})",
                p = param_index
            ));
            param_index += 1;
        }
        if status.is_some() {
            where_clauses.push(format!("i.status = ${}", param_index));
            param_index += 1;
        }
        if supplier_id.is_some() {
            where_clauses.push(format!("i.supplier_id = ${}", param_index));
            param_index += 1;
        }
        if warehouse_id.is_some() {
            where_clauses.push(format!("i.warehouse_id = ${}", param_index));
            param_index += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!(
            r#"SELECT COUNT(*) AS total FROM invoices i
               LEFT JOIN suppliers s ON s.id = i.supplier_id
               {}"#,
            where_sql
        );
        let list_sql = format!(
            r#"SELECT i.id, i.invoice_number, i.series, i.access_key, i.issue_date,
                      i.supplier_id, s.legal_name AS supplier_name,
                      i.warehouse_id, w.name AS warehouse_name,
                      i.total_products, i.total_freight, i.total_discount, i.total_value,
                      i.status,
                      i.received_at, i.received_by,
                      i.checked_at, i.checked_by,
                      i.posted_at, i.posted_by,
                      i.commitment_number, i.purchase_order_number, i.contract_number,
                      i.notes, i.rejection_reason, i.pdf_url, i.xml_url,
                      i.created_at, i.updated_at
               FROM invoices i
               LEFT JOIN suppliers s ON s.id = i.supplier_id
               LEFT JOIN warehouses w ON w.id = i.warehouse_id
               {}
               ORDER BY i.issue_date DESC
               LIMIT ${} OFFSET ${}"#,
            where_sql, param_index, param_index + 1
        );

        let mut count_query = sqlx::query(&count_sql);
        let mut list_query = sqlx::query_as::<_, InvoiceWithDetailsDto>(&list_sql);

        if let Some(ref s) = search {
            let pattern = format!("%{}%", s);
            count_query = count_query.bind(pattern.clone());
            list_query = list_query.bind(pattern);
        }
        if let Some(ref st) = status {
            count_query = count_query.bind(st);
            list_query = list_query.bind(st);
        }
        if let Some(sid) = supplier_id {
            count_query = count_query.bind(sid);
            list_query = list_query.bind(sid);
        }
        if let Some(wid) = warehouse_id {
            count_query = count_query.bind(wid);
            list_query = list_query.bind(wid);
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

// ============================
// Invoice Item Repository
// ============================

pub struct InvoiceItemRepository {
    pool: PgPool,
}

impl InvoiceItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvoiceItemRepositoryPort for InvoiceItemRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InvoiceItemDto>, RepositoryError> {
        sqlx::query_as::<_, InvoiceItemDto>("SELECT * FROM invoice_items WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn list_by_invoice(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceItemWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, InvoiceItemWithDetailsDto>(
            r#"SELECT ii.id, ii.invoice_id,
                      ii.catalog_item_id, ci.name AS catalog_item_name,
                      ii.unit_conversion_id,
                      ii.unit_raw_id, u.name AS unit_raw_name, u.symbol AS unit_raw_symbol,
                      COALESCE(pdm.material_classification, 'STOCKABLE'::material_classification_enum) AS material_classification,
                      ii.quantity_raw, ii.unit_value_raw, ii.total_value,
                      ii.conversion_factor, ii.quantity_base, ii.unit_value_base,
                      ii.ncm, ii.cfop, ii.cest,
                      ii.batch_number, ii.manufacturing_date, ii.expiration_date,
                      ii.created_at
               FROM invoice_items ii
               LEFT JOIN catmat_items ci ON ci.id = ii.catalog_item_id
               LEFT JOIN catmat_pdms pdm ON pdm.id = ci.pdm_id
               LEFT JOIN units_of_measure u ON u.id = ii.unit_raw_id
               WHERE ii.invoice_id = $1
               ORDER BY ii.created_at ASC"#,
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn create(
        &self,
        invoice_id: Uuid,
        catalog_item_id: Uuid,
        unit_conversion_id: Option<Uuid>,
        unit_raw_id: Uuid,
        quantity_raw: Decimal,
        unit_value_raw: Decimal,
        conversion_factor: Decimal,
        ncm: Option<&str>,
        cfop: Option<&str>,
        cest: Option<&str>,
        batch_number: Option<&str>,
        manufacturing_date: Option<NaiveDate>,
        expiration_date: Option<NaiveDate>,
    ) -> Result<InvoiceItemDto, RepositoryError> {
        let total_value = quantity_raw * unit_value_raw;
        sqlx::query_as::<_, InvoiceItemDto>(
            r#"INSERT INTO invoice_items (
                invoice_id, catalog_item_id, unit_conversion_id, unit_raw_id,
                quantity_raw, unit_value_raw, total_value, conversion_factor,
                ncm, cfop, cest,
                batch_number, manufacturing_date, expiration_date
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(invoice_id)
        .bind(catalog_item_id)
        .bind(unit_conversion_id)
        .bind(unit_raw_id)
        .bind(quantity_raw)
        .bind(unit_value_raw)
        .bind(total_value)
        .bind(conversion_factor)
        .bind(ncm)
        .bind(cfop)
        .bind(cest)
        .bind(batch_number)
        .bind(manufacturing_date)
        .bind(expiration_date)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM invoice_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_by_invoice(&self, invoice_id: Uuid) -> Result<u64, RepositoryError> {
        let result = sqlx::query("DELETE FROM invoice_items WHERE invoice_id = $1")
            .bind(invoice_id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected())
    }
}
