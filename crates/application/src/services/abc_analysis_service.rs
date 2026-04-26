use std::sync::Arc;

use chrono::Utc;
use domain::{
    models::abc_analysis::*,
    ports::abc_analysis::AbcAnalysisRepositoryPort,
};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::errors::ServiceError;

pub struct AbcAnalysisService {
    repo: Arc<dyn AbcAnalysisRepositoryPort>,
    /// Cumulative % threshold to classify as A (e.g. 0.80)
    threshold_a: Decimal,
    /// Cumulative % threshold to classify as B (e.g. 0.95, remainder is C)
    threshold_b: Decimal,
}

impl AbcAnalysisService {
    pub fn new(repo: Arc<dyn AbcAnalysisRepositoryPort>) -> Self {
        Self {
            repo,
            threshold_a: Decimal::new(80, 2),  // 0.80
            threshold_b: Decimal::new(95, 2),  // 0.95
        }
    }

    pub fn with_thresholds(mut self, threshold_a: Decimal, threshold_b: Decimal) -> Self {
        self.threshold_a = threshold_a;
        self.threshold_b = threshold_b;
        self
    }

    /// Run ABC analysis for the given warehouse (or all warehouses if None).
    /// Returns a summary of the analysis results.
    pub async fn run_analysis(
        &self,
        input: RunAbcInput,
    ) -> Result<AbcSummary, ServiceError> {
        let stock_values = self
            .repo
            .get_stock_values(input.warehouse_id)
            .await
            .map_err(ServiceError::from)?;

        if stock_values.is_empty() {
            return Err(ServiceError::BadRequest(
                "No stock data found to perform ABC analysis".into(),
            ));
        }

        let grand_total: Decimal = stock_values.iter().map(|(_, v)| *v).sum();
        if grand_total.is_zero() {
            return Err(ServiceError::BadRequest(
                "Total stock value is zero; cannot perform ABC analysis".into(),
            ));
        }

        let run_at = Utc::now();
        let mut cumulative = Decimal::ZERO;
        let mut results: Vec<AbcAnalysisResultDto> = Vec::with_capacity(stock_values.len());

        let mut class_a = (0i64, Decimal::ZERO);
        let mut class_b = (0i64, Decimal::ZERO);
        let mut class_c = (0i64, Decimal::ZERO);

        for (rank, (item_id, value)) in stock_values.iter().enumerate() {
            cumulative += value;
            let cum_pct = cumulative / grand_total;

            let classification = if cum_pct <= self.threshold_a {
                AbcClassification::A
            } else if cum_pct <= self.threshold_b {
                AbcClassification::B
            } else {
                AbcClassification::C
            };

            match classification {
                AbcClassification::A => { class_a.0 += 1; class_a.1 += value; }
                AbcClassification::B => { class_b.0 += 1; class_b.1 += value; }
                AbcClassification::C => { class_c.0 += 1; class_c.1 += value; }
            }

            results.push(AbcAnalysisResultDto {
                id: Uuid::new_v4(),
                run_at,
                warehouse_id: input.warehouse_id,
                catalog_item_id: *item_id,
                classification,
                total_value: *value,
                cumulative_percentage: cum_pct,
                rank_position: (rank + 1) as i32,
                created_at: run_at,
            });
        }

        self.repo
            .save_results(run_at, input.warehouse_id, results)
            .await
            .map_err(ServiceError::from)?;

        Ok(AbcSummary {
            warehouse_id: input.warehouse_id,
            run_at,
            total_items: stock_values.len() as i64,
            class_a_count: class_a.0,
            class_b_count: class_b.0,
            class_c_count: class_c.0,
            class_a_value: class_a.1,
            class_b_value: class_b.1,
            class_c_value: class_c.1,
        })
    }

    pub async fn get_latest_results(
        &self,
        warehouse_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AbcAnalysisResultDto>, i64), ServiceError> {
        self.repo
            .get_latest_results(warehouse_id, limit, offset)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_latest_run_at(
        &self,
        warehouse_id: Option<Uuid>,
    ) -> Result<Option<chrono::DateTime<Utc>>, ServiceError> {
        self.repo
            .get_latest_run_at(warehouse_id)
            .await
            .map_err(ServiceError::from)
    }
}
