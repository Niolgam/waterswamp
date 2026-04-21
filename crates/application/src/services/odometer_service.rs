use crate::errors::ServiceError;
use chrono::{DateTime, Utc};
use domain::{
    models::odometer::*,
    ports::odometer::*,
    ports::vehicle::VehicleRepositoryPort,
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

/// Velocidade máxima plausível para detectar saltos irreais (DRS 4.3.3).
const MAX_PLAUSIBLE_SPEED_KMH: f64 = 200.0;

pub struct OdometerService {
    odometer_repo: Arc<dyn OdometerReadingRepositoryPort>,
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
}

impl OdometerService {
    pub fn new(
        odometer_repo: Arc<dyn OdometerReadingRepositoryPort>,
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    ) -> Self {
        Self { odometer_repo, vehicle_repo }
    }

    /// Registra uma leitura de odômetro aplicando as regras de validação (DRS 4.3.3).
    ///
    /// Se `request_id` já existir na tabela, retorna o registro original sem reprocessar.
    pub async fn register_reading(
        &self,
        payload: CreateOdometerReadingPayload,
        request_id: Uuid,
        created_by: Option<Uuid>,
    ) -> Result<OdometerReadingDto, ServiceError> {
        // Idempotência: retorna leitura existente sem reprocessar
        if let Some(existing) = self.odometer_repo
            .find_by_request_id(request_id)
            .await
            .map_err(ServiceError::from)?
        {
            return Ok(existing);
        }

        // Veículo existe?
        self.vehicle_repo
            .find_by_id(payload.veiculo_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if payload.valor_km < Decimal::ZERO {
            return Err(ServiceError::BadRequest("valor_km não pode ser negativo".to_string()));
        }

        // Odômetro projetado atual
        let projection = self.odometer_repo
            .get_projection(payload.veiculo_id)
            .await
            .map_err(ServiceError::from)?;

        let (status, motivo) = self.classify_reading(
            payload.valor_km,
            payload.coletado_em,
            &projection,
        ).await?;

        let reading = self.odometer_repo
            .create(
                payload.veiculo_id,
                payload.valor_km,
                payload.fonte,
                payload.referencia_id,
                payload.coletado_em,
                status,
                motivo.as_deref().or(payload.motivo.as_deref()),
                request_id,
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        Ok(reading)
    }

    /// Lista leituras de um veículo.
    pub async fn list_readings(
        &self,
        veiculo_id: Uuid,
        limit: i64,
        offset: i64,
        status: Option<StatusLeitura>,
    ) -> Result<(Vec<OdometerReadingDto>, i64), ServiceError> {
        // Veículo existe?
        self.vehicle_repo
            .find_by_id(veiculo_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.odometer_repo
            .list_by_vehicle(veiculo_id, limit, offset, status)
            .await
            .map_err(ServiceError::from)
    }

    /// Retorna o `Odômetro_Projetado` de um veículo.
    pub async fn get_projection(
        &self,
        veiculo_id: Uuid,
    ) -> Result<OdometerProjectionDto, ServiceError> {
        self.vehicle_repo
            .find_by_id(veiculo_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.odometer_repo
            .get_projection(veiculo_id)
            .await
            .map_err(ServiceError::from)
    }

    /// Resolve uma leitura em quarentena: valida ou rejeita (RF-INS-03 / RN16).
    pub async fn resolve_quarantine(
        &self,
        reading_id: Uuid,
        payload: ResolveQuarantinePayload,
    ) -> Result<OdometerReadingDto, ServiceError> {
        let reading = self.odometer_repo
            .find_by_id(reading_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound("Leitura não encontrada".to_string()))?;

        if reading.status != StatusLeitura::Quarentena {
            return Err(ServiceError::BadRequest(
                "Apenas leituras em QUARENTENA podem ser resolvidas".to_string(),
            ));
        }

        let novo_status = if payload.validar {
            StatusLeitura::Validado
        } else {
            StatusLeitura::Rejeitado
        };

        self.odometer_repo
            .resolve_quarantine(reading_id, novo_status, payload.motivo.as_deref(), payload.version)
            .await
            .map_err(ServiceError::from)
    }

    // ── Validação (DRS 4.3.3) ───────────────────────────────────────────────

    async fn classify_reading(
        &self,
        valor_km: Decimal,
        coletado_em: DateTime<Utc>,
        projection: &OdometerProjectionDto,
    ) -> Result<(StatusLeitura, Option<String>), ServiceError> {
        let Some(projetado) = projection.odometro_projetado_km else {
            // Primeira leitura do veículo — sempre validada
            return Ok((StatusLeitura::Validado, None));
        };

        // Regra 1/2: regressão de odômetro
        if valor_km <= projetado {
            return Ok((
                StatusLeitura::Quarentena,
                Some(format!(
                    "Regressão detectada: nova leitura {:.1} km ≤ projetado {:.1} km",
                    valor_km, projetado
                )),
            ));
        }

        // Regra 3: salto irreal (> 200 km/h para o intervalo de tempo)
        if let Some(ultima_em) = projection.ultima_leitura_validada_em {
            let delta_km = (valor_km - projetado)
                .to_f64()
                .unwrap_or(0.0);
            let delta_h = (coletado_em - ultima_em).num_seconds() as f64 / 3600.0;
            if delta_h > 0.0 {
                let velocidade = delta_km / delta_h;
                if velocidade > MAX_PLAUSIBLE_SPEED_KMH {
                    return Ok((
                        StatusLeitura::Quarentena,
                        Some(format!(
                            "Salto irreal detectado: {:.1} km/h excede limite de {:.0} km/h",
                            velocidade, MAX_PLAUSIBLE_SPEED_KMH
                        )),
                    ));
                }
            }
        }

        Ok((StatusLeitura::Validado, None))
    }
}
