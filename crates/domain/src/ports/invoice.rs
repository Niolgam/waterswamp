use crate::errors::RepositoryError;
use crate::models::invoice::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait InvoiceRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InvoiceDto>, RepositoryError>;
    async fn find_with_details_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<InvoiceWithDetailsDto>, RepositoryError>;
    async fn exists_by_access_key(&self, access_key: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_access_key_excluding(
        &self,
        access_key: &str,
        id: Uuid,
    ) -> Result<bool, RepositoryError>;

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
        created_by: Option<Uuid>,
    ) -> Result<InvoiceDto, RepositoryError>;

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
        updated_by: Option<Uuid>,
    ) -> Result<InvoiceDto, RepositoryError>;

    async fn transition_to_checking(
        &self,
        id: Uuid,
        received_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError>;

    async fn transition_to_checked(
        &self,
        id: Uuid,
        checked_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError>;

    async fn transition_to_posted(
        &self,
        id: Uuid,
        posted_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError>;

    async fn transition_to_rejected(
        &self,
        id: Uuid,
        rejection_reason: &str,
        rejected_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError>;

    async fn transition_to_cancelled(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
    ) -> Result<InvoiceDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        status: Option<InvoiceStatus>,
        supplier_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<InvoiceWithDetailsDto>, i64), RepositoryError>;
}

#[async_trait]
pub trait InvoiceItemRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InvoiceItemDto>, RepositoryError>;
    async fn list_by_invoice(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceItemWithDetailsDto>, RepositoryError>;

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
        manufacturing_date: Option<chrono::NaiveDate>,
        expiration_date: Option<chrono::NaiveDate>,
    ) -> Result<InvoiceItemDto, RepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn delete_by_invoice(&self, invoice_id: Uuid) -> Result<u64, RepositoryError>;
}
