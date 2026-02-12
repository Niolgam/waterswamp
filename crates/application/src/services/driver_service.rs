use crate::errors::ServiceError;
use crate::services::supplier_service::{normalize_document, validate_cpf};
use domain::{
    models::driver::*,
    ports::driver::*,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct DriverService {
    driver_repo: Arc<dyn DriverRepositoryPort>,
}

impl DriverService {
    pub fn new(driver_repo: Arc<dyn DriverRepositoryPort>) -> Self {
        Self { driver_repo }
    }

    pub async fn create_driver(
        &self,
        payload: CreateDriverPayload,
        created_by: Option<Uuid>,
    ) -> Result<DriverDto, ServiceError> {
        // Normalize and validate CPF
        let cpf = normalize_document(&payload.cpf);
        validate_cpf(&cpf).map_err(ServiceError::BadRequest)?;

        // Check CPF uniqueness
        if self.driver_repo.exists_by_cpf(&cpf).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!(
                "Motorista com CPF '{}' já existe", cpf
            )));
        }

        // Check CNH uniqueness
        let cnh = payload.cnh_number.trim().to_string();
        if cnh.is_empty() {
            return Err(ServiceError::BadRequest("Número da CNH é obrigatório".to_string()));
        }
        if self.driver_repo.exists_by_cnh(&cnh).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!(
                "Motorista com CNH '{}' já existe", cnh
            )));
        }

        // Validate CNH category
        let category = payload.cnh_category.trim().to_uppercase();
        validate_cnh_category(&category)?;

        self.driver_repo
            .create(
                &payload.driver_type,
                payload.full_name.trim(),
                &cpf,
                &cnh,
                &category,
                payload.cnh_expiration,
                payload.phone.as_deref(),
                payload.email.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_driver(&self, id: Uuid) -> Result<DriverDto, ServiceError> {
        self.driver_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Motorista não encontrado".to_string()))
    }

    pub async fn update_driver(
        &self,
        id: Uuid,
        payload: UpdateDriverPayload,
        updated_by: Option<Uuid>,
    ) -> Result<DriverDto, ServiceError> {
        let _current = self.driver_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Motorista não encontrado".to_string()))?;

        // Validate and normalize CPF if being changed
        let normalized_cpf = if let Some(ref cpf_raw) = payload.cpf {
            let cpf = normalize_document(cpf_raw);
            validate_cpf(&cpf).map_err(ServiceError::BadRequest)?;
            if self.driver_repo.exists_by_cpf_excluding(&cpf, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!(
                    "Motorista com CPF '{}' já existe", cpf
                )));
            }
            Some(cpf)
        } else {
            None
        };

        // Validate CNH if being changed
        let normalized_cnh = if let Some(ref cnh_raw) = payload.cnh_number {
            let cnh = cnh_raw.trim().to_string();
            if cnh.is_empty() {
                return Err(ServiceError::BadRequest("Número da CNH é obrigatório".to_string()));
            }
            if self.driver_repo.exists_by_cnh_excluding(&cnh, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!(
                    "Motorista com CNH '{}' já existe", cnh
                )));
            }
            Some(cnh)
        } else {
            None
        };

        // Validate CNH category if being changed
        let normalized_category = if let Some(ref cat) = payload.cnh_category {
            let c = cat.trim().to_uppercase();
            validate_cnh_category(&c)?;
            Some(c)
        } else {
            None
        };

        self.driver_repo
            .update(
                id,
                payload.driver_type.as_ref(),
                payload.full_name.as_deref(),
                normalized_cpf.as_deref(),
                normalized_cnh.as_deref(),
                normalized_category.as_deref(),
                payload.cnh_expiration,
                payload.phone.as_deref(),
                payload.email.as_deref(),
                payload.is_active,
                updated_by,
            )
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_driver(&self, id: Uuid) -> Result<bool, ServiceError> {
        let _ = self.driver_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Motorista não encontrado".to_string()))?;
        self.driver_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_drivers(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        driver_type: Option<DriverType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<DriverDto>, i64), ServiceError> {
        self.driver_repo
            .list(limit, offset, search, driver_type, is_active)
            .await
            .map_err(ServiceError::from)
    }
}

fn validate_cnh_category(category: &str) -> Result<(), ServiceError> {
    const VALID: &[&str] = &["A", "B", "C", "D", "E", "AB", "AC", "AD", "AE"];
    if !VALID.contains(&category) {
        return Err(ServiceError::BadRequest(format!(
            "Categoria CNH '{}' inválida. Valores válidos: A, B, C, D, E, AB, AC, AD, AE",
            category
        )));
    }
    Ok(())
}
