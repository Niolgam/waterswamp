use std::sync::Arc;

use crate::errors::ServiceError;
use domain::models::{
    MaterialConsumptionReportDto, MostRequestedMaterialsReportDto, MovementAnalysisReportDto,
    StockValueDetailDto, StockValueReportDto,
};
use domain::ports::WarehouseReportsPort;
use uuid::Uuid;

/// Serviço para geração de relatórios do almoxarifado
pub struct WarehouseReportsService {
    reports_repo: Arc<dyn WarehouseReportsPort>,
}

impl WarehouseReportsService {
    pub fn new(reports_repo: Arc<dyn WarehouseReportsPort>) -> Self {
        Self { reports_repo }
    }

    /// Relatório de valor total de estoque por almoxarifado
    pub async fn get_stock_value_report(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Vec<StockValueReportDto>, ServiceError> {
        self.reports_repo
            .get_stock_value_report(warehouse_id)
            .await
            .map_err(|e| e.into())
    }

    /// Relatório detalhado de valor de estoque por material em um almoxarifado
    pub async fn get_stock_value_detail(
        &self,
        warehouse_id: Uuid,
        material_group_id: Option<Uuid>,
    ) -> Result<Vec<StockValueDetailDto>, ServiceError> {
        self.reports_repo
            .get_stock_value_detail(warehouse_id, material_group_id)
            .await
            .map_err(|e| e.into())
    }

    /// Relatório de consumo de materiais por período
    pub async fn get_material_consumption_report(
        &self,
        warehouse_id: Option<Uuid>,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        limit: Option<i64>,
    ) -> Result<Vec<MaterialConsumptionReportDto>, ServiceError> {
        let limit = limit.unwrap_or(50);

        if limit < 1 || limit > 100 {
            return Err(ServiceError::BadRequest(
                "Limite deve estar entre 1 e 100".to_string(),
            ));
        }

        if start_date > end_date {
            return Err(ServiceError::BadRequest(
                "Data inicial deve ser anterior à data final".to_string(),
            ));
        }

        self.reports_repo
            .get_material_consumption_report(warehouse_id, start_date, end_date, limit)
            .await
            .map_err(|e| e.into())
    }

    /// Relatório de materiais mais requisitados
    pub async fn get_most_requested_materials(
        &self,
        warehouse_id: Option<Uuid>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<MostRequestedMaterialsReportDto>, ServiceError> {
        let limit = limit.unwrap_or(50);

        if limit < 1 || limit > 100 {
            return Err(ServiceError::BadRequest(
                "Limite deve estar entre 1 e 100".to_string(),
            ));
        }

        if let (Some(start), Some(end)) = (start_date, end_date) {
            if start > end {
                return Err(ServiceError::BadRequest(
                    "Data inicial deve ser anterior à data final".to_string(),
                ));
            }
        }

        self.reports_repo
            .get_most_requested_materials(warehouse_id, start_date, end_date, limit)
            .await
            .map_err(|e| e.into())
    }

    /// Análise de movimentações por tipo e período
    pub async fn get_movement_analysis(
        &self,
        warehouse_id: Option<Uuid>,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<MovementAnalysisReportDto>, ServiceError> {
        if start_date > end_date {
            return Err(ServiceError::BadRequest(
                "Data inicial deve ser anterior à data final".to_string(),
            ));
        }

        self.reports_repo
            .get_movement_analysis(warehouse_id, start_date, end_date)
            .await
            .map_err(|e| e.into())
    }
}
