use crate::errors::ServiceError;
use domain::{
    models::fueling::*,
    ports::fueling::*,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct FuelingService {
    fueling_repo: Arc<dyn FuelingRepositoryPort>,
}

impl FuelingService {
    pub fn new(fueling_repo: Arc<dyn FuelingRepositoryPort>) -> Self {
        Self { fueling_repo }
    }

    pub async fn create_fueling(
        &self,
        payload: CreateFuelingPayload,
        created_by: Option<Uuid>,
    ) -> Result<FuelingWithDetailsDto, ServiceError> {
        // Validate positive values
        if payload.quantity_liters <= Decimal::ZERO {
            return Err(ServiceError::BadRequest("Quantidade de litros deve ser positiva".to_string()));
        }
        if payload.unit_price <= Decimal::ZERO {
            return Err(ServiceError::BadRequest("Preço unitário deve ser positivo".to_string()));
        }
        if payload.total_cost <= Decimal::ZERO {
            return Err(ServiceError::BadRequest("Custo total deve ser positivo".to_string()));
        }
        if payload.odometer_km < 0 {
            return Err(ServiceError::BadRequest("Quilometragem deve ser zero ou positiva".to_string()));
        }

        let fueling = self.fueling_repo
            .create(
                payload.vehicle_id,
                payload.driver_id,
                payload.supplier_id,
                payload.fuel_type_id,
                payload.fueling_date,
                payload.odometer_km,
                payload.quantity_liters,
                payload.unit_price,
                payload.total_cost,
                payload.notes.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.fueling_repo
            .find_with_details_by_id(fueling.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar abastecimento criado".to_string()))
    }

    pub async fn get_fueling(&self, id: Uuid) -> Result<FuelingWithDetailsDto, ServiceError> {
        self.fueling_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Abastecimento não encontrado".to_string()))
    }

    pub async fn update_fueling(
        &self,
        id: Uuid,
        payload: UpdateFuelingPayload,
        updated_by: Option<Uuid>,
    ) -> Result<FuelingWithDetailsDto, ServiceError> {
        let _current = self.fueling_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Abastecimento não encontrado".to_string()))?;

        // Validate positive values if provided
        if let Some(qty) = payload.quantity_liters {
            if qty <= Decimal::ZERO {
                return Err(ServiceError::BadRequest("Quantidade de litros deve ser positiva".to_string()));
            }
        }
        if let Some(price) = payload.unit_price {
            if price <= Decimal::ZERO {
                return Err(ServiceError::BadRequest("Preço unitário deve ser positivo".to_string()));
            }
        }
        if let Some(cost) = payload.total_cost {
            if cost <= Decimal::ZERO {
                return Err(ServiceError::BadRequest("Custo total deve ser positivo".to_string()));
            }
        }
        if let Some(km) = payload.odometer_km {
            if km < 0 {
                return Err(ServiceError::BadRequest("Quilometragem deve ser zero ou positiva".to_string()));
            }
        }

        let _ = self.fueling_repo
            .update(
                id,
                payload.vehicle_id,
                payload.driver_id,
                payload.supplier_id,
                payload.fuel_type_id,
                payload.fueling_date,
                payload.odometer_km,
                payload.quantity_liters,
                payload.unit_price,
                payload.total_cost,
                payload.notes.as_deref(),
                updated_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.fueling_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar abastecimento atualizado".to_string()))
    }

    pub async fn delete_fueling(&self, id: Uuid) -> Result<bool, ServiceError> {
        let _ = self.fueling_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Abastecimento não encontrado".to_string()))?;
        self.fueling_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_fuelings(
        &self,
        limit: i64,
        offset: i64,
        vehicle_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
    ) -> Result<(Vec<FuelingWithDetailsDto>, i64), ServiceError> {
        self.fueling_repo
            .list(limit, offset, vehicle_id, driver_id, supplier_id)
            .await
            .map_err(ServiceError::from)
    }
}
