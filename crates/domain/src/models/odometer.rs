use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Origem da leitura de odômetro — define hierarquia de confiança (DRS 4.3.2).
/// Peso 1 (maior confiança) → Peso 5 (menor confiança).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "leituras_hodometro_fonte_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FonteLeitura {
    /// Peso 1 — check-in validado pelo Gestor de Frota
    CheckinGestor,
    /// Peso 2 — check-in preenchido pelo condutor no PWA
    CheckinCondutor,
    /// Peso 3 — check-out do condutor
    CheckoutCondutor,
    /// Peso 4 — leitura de planilha do fornecedor contratado
    AbastecimentoImportacao,
    /// Peso 5 — registro manual pelo condutor
    AbastecimentoManual,
}

impl FonteLeitura {
    /// Peso de confiança: menor número = maior confiança (DRS 4.3.2).
    pub fn peso(&self) -> u8 {
        match self {
            FonteLeitura::CheckinGestor => 1,
            FonteLeitura::CheckinCondutor => 2,
            FonteLeitura::CheckoutCondutor => 3,
            FonteLeitura::AbastecimentoImportacao => 4,
            FonteLeitura::AbastecimentoManual => 5,
        }
    }
}

/// Status de validação da leitura de odômetro (DRS 4.3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "leituras_hodometro_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusLeitura {
    /// Leitura aceita e refletida no `Odômetro_Projetado`.
    Validado,
    /// Leitura aceita mas não efetivada — aguarda revisão manual pelo Gestor (RF-INS-03).
    Quarentena,
    /// Leitura descartada após revisão manual.
    Rejeitado,
}

/// Registro imutável na série temporal do odômetro de um veículo (DRS 4.3.1).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct OdometerReadingDto {
    pub id: Uuid,
    pub veiculo_id: Uuid,
    pub valor_km: Decimal,
    pub fonte: FonteLeitura,
    pub referencia_id: Option<Uuid>,
    pub coletado_em: DateTime<Utc>,
    pub registrado_em: DateTime<Utc>,
    pub status: StatusLeitura,
    pub motivo_quarentena: Option<String>,
    pub request_id: Uuid,
    pub version: i32,
    pub created_by: Option<Uuid>,
}

/// Payload para registrar uma nova leitura de odômetro.
///
/// O cliente DEVE gerar um UUID v4 único para `request_id` (= `Idempotency-Key` do header).
/// Re-envios com o mesmo `request_id` retornam o resultado original sem reprocessar.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOdometerReadingPayload {
    pub veiculo_id: Uuid,
    pub valor_km: Decimal,
    pub fonte: FonteLeitura,
    pub referencia_id: Option<Uuid>,
    /// Momento real da coleta (pode ser retroativo).
    pub coletado_em: DateTime<Utc>,
    /// Motivo da leitura retroativa ou de quarentena esperada (opcional).
    pub motivo: Option<String>,
}

/// Payload para resolver uma leitura em quarentena (RF-INS-03 / RN16).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResolveQuarantinePayload {
    /// `true` = promover para VALIDADO; `false` = marcar como REJEITADO.
    pub validar: bool,
    pub motivo: Option<String>,
    pub version: i32,
}

/// Resumo do odômetro projetado de um veículo.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OdometerProjectionDto {
    pub veiculo_id: Uuid,
    /// Maior leitura VALIDADA (Odômetro_Projetado). `None` se não há leitura validada.
    pub odometro_projetado_km: Option<Decimal>,
    pub ultima_leitura_validada_em: Option<DateTime<Utc>>,
    pub leituras_em_quarentena: i64,
}

/// Entrada na tabela `idempotency_keys`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdempotencyKeyDto {
    pub id: Uuid,
    pub request_id: Uuid,
    pub endpoint: String,
    pub response_status: i32,
    pub response_body: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
