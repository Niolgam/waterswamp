use std::sync::Arc;

use domain::{
    models::alert::*,
    ports::alert::StockAlertRepositoryPort,
};
use uuid::Uuid;

use crate::errors::ServiceError;

pub struct AlertService {
    repo: Arc<dyn StockAlertRepositoryPort>,
}

impl AlertService {
    pub fn new(repo: Arc<dyn StockAlertRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn create_alert(
        &self,
        input: CreateStockAlertInput,
    ) -> Result<StockAlertDto, ServiceError> {
        if input.title.trim().is_empty() {
            return Err(ServiceError::BadRequest("Alert title cannot be empty".into()));
        }
        let valid_severities = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
        if !valid_severities.contains(&input.severity.as_str()) {
            return Err(ServiceError::BadRequest(format!(
                "Invalid severity '{}'. Must be one of: LOW, MEDIUM, HIGH, CRITICAL",
                input.severity
            )));
        }
        self.repo.create(input).await.map_err(ServiceError::from)
    }

    pub async fn get_alert(&self, id: Uuid) -> Result<StockAlertDto, ServiceError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound(format!("Alert {} not found", id)))
    }

    pub async fn list_alerts(
        &self,
        warehouse_id: Option<Uuid>,
        status: Option<StockAlertStatus>,
        alert_type: Option<StockAlertType>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<StockAlertDto>, i64), ServiceError> {
        self.repo
            .list(warehouse_id, status, alert_type, limit, offset)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn acknowledge_alert(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<StockAlertDto, ServiceError> {
        let alert = self.get_alert(id).await?;
        if alert.status != StockAlertStatus::Open {
            return Err(ServiceError::BadRequest(format!(
                "Alert {} cannot be acknowledged in status {:?}",
                id, alert.status
            )));
        }
        self.repo
            .acknowledge(id, user_id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn resolve_alert(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<StockAlertDto, ServiceError> {
        let alert = self.get_alert(id).await?;
        if alert.status == StockAlertStatus::Resolved {
            return Err(ServiceError::BadRequest(format!(
                "Alert {} is already resolved",
                id
            )));
        }
        self.repo
            .resolve(id, user_id)
            .await
            .map_err(ServiceError::from)
    }

    /// Marks all alerts with expired SLA as SLA_BREACHED.
    /// Call this from a scheduled job or health-check endpoint.
    pub async fn process_sla_breaches(&self) -> Result<usize, ServiceError> {
        let overdue = self
            .repo
            .find_overdue_sla()
            .await
            .map_err(ServiceError::from)?;

        let count = overdue.len();
        for alert in overdue {
            let _ = self.repo.mark_sla_breached(alert.id).await;
        }
        Ok(count)
    }
}
