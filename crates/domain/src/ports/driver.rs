use crate::errors::RepositoryError;
use crate::models::driver::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[async_trait]
pub trait DriverRepositoryPort: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DriverDto>, RepositoryError>;
    async fn exists_by_cpf(&self, cpf: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_cpf_excluding(&self, cpf: &str, id: Uuid) -> Result<bool, RepositoryError>;
    async fn exists_by_cnh(&self, cnh_number: &str) -> Result<bool, RepositoryError>;
    async fn exists_by_cnh_excluding(&self, cnh_number: &str, id: Uuid) -> Result<bool, RepositoryError>;
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
    ) -> Result<DriverDto, RepositoryError>;
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
    ) -> Result<DriverDto, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        driver_type: Option<DriverType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<DriverDto>, i64), RepositoryError>;
}
