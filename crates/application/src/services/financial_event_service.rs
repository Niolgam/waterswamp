use crate::errors::ServiceError;
use domain::{
    models::financial_event::{CreateFinancialEventInput, FinancialEventDto, FinancialEventType},
    ports::financial_event::FinancialEventRepositoryPort,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

/// Publishes financial events to the event log (RF-028).
/// The `financial_events` table serves as the durable event store; downstream
/// systems poll it or subscribe via NOTIFY/LISTEN to react to events.
pub struct FinancialEventPublisher {
    repo: Arc<dyn FinancialEventRepositoryPort>,
}

impl FinancialEventPublisher {
    pub fn new(repo: Arc<dyn FinancialEventRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn publish_glosa_criada(
        &self,
        invoice_id: Uuid,
        invoice_adjustment_id: Uuid,
        supplier_id: Uuid,
        warehouse_id: Uuid,
        amount: Decimal,
        created_by: Uuid,
    ) -> Result<FinancialEventDto, ServiceError> {
        self.repo
            .create(CreateFinancialEventInput {
                event_type: FinancialEventType::GlosaCriada,
                invoice_id: Some(invoice_id),
                invoice_adjustment_id: Some(invoice_adjustment_id),
                supplier_id: Some(supplier_id),
                warehouse_id: Some(warehouse_id),
                amount: Some(amount),
                commitment_number: None,
                metadata: None,
                created_by: Some(created_by),
            })
            .await
            .map_err(ServiceError::from)
    }

    pub async fn publish_empenho_validado(
        &self,
        invoice_id: Uuid,
        commitment_number: &str,
        available_balance: Decimal,
        requested_amount: Decimal,
        created_by: Uuid,
    ) -> Result<FinancialEventDto, ServiceError> {
        let metadata = serde_json::json!({
            "available_balance": available_balance,
            "requested_amount": requested_amount,
        });
        self.repo
            .create(CreateFinancialEventInput {
                event_type: FinancialEventType::EmpenhoValidado,
                invoice_id: Some(invoice_id),
                invoice_adjustment_id: None,
                supplier_id: None,
                warehouse_id: None,
                amount: Some(available_balance),
                commitment_number: Some(commitment_number.to_string()),
                metadata: Some(metadata),
                created_by: Some(created_by),
            })
            .await
            .map_err(ServiceError::from)
    }

    pub async fn publish_empenho_insuficiente(
        &self,
        commitment_number: &str,
        available_balance: Decimal,
        requested_amount: Decimal,
    ) -> Result<FinancialEventDto, ServiceError> {
        let metadata = serde_json::json!({
            "available_balance": available_balance,
            "requested_amount": requested_amount,
        });
        self.repo
            .create(CreateFinancialEventInput {
                event_type: FinancialEventType::EmpenhoInsuficiente,
                invoice_id: None,
                invoice_adjustment_id: None,
                supplier_id: None,
                warehouse_id: None,
                amount: Some(requested_amount),
                commitment_number: Some(commitment_number.to_string()),
                metadata: Some(metadata),
                created_by: None,
            })
            .await
            .map_err(ServiceError::from)
    }
}
