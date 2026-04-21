use crate::errors::RepositoryError;
use crate::models::maintenance::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait MaintenanceOrderRepositoryPort: Send + Sync {
    async fn create(
        &self,
        vehicle_id: Uuid,
        tipo: MaintenanceOrderType,
        titulo: &str,
        descricao: Option<&str>,
        fornecedor_id: Option<Uuid>,
        data_abertura: NaiveDate,
        data_prevista_conclusao: Option<NaiveDate>,
        km_abertura: Option<i64>,
        custo_previsto: Option<Decimal>,
        numero_os_externo: Option<&str>,
        documento_sei: Option<&str>,
        incident_id: Option<Uuid>,
        notas: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderDto, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<MaintenanceOrderDto>, RepositoryError>;

    async fn advance_status(
        &self,
        id: Uuid,
        new_status: MaintenanceOrderStatus,
        custo_real: Option<Decimal>,
        data_conclusao: Option<NaiveDate>,
        notas: Option<&str>,
        motivo_cancelamento: Option<&str>,
        concluido_por: Option<Uuid>,
        cancelado_por: Option<Uuid>,
        version: i32,
    ) -> Result<MaintenanceOrderDto, RepositoryError>;

    async fn list(
        &self,
        vehicle_id: Option<Uuid>,
        status: Option<MaintenanceOrderStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MaintenanceOrderDto>, i64), RepositoryError>;

    async fn cost_summary(
        &self,
        vehicle_id: Uuid,
    ) -> Result<MaintenanceCostSummaryDto, RepositoryError>;

    // Items
    async fn add_item(
        &self,
        order_id: Uuid,
        service_id: Option<Uuid>,
        descricao: &str,
        quantidade: Decimal,
        custo_unitario: Option<Decimal>,
        created_by: Option<Uuid>,
    ) -> Result<MaintenanceOrderItemDto, RepositoryError>;

    async fn list_items(
        &self,
        order_id: Uuid,
    ) -> Result<Vec<MaintenanceOrderItemDto>, RepositoryError>;
}
