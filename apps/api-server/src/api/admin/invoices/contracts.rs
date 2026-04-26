use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use domain::models::invoice::{
    CancelInvoicePayload, CheckInvoicePayload, CompensatoryReversalPayload,
    CreateInvoiceItemPayload, CreateInvoicePayload, InvoiceItemWithDetailsDto, InvoiceStatus,
    InvoiceWithDetailsDto, PostInvoicePayload, RejectInvoicePayload, StartCheckingPayload,
    UpdateInvoicePayload,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvoicesListResponse {
    pub data: Vec<InvoiceWithDetailsDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvoiceItemsListResponse {
    pub items: Vec<InvoiceItemWithDetailsDto>,
}
