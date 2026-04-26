use super::contracts::*;
use crate::extractors::current_user::CurrentUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use domain::models::invoice_adjustment::{
    CreateInvoiceAdjustmentPayload, InvoiceAdjustmentWithItemsDto,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct InvoiceListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub search: Option<String>,
    pub status: Option<InvoiceStatus>,
    pub supplier_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
}

fn default_limit() -> i64 {
    50
}

pub async fn create_invoice(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateInvoicePayload>,
) -> Result<(StatusCode, Json<InvoiceWithDetailsDto>), (StatusCode, String)> {
    state
        .invoice_service
        .create_invoice(payload, Some(user.id))
        .await
        .map(|i| (StatusCode::CREATED, Json(i)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_invoice(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .get_invoice(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn get_invoice_items(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceItemsListResponse>, (StatusCode, String)> {
    state
        .invoice_service
        .get_invoice_items(id)
        .await
        .map(|items| Json(InvoiceItemsListResponse { items }))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn update_invoice(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInvoicePayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .update_invoice(id, payload, Some(user.id))
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn start_checking(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<StartCheckingPayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .start_checking(id, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn finish_checking(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<CheckInvoicePayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .finish_checking(id, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn post_invoice(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<PostInvoicePayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .post_invoice(id, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn reject_invoice(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectInvoicePayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .reject_invoice(id, payload, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn cancel_invoice(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<CancelInvoicePayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .cancel_invoice(id, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

/// RN-008: Lançamento compensatório — reverte NF POSTED dentro de 24h do lançamento.
pub async fn compensatory_reversal(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompensatoryReversalPayload>,
) -> Result<Json<InvoiceWithDetailsDto>, (StatusCode, String)> {
    state
        .invoice_service
        .compensatory_reversal(id, payload, user.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn delete_invoice(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .invoice_service
        .delete_invoice(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn list_invoices(
    _user: CurrentUser,
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<InvoicesListResponse>, (StatusCode, String)> {
    state
        .invoice_service
        .list_invoices(
            query.limit,
            query.offset,
            query.search,
            query.status,
            query.supplier_id,
            query.warehouse_id,
        )
        .await
        .map(|(data, total)| {
            Json(InvoicesListResponse {
                data,
                total,
                limit: query.limit,
                offset: query.offset,
            })
        })
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

// ── Invoice Adjustments (Glosas) ────────────────────────────────────────────

pub async fn list_invoice_adjustments(
    _user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<InvoiceAdjustmentWithItemsDto>>, (StatusCode, String)> {
    state
        .invoice_adjustment_service
        .list_adjustments(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}

pub async fn create_invoice_adjustment(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateInvoiceAdjustmentPayload>,
) -> Result<(StatusCode, Json<InvoiceAdjustmentWithItemsDto>), (StatusCode, String)> {
    state
        .invoice_adjustment_service
        .create_adjustment(id, payload, user.id)
        .await
        .map(|a| (StatusCode::CREATED, Json(a)))
        .map_err(|e| (StatusCode::from(&e), e.to_string()))
}
