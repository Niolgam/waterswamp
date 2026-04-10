use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Enums
// ============================

/// Status calculado dinamicamente pela comparação entre quantidade original e ajustada
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdjustmentItemStatus {
    /// Sem ajuste aplicado
    Regular,
    /// Ajuste parcial (adjusted_quantity < quantity_base)
    PartialAdjustment,
    /// Ajuste total (adjusted_quantity >= quantity_base)
    TotalAdjustment,
}

impl AdjustmentItemStatus {
    /// Calcula o status comparando a quantidade ajustada acumulada com a quantidade original
    pub fn calculate(adjusted_quantity: Decimal, original_quantity: Decimal) -> Self {
        if adjusted_quantity <= Decimal::ZERO {
            AdjustmentItemStatus::Regular
        } else if adjusted_quantity >= original_quantity {
            AdjustmentItemStatus::TotalAdjustment
        } else {
            AdjustmentItemStatus::PartialAdjustment
        }
    }
}

// ============================
// DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InvoiceAdjustmentDto {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub reason: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct InvoiceAdjustmentItemDto {
    pub id: Uuid,
    pub adjustment_id: Uuid,
    pub invoice_item_id: Uuid,
    pub adjusted_quantity: Decimal,
    pub adjusted_value: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Ajuste com nome do item do catálogo e status calculado
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvoiceAdjustmentWithItemsDto {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub reason: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub items: Vec<InvoiceAdjustmentItemDetailDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvoiceAdjustmentItemDetailDto {
    pub id: Uuid,
    pub adjustment_id: Uuid,
    pub invoice_item_id: Uuid,
    pub catalog_item_name: Option<String>,
    pub adjusted_quantity: Decimal,
    pub adjusted_value: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================
// Request Payloads
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInvoiceAdjustmentItemPayload {
    pub invoice_item_id: Uuid,
    /// Quantidade física devolvida/recusada (0 se apenas financeiro)
    pub adjusted_quantity: Option<Decimal>,
    /// Desconto financeiro (glosa) em valor absoluto
    pub adjusted_value: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInvoiceAdjustmentPayload {
    pub reason: String,
    pub items: Vec<CreateInvoiceAdjustmentItemPayload>,
}
