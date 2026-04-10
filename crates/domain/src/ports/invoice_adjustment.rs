use crate::errors::RepositoryError;
use crate::models::invoice_adjustment::*;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait InvoiceAdjustmentRepositoryPort: Send + Sync {
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<InvoiceAdjustmentDto>, RepositoryError>;

    async fn list_by_invoice(
        &self,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceAdjustmentWithItemsDto>, RepositoryError>;

    async fn create(
        &self,
        invoice_id: Uuid,
        reason: &str,
        created_by: Uuid,
    ) -> Result<InvoiceAdjustmentDto, RepositoryError>;

    async fn create_item(
        &self,
        adjustment_id: Uuid,
        invoice_item_id: Uuid,
        adjusted_quantity: rust_decimal::Decimal,
        adjusted_value: rust_decimal::Decimal,
        notes: Option<&str>,
    ) -> Result<InvoiceAdjustmentItemDto, RepositoryError>;
}
