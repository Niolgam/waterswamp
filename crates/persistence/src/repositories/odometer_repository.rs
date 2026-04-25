use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::odometer::*,
    ports::odometer::*,
};
use rust_decimal::Decimal;
use uuid::Uuid;
use sqlx::PgPool;

use crate::db_utils::map_db_error;

// ============================
// Odometer Reading Repository
// ============================

pub struct OdometerReadingRepository {
    pool: PgPool,
}

impl OdometerReadingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OdometerReadingRepositoryPort for OdometerReadingRepository {
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
    ) -> Result<OdometerReadingDto, RepositoryError> {
        // ON CONFLICT DO NOTHING + RETURNING: se já existe com este request_id,
        // retorna None; o caller busca pelo request_id para idempotência.
        let existing = self.find_by_request_id(request_id).await?;
        if let Some(reading) = existing {
            return Ok(reading);
        }

        sqlx::query_as::<_, OdometerReadingDto>(
            r#"
            INSERT INTO leituras_hodometro
                (veiculo_id, valor_km, fonte, referencia_id, coletado_em,
                 status, motivo_quarentena, request_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(veiculo_id)
        .bind(valor_km)
        .bind(fonte)
        .bind(referencia_id)
        .bind(coletado_em)
        .bind(status)
        .bind(motivo_quarentena)
        .bind(request_id)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OdometerReadingDto>, RepositoryError> {
        sqlx::query_as::<_, OdometerReadingDto>(
            "SELECT * FROM leituras_hodometro WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_by_request_id(
        &self,
        request_id: Uuid,
    ) -> Result<Option<OdometerReadingDto>, RepositoryError> {
        sqlx::query_as::<_, OdometerReadingDto>(
            "SELECT * FROM leituras_hodometro WHERE request_id = $1",
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_vehicle(
        &self,
        veiculo_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<StatusLeitura>,
    ) -> Result<(Vec<OdometerReadingDto>, i64), RepositoryError> {
        let rows = sqlx::query_as::<_, OdometerReadingDto>(
            r#"
            SELECT * FROM leituras_hodometro
            WHERE veiculo_id = $1
              AND ($4::leituras_hodometro_status_enum IS NULL OR status = $4)
            ORDER BY coletado_em DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(veiculo_id)
        .bind(limit)
        .bind(offset)
        .bind(&status)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM leituras_hodometro
            WHERE veiculo_id = $1
              AND ($2::leituras_hodometro_status_enum IS NULL OR status = $2)
            "#,
        )
        .bind(veiculo_id)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((rows, total))
    }

    async fn get_projection(
        &self,
        veiculo_id: Uuid,
    ) -> Result<OdometerProjectionDto, RepositoryError> {
        let projetado: Option<(Option<Decimal>, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT MAX(valor_km), MAX(coletado_em)
            FROM leituras_hodometro
            WHERE veiculo_id = $1 AND status = 'VALIDADO'
            "#,
        )
        .bind(veiculo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        let quarentena: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM leituras_hodometro WHERE veiculo_id = $1 AND status = 'QUARENTENA'",
        )
        .bind(veiculo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let (projetado_km, ultima_em) = projetado.unwrap_or((None, None));

        Ok(OdometerProjectionDto {
            veiculo_id,
            odometro_projetado_km: projetado_km,
            ultima_leitura_validada_em: ultima_em,
            leituras_em_quarentena: quarentena,
        })
    }

    async fn resolve_quarantine(
        &self,
        id: Uuid,
        novo_status: StatusLeitura,
        motivo: Option<&str>,
        version: i32,
    ) -> Result<OdometerReadingDto, RepositoryError> {
        let result = sqlx::query_as::<_, OdometerReadingDto>(
            r#"
            UPDATE leituras_hodometro
            SET status = $2,
                motivo_quarentena = COALESCE($3, motivo_quarentena),
                version = version + 1
            WHERE id = $1
              AND status = 'QUARENTENA'
              AND version = $4
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(novo_status)
        .bind(motivo)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        result.ok_or_else(|| {
            RepositoryError::OptimisticLockConflict(format!("odometer_reading:{}", id))
        })
    }
}

// ============================
// Idempotency Key Repository
// ============================

pub struct IdempotencyKeyRepository {
    pool: PgPool,
}

impl IdempotencyKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IdempotencyKeyRepositoryPort for IdempotencyKeyRepository {
    async fn find_active(
        &self,
        request_id: Uuid,
        endpoint: &str,
    ) -> Result<Option<IdempotencyKeyDto>, RepositoryError> {
        sqlx::query_as::<_, IdempotencyKeyDto>(
            r#"
            SELECT * FROM idempotency_keys
            WHERE request_id = $1
              AND endpoint = $2
              AND expires_at > NOW()
            "#,
        )
        .bind(request_id)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn store(
        &self,
        request_id: Uuid,
        endpoint: &str,
        response_status: i32,
        response_body: Option<serde_json::Value>,
    ) -> Result<IdempotencyKeyDto, RepositoryError> {
        sqlx::query_as::<_, IdempotencyKeyDto>(
            r#"
            INSERT INTO idempotency_keys (request_id, endpoint, response_status, response_body)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (request_id) DO UPDATE
              SET response_status = EXCLUDED.response_status,
                  response_body   = EXCLUDED.response_body
            RETURNING *
            "#,
        )
        .bind(request_id)
        .bind(endpoint)
        .bind(response_status)
        .bind(response_body)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete_expired(&self) -> Result<u64, RepositoryError> {
        let result = sqlx::query("DELETE FROM idempotency_keys WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected())
    }
}
