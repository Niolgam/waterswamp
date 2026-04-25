use crate::errors::RepositoryError;
use crate::models::odometer::*;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

// ============================
// Odometer Repository Port
// ============================

#[async_trait]
pub trait OdometerReadingRepositoryPort: Send + Sync {
    /// Registra uma nova leitura. `request_id` garante idempotência — se já existir,
    /// retorna o registro original sem inserir novamente.
    async fn create(
        &self,
        veiculo_id: Uuid,
        valor_km: Decimal,
        fonte: FonteLeitura,
        referencia_id: Option<Uuid>,
        coletado_em: DateTime<Utc>,
        status: StatusLeitura,
        motivo_quarentena: Option<&str>,
        request_id: Uuid,
        created_by: Option<Uuid>,
    ) -> Result<OdometerReadingDto, RepositoryError>;

    /// Busca leitura por ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OdometerReadingDto>, RepositoryError>;

    /// Busca leitura por `request_id` (idempotência).
    async fn find_by_request_id(
        &self,
        request_id: Uuid,
    ) -> Result<Option<OdometerReadingDto>, RepositoryError>;

    /// Lista leituras de um veículo (ordem decrescente de coletado_em).
    async fn list_by_vehicle(
        &self,
        veiculo_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<StatusLeitura>,
    ) -> Result<(Vec<OdometerReadingDto>, i64), RepositoryError>;

    /// Retorna o `Odômetro_Projetado`: maior `valor_km` com `status = VALIDADO`.
    async fn get_projection(
        &self,
        veiculo_id: Uuid,
    ) -> Result<OdometerProjectionDto, RepositoryError>;

    /// Resolve uma leitura em quarentena: promove para VALIDADO ou REJEITADO (RF-INS-03).
    /// Usa OCC via `version`.
    async fn resolve_quarantine(
        &self,
        id: Uuid,
        novo_status: StatusLeitura,
        motivo: Option<&str>,
        version: i32,
    ) -> Result<OdometerReadingDto, RepositoryError>;
}

// ============================
// Idempotency Key Repository Port
// ============================

#[async_trait]
pub trait IdempotencyKeyRepositoryPort: Send + Sync {
    /// Busca entrada não-expirada pelo `request_id`.
    async fn find_active(
        &self,
        request_id: Uuid,
        endpoint: &str,
    ) -> Result<Option<IdempotencyKeyDto>, RepositoryError>;

    /// Persiste o resultado de uma operação para deduplicação de retries.
    async fn store(
        &self,
        request_id: Uuid,
        endpoint: &str,
        response_status: i32,
        response_body: Option<serde_json::Value>,
    ) -> Result<IdempotencyKeyDto, RepositoryError>;

    /// Remove entradas expiradas (TTL > 24h). Chamado por job de limpeza periódico.
    async fn delete_expired(&self) -> Result<u64, RepositoryError>;
}
