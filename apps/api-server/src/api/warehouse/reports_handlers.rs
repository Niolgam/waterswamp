use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{extractors::current_user::CurrentUser, infra::{errors::AppError, state::AppState}};

// ============================
// Request/Query Contracts
// ============================

#[derive(Debug, Deserialize)]
pub struct StockValueReportQuery {
    pub warehouse_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct StockValueDetailQuery {
    pub warehouse_id: Uuid,
    pub material_group_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ConsumptionReportQuery {
    pub warehouse_id: Option<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct MostRequestedQuery {
    pub warehouse_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct MovementAnalysisQuery {
    pub warehouse_id: Option<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

fn default_limit() -> i64 {
    50
}

// ============================
// Handlers
// ============================

/// GET /api/admin/warehouse/reports/stock-value
/// Relatório de valor total de estoque por almoxarifado
pub async fn get_stock_value_report(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(query): Query<StockValueReportQuery>,
) -> Result<Json<Value>, AppError> {
    let report = state
        .warehouse_reports_service
        .get_stock_value_report(query.warehouse_id)
        .await?;

    Ok(Json(json!({
        "report": report,
        "message": "Relatório de valor de estoque gerado com sucesso"
    })))
}

/// GET /api/admin/warehouse/reports/stock-value/detail
/// Relatório detalhado de valor de estoque por material
pub async fn get_stock_value_detail(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(query): Query<StockValueDetailQuery>,
) -> Result<Json<Value>, AppError> {
    let report = state
        .warehouse_reports_service
        .get_stock_value_detail(query.warehouse_id, query.material_group_id)
        .await?;

    Ok(Json(json!({
        "report": report,
        "message": "Relatório detalhado de estoque gerado com sucesso"
    })))
}

/// GET /api/admin/warehouse/reports/consumption
/// Relatório de consumo de materiais por período
pub async fn get_consumption_report(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(query): Query<ConsumptionReportQuery>,
) -> Result<Json<Value>, AppError> {
    let report = state
        .warehouse_reports_service
        .get_material_consumption_report(
            query.warehouse_id,
            query.start_date,
            query.end_date,
            Some(query.limit),
        )
        .await?;

    Ok(Json(json!({
        "report": report,
        "period": {
            "start": query.start_date,
            "end": query.end_date
        },
        "message": "Relatório de consumo gerado com sucesso"
    })))
}

/// GET /api/admin/warehouse/reports/most-requested
/// Relatório de materiais mais requisitados
pub async fn get_most_requested_materials(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(query): Query<MostRequestedQuery>,
) -> Result<Json<Value>, AppError> {
    let report = state
        .warehouse_reports_service
        .get_most_requested_materials(
            query.warehouse_id,
            query.start_date,
            query.end_date,
            Some(query.limit),
        )
        .await?;

    Ok(Json(json!({
        "report": report,
        "period": {
            "start": query.start_date,
            "end": query.end_date
        },
        "message": "Relatório de materiais mais requisitados gerado com sucesso"
    })))
}

/// GET /api/admin/warehouse/reports/movement-analysis
/// Análise de movimentações por tipo e período
pub async fn get_movement_analysis(
    State(state): State<AppState>,
    _user: CurrentUser,
    Query(query): Query<MovementAnalysisQuery>,
) -> Result<Json<Value>, AppError> {
    let report = state
        .warehouse_reports_service
        .get_movement_analysis(query.warehouse_id, query.start_date, query.end_date)
        .await?;

    Ok(Json(json!({
        "report": report,
        "period": {
            "start": query.start_date,
            "end": query.end_date
        },
        "message": "Análise de movimentações gerada com sucesso"
    })))
}
