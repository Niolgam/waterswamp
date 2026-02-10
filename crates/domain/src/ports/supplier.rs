use crate::errors::RepositoryError;
use crate::models::supplier::*;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SupplierRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SupplierDto>, RepositoryError>;
    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<SupplierWithDetailsDto>, RepositoryError>;
    async fn exists_by_document_number(&self, document_number: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_document_number_excluding(&self, document_number: &str, id: Uuid) -> Result<bool, RepositoryError>;
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
    ) -> Result<SupplierDto, RepositoryError>;
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
    ) -> Result<SupplierDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        supplier_type: Option<SupplierType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<SupplierWithDetailsDto>, i64), RepositoryError>;
}
