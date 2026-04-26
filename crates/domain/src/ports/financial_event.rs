use crate::errors::RepositoryError;
use crate::models::financial_event::{CreateFinancialEventInput, FinancialEventDto, FinancialEventType};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FinancialEventRepositoryPort: Send + Sync {
    async fn create(&self, input: CreateFinancialEventInput) -> Result<FinancialEventDto, RepositoryError>;
    async fn list_by_invoice(
        &self,
        invoice_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FinancialEventDto>, i64), RepositoryError>;
    async fn list_by_supplier(
        &self,
        supplier_id: Uuid,
        event_type: Option<FinancialEventType>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FinancialEventDto>, i64), RepositoryError>;
}
