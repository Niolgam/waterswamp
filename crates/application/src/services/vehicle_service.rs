use crate::errors::ServiceError;
use domain::{
    models::vehicle::*,
    ports::vehicle::*,
};
use regex::Regex;
use std::sync::Arc;
use uuid::Uuid;

// ============================
// Brazilian Vehicle Validators
// ============================

/// Validates a Brazilian license plate (both old ABC-1234 and Mercosul ABC1D23 formats)
pub fn validate_license_plate(plate: &str) -> Result<(), String> {
    let plate = plate.to_uppercase().replace("-", "");
    if plate.len() != 7 {
        return Err("Placa deve ter 7 caracteres".to_string());
    }
    // Old format: ABC1234
    let old_format = Regex::new(r"^[A-Z]{3}[0-9]{4}$").unwrap();
    // Mercosul format: ABC1D23
    let mercosul_format = Regex::new(r"^[A-Z]{3}[0-9][A-Z][0-9]{2}$").unwrap();

    if !old_format.is_match(&plate) && !mercosul_format.is_match(&plate) {
        return Err("Formato de placa inválido. Use ABC1234 ou ABC1D23".to_string());
    }
    Ok(())
}

/// Validates a Brazilian chassis number (VIN - 17 characters, no I/O/Q)
pub fn validate_chassis_number(chassis: &str) -> Result<(), String> {
    let chassis = chassis.to_uppercase();
    if chassis.len() != 17 {
        return Err("Chassi deve ter exatamente 17 caracteres".to_string());
    }
    let valid = Regex::new(r"^[A-HJ-NPR-Z0-9]{17}$").unwrap();
    if !valid.is_match(&chassis) {
        return Err("Chassi contém caracteres inválidos (I, O e Q não são permitidos)".to_string());
    }
    Ok(())
}

/// Validates a Brazilian Renavam (11 digits with check digit)
pub fn validate_renavam(renavam: &str) -> Result<(), String> {
    let digits_only: String = renavam.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits_only.len() != 11 {
        return Err("Renavam deve ter exatamente 11 dígitos".to_string());
    }

    // Renavam check digit validation
    // Algorithm: multiply first 10 digits by weights [3,2,9,8,7,6,5,4,3,2] left-to-right,
    // sum products, multiply by 10, mod 11. If >= 10, check digit = 0.
    let digits: Vec<u32> = digits_only.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let weights = [3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let mut sum: u32 = 0;
    for i in 0..10 {
        sum += digits[i] * weights[i];
    }
    let check = (sum * 10) % 11;
    let check_digit = if check >= 10 { 0 } else { check };

    if check_digit != digits[10] {
        return Err("Dígito verificador do Renavam inválido".to_string());
    }
    Ok(())
}

/// Validates an engine number (alphanumeric, 5-30 chars)
pub fn validate_engine_number(engine: &str) -> Result<(), String> {
    if engine.is_empty() || engine.len() > 30 {
        return Err("Número do motor deve ter entre 1 e 30 caracteres".to_string());
    }
    let valid = Regex::new(r"^[A-Za-z0-9\-\.]+$").unwrap();
    if !valid.is_match(engine) {
        return Err("Número do motor contém caracteres inválidos".to_string());
    }
    Ok(())
}

/// Normalizes a license plate to uppercase without dashes
pub fn normalize_license_plate(plate: &str) -> String {
    plate.to_uppercase().replace("-", "")
}

// ============================
// Vehicle Service
// ============================

pub struct VehicleService {
    vehicle_repo: Arc<dyn VehicleRepositoryPort>,
    category_repo: Arc<dyn VehicleCategoryRepositoryPort>,
    make_repo: Arc<dyn VehicleMakeRepositoryPort>,
    model_repo: Arc<dyn VehicleModelRepositoryPort>,
    color_repo: Arc<dyn VehicleColorRepositoryPort>,
    fuel_type_repo: Arc<dyn VehicleFuelTypeRepositoryPort>,
    transmission_type_repo: Arc<dyn VehicleTransmissionTypeRepositoryPort>,
    document_repo: Arc<dyn VehicleDocumentRepositoryPort>,
    status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
}

impl VehicleService {
    pub fn new(
        vehicle_repo: Arc<dyn VehicleRepositoryPort>,
        category_repo: Arc<dyn VehicleCategoryRepositoryPort>,
        make_repo: Arc<dyn VehicleMakeRepositoryPort>,
        model_repo: Arc<dyn VehicleModelRepositoryPort>,
        color_repo: Arc<dyn VehicleColorRepositoryPort>,
        fuel_type_repo: Arc<dyn VehicleFuelTypeRepositoryPort>,
        transmission_type_repo: Arc<dyn VehicleTransmissionTypeRepositoryPort>,
        document_repo: Arc<dyn VehicleDocumentRepositoryPort>,
        status_history_repo: Arc<dyn VehicleStatusHistoryRepositoryPort>,
    ) -> Self {
        Self {
            vehicle_repo,
            category_repo,
            make_repo,
            model_repo,
            color_repo,
            fuel_type_repo,
            transmission_type_repo,
            document_repo,
            status_history_repo,
        }
    }

    // ============================
    // Vehicle Category Operations
    // ============================

    pub async fn create_vehicle_category(&self, payload: CreateVehicleCategoryPayload) -> Result<VehicleCategoryDto, ServiceError> {
        if self.category_repo.exists_by_name(&payload.name).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Categoria '{}' já existe", payload.name)));
        }
        self.category_repo
            .create(&payload.name, payload.description.as_deref(), payload.is_active.unwrap_or(true))
            .await
            .map_err(ServiceError::from)
    }

    pub async fn get_vehicle_category(&self, id: Uuid) -> Result<VehicleCategoryDto, ServiceError> {
        self.category_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Categoria de veículo não encontrada".to_string()))
    }

    pub async fn update_vehicle_category(&self, id: Uuid, payload: UpdateVehicleCategoryPayload) -> Result<VehicleCategoryDto, ServiceError> {
        let _ = self.get_vehicle_category(id).await?;
        if let Some(ref name) = payload.name {
            if self.category_repo.exists_by_name_excluding(name, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Categoria '{}' já existe", name)));
            }
        }
        self.category_repo
            .update(id, payload.name.as_deref(), payload.description.as_deref(), payload.is_active)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_category(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.category_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_categories(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleCategoryDto>, i64), ServiceError> {
        self.category_repo.list(limit, offset, search).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle Make Operations
    // ============================

    pub async fn create_vehicle_make(&self, payload: CreateVehicleMakePayload) -> Result<VehicleMakeDto, ServiceError> {
        if self.make_repo.exists_by_name(&payload.name).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Marca '{}' já existe", payload.name)));
        }
        self.make_repo.create(&payload.name).await.map_err(ServiceError::from)
    }

    pub async fn get_vehicle_make(&self, id: Uuid) -> Result<VehicleMakeDto, ServiceError> {
        self.make_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Marca de veículo não encontrada".to_string()))
    }

    pub async fn update_vehicle_make(&self, id: Uuid, payload: UpdateVehicleMakePayload) -> Result<VehicleMakeDto, ServiceError> {
        let _ = self.get_vehicle_make(id).await?;
        if let Some(ref name) = payload.name {
            if self.make_repo.exists_by_name_excluding(name, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Marca '{}' já existe", name)));
            }
        }
        self.make_repo.update(id, payload.name.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_make(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.make_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_makes(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleMakeDto>, i64), ServiceError> {
        self.make_repo.list(limit, offset, search).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle Model Operations
    // ============================

    pub async fn create_vehicle_model(&self, payload: CreateVehicleModelPayload) -> Result<VehicleModelDto, ServiceError> {
        let _ = self.get_vehicle_make(payload.make_id).await?;
        if self.model_repo.exists_by_name_in_make(&payload.name, payload.make_id).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Modelo '{}' já existe para esta marca", payload.name)));
        }
        self.model_repo.create(payload.make_id, &payload.name).await.map_err(ServiceError::from)
    }

    pub async fn get_vehicle_model(&self, id: Uuid) -> Result<VehicleModelDto, ServiceError> {
        self.model_repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Modelo de veículo não encontrado".to_string()))
    }

    pub async fn update_vehicle_model(&self, id: Uuid, payload: UpdateVehicleModelPayload) -> Result<VehicleModelDto, ServiceError> {
        let current = self.get_vehicle_model(id).await?;
        if let Some(ref name) = payload.name {
            if self.model_repo.exists_by_name_in_make_excluding(name, current.make_id, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Modelo '{}' já existe para esta marca", name)));
            }
        }
        self.model_repo.update(id, payload.name.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_model(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.model_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_models(&self, limit: i64, offset: i64, search: Option<String>, make_id: Option<Uuid>) -> Result<(Vec<VehicleModelDto>, i64), ServiceError> {
        self.model_repo.list(limit, offset, search, make_id).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle Color Operations
    // ============================

    pub async fn create_vehicle_color(&self, payload: CreateVehicleColorPayload) -> Result<VehicleColorDto, ServiceError> {
        if self.color_repo.exists_by_name(&payload.name).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Cor '{}' já existe", payload.name)));
        }
        self.color_repo.create(&payload.name, payload.hex_code.as_deref()).await.map_err(ServiceError::from)
    }

    pub async fn get_vehicle_color(&self, id: Uuid) -> Result<VehicleColorDto, ServiceError> {
        self.color_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Cor não encontrada".to_string()))
    }

    pub async fn update_vehicle_color(&self, id: Uuid, payload: UpdateVehicleColorPayload) -> Result<VehicleColorDto, ServiceError> {
        let _ = self.get_vehicle_color(id).await?;
        if let Some(ref name) = payload.name {
            if self.color_repo.exists_by_name_excluding(name, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Cor '{}' já existe", name)));
            }
        }
        self.color_repo.update(id, payload.name.as_deref(), payload.hex_code.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_color(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.color_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_colors(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleColorDto>, i64), ServiceError> {
        self.color_repo.list(limit, offset, search).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle Fuel Type Operations
    // ============================

    pub async fn create_vehicle_fuel_type(&self, payload: CreateVehicleFuelTypePayload) -> Result<VehicleFuelTypeDto, ServiceError> {
        if self.fuel_type_repo.exists_by_name(&payload.name).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Tipo de combustível '{}' já existe", payload.name)));
        }
        self.fuel_type_repo.create(&payload.name).await.map_err(ServiceError::from)
    }

    pub async fn get_vehicle_fuel_type(&self, id: Uuid) -> Result<VehicleFuelTypeDto, ServiceError> {
        self.fuel_type_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Tipo de combustível não encontrado".to_string()))
    }

    pub async fn update_vehicle_fuel_type(&self, id: Uuid, payload: UpdateVehicleFuelTypePayload) -> Result<VehicleFuelTypeDto, ServiceError> {
        let _ = self.get_vehicle_fuel_type(id).await?;
        if let Some(ref name) = payload.name {
            if self.fuel_type_repo.exists_by_name_excluding(name, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Tipo de combustível '{}' já existe", name)));
            }
        }
        self.fuel_type_repo.update(id, payload.name.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_fuel_type(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.fuel_type_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_fuel_types(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleFuelTypeDto>, i64), ServiceError> {
        self.fuel_type_repo.list(limit, offset, search).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle Transmission Type Operations
    // ============================

    pub async fn create_vehicle_transmission_type(&self, payload: CreateVehicleTransmissionTypePayload) -> Result<VehicleTransmissionTypeDto, ServiceError> {
        if self.transmission_type_repo.exists_by_name(&payload.name).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Tipo de câmbio '{}' já existe", payload.name)));
        }
        self.transmission_type_repo.create(&payload.name).await.map_err(ServiceError::from)
    }

    pub async fn get_vehicle_transmission_type(&self, id: Uuid) -> Result<VehicleTransmissionTypeDto, ServiceError> {
        self.transmission_type_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Tipo de câmbio não encontrado".to_string()))
    }

    pub async fn update_vehicle_transmission_type(&self, id: Uuid, payload: UpdateVehicleTransmissionTypePayload) -> Result<VehicleTransmissionTypeDto, ServiceError> {
        let _ = self.get_vehicle_transmission_type(id).await?;
        if let Some(ref name) = payload.name {
            if self.transmission_type_repo.exists_by_name_excluding(name, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Tipo de câmbio '{}' já existe", name)));
            }
        }
        self.transmission_type_repo.update(id, payload.name.as_deref(), payload.is_active).await.map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_transmission_type(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.transmission_type_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_transmission_types(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleTransmissionTypeDto>, i64), ServiceError> {
        self.transmission_type_repo.list(limit, offset, search).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle CRUD Operations
    // ============================

    pub async fn create_vehicle(&self, payload: CreateVehiclePayload, created_by: Option<Uuid>) -> Result<VehicleWithDetailsDto, ServiceError> {
        // Normalize and validate license plate
        let plate = normalize_license_plate(&payload.license_plate);
        validate_license_plate(&plate).map_err(ServiceError::BadRequest)?;

        // Validate chassis
        let chassis = payload.chassis_number.to_uppercase();
        validate_chassis_number(&chassis).map_err(ServiceError::BadRequest)?;

        // Validate Renavam
        validate_renavam(&payload.renavam).map_err(ServiceError::BadRequest)?;

        // Validate engine number if provided
        if let Some(ref engine) = payload.engine_number {
            validate_engine_number(engine).map_err(ServiceError::BadRequest)?;
        }

        // Check uniqueness
        if self.vehicle_repo.exists_by_license_plate(&plate).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Veículo com placa '{}' já existe", plate)));
        }
        if self.vehicle_repo.exists_by_chassis(&chassis).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Veículo com chassi '{}' já existe", chassis)));
        }
        if self.vehicle_repo.exists_by_renavam(&payload.renavam).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!("Veículo com Renavam '{}' já existe", payload.renavam)));
        }

        // Validate foreign keys exist
        let _ = self.category_repo.find_by_id(payload.category_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Categoria não encontrada".to_string()))?;
        let _ = self.make_repo.find_by_id(payload.make_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Marca não encontrada".to_string()))?;
        let _ = self.model_repo.find_by_id(payload.model_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Modelo não encontrado".to_string()))?;
        let _ = self.color_repo.find_by_id(payload.color_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Cor não encontrada".to_string()))?;
        let _ = self.fuel_type_repo.find_by_id(payload.fuel_type_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Tipo de combustível não encontrado".to_string()))?;
        if let Some(tt_id) = payload.transmission_type_id {
            let _ = self.transmission_type_repo.find_by_id(tt_id).await.map_err(ServiceError::from)?
                .ok_or(ServiceError::NotFound("Tipo de câmbio não encontrado".to_string()))?;
        }

        // Validate years
        if payload.manufacture_year < 1900 || payload.manufacture_year > 2100 {
            return Err(ServiceError::BadRequest("Ano de fabricação inválido".to_string()));
        }
        if payload.model_year < 1900 || payload.model_year > 2100 {
            return Err(ServiceError::BadRequest("Ano do modelo inválido".to_string()));
        }

        let status = payload.status.unwrap_or(VehicleStatus::Active);

        let vehicle = self.vehicle_repo
            .create(
                &plate,
                &chassis,
                &payload.renavam,
                payload.engine_number.as_deref(),
                payload.category_id,
                payload.make_id,
                payload.model_id,
                payload.color_id,
                payload.fuel_type_id,
                payload.transmission_type_id,
                payload.manufacture_year,
                payload.model_year,
                payload.passenger_capacity,
                payload.load_capacity_kg,
                payload.engine_displacement,
                payload.horsepower,
                payload.acquisition_type,
                payload.acquisition_date,
                payload.purchase_value,
                payload.patrimony_number.as_deref(),
                payload.department_id,
                status.clone(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        // Record initial status in history
        let _ = self.status_history_repo
            .create(vehicle.id, None, status, Some("Cadastro inicial"), created_by)
            .await;

        // Return with details
        self.vehicle_repo
            .find_with_details_by_id(vehicle.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar veículo criado".to_string()))
    }

    pub async fn get_vehicle(&self, id: Uuid) -> Result<VehicleWithDetailsDto, ServiceError> {
        self.vehicle_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Veículo não encontrado".to_string()))
    }

    pub async fn update_vehicle(&self, id: Uuid, payload: UpdateVehiclePayload, updated_by: Option<Uuid>) -> Result<VehicleWithDetailsDto, ServiceError> {
        let current = self.vehicle_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        // Validate and check uniqueness for plate
        let plate = payload.license_plate.as_ref().map(|p| normalize_license_plate(p));
        if let Some(ref p) = plate {
            validate_license_plate(p).map_err(ServiceError::BadRequest)?;
            if self.vehicle_repo.exists_by_license_plate_excluding(p, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Veículo com placa '{}' já existe", p)));
            }
        }

        // Validate chassis
        let chassis = payload.chassis_number.as_ref().map(|c| c.to_uppercase());
        if let Some(ref c) = chassis {
            validate_chassis_number(c).map_err(ServiceError::BadRequest)?;
            if self.vehicle_repo.exists_by_chassis_excluding(c, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Veículo com chassi '{}' já existe", c)));
            }
        }

        // Validate Renavam
        if let Some(ref renavam) = payload.renavam {
            validate_renavam(renavam).map_err(ServiceError::BadRequest)?;
            if self.vehicle_repo.exists_by_renavam_excluding(renavam, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!("Veículo com Renavam '{}' já existe", renavam)));
            }
        }

        // Validate engine number
        if let Some(ref engine) = payload.engine_number {
            validate_engine_number(engine).map_err(ServiceError::BadRequest)?;
        }

        // Track status change
        if let Some(ref new_status) = payload.status {
            if *new_status != current.status {
                let _ = self.status_history_repo
                    .create(id, Some(current.status.clone()), new_status.clone(), None, updated_by)
                    .await;
            }
        }

        let _ = self.vehicle_repo
            .update(
                id,
                plate.as_deref(),
                chassis.as_deref(),
                payload.renavam.as_deref(),
                payload.engine_number.as_deref(),
                payload.category_id,
                payload.make_id,
                payload.model_id,
                payload.color_id,
                payload.fuel_type_id,
                payload.transmission_type_id,
                payload.manufacture_year,
                payload.model_year,
                payload.passenger_capacity,
                payload.load_capacity_kg,
                payload.engine_displacement,
                payload.horsepower,
                payload.acquisition_type,
                payload.acquisition_date,
                payload.purchase_value,
                payload.patrimony_number.as_deref(),
                payload.department_id,
                payload.status,
                updated_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.vehicle_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar veículo atualizado".to_string()))
    }

    pub async fn delete_vehicle(&self, id: Uuid, deleted_by: Option<Uuid>) -> Result<bool, ServiceError> {
        let current = self.vehicle_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        // Record status change
        let _ = self.status_history_repo
            .create(id, Some(current.status), VehicleStatus::Inactive, Some("Veículo excluído (soft delete)"), deleted_by)
            .await;

        self.vehicle_repo.soft_delete(id, deleted_by).await.map_err(ServiceError::from)
    }

    pub async fn change_vehicle_status(&self, id: Uuid, payload: ChangeVehicleStatusPayload, changed_by: Option<Uuid>) -> Result<VehicleWithDetailsDto, ServiceError> {
        let current = self.vehicle_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        if current.status == payload.status {
            return Err(ServiceError::BadRequest("Veículo já está neste status".to_string()));
        }

        // Record in history
        let _ = self.status_history_repo
            .create(id, Some(current.status), payload.status.clone(), payload.reason.as_deref(), changed_by)
            .await;

        // Update status
        let _ = self.vehicle_repo
            .update(
                id, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None,
                Some(payload.status), changed_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.get_vehicle(id).await
    }

    pub async fn list_vehicles(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        status: Option<VehicleStatus>,
        category_id: Option<Uuid>,
        make_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        department_id: Option<Uuid>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleWithDetailsDto>, i64), ServiceError> {
        self.vehicle_repo
            .list(limit, offset, search, status, category_id, make_id, fuel_type_id, department_id, include_deleted)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn search_vehicles(&self, query: &str, limit: i64) -> Result<Vec<VehicleDto>, ServiceError> {
        self.vehicle_repo.search_autocomplete(query, limit).await.map_err(ServiceError::from)
    }

    pub async fn get_vehicle_status_history(&self, vehicle_id: Uuid) -> Result<Vec<VehicleStatusHistoryDto>, ServiceError> {
        // Verify vehicle exists
        let _ = self.vehicle_repo.find_by_id(vehicle_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Veículo não encontrado".to_string()))?;
        self.status_history_repo.list_by_vehicle(vehicle_id).await.map_err(ServiceError::from)
    }

    // ============================
    // Vehicle Document Operations
    // ============================

    pub async fn create_vehicle_document(
        &self,
        vehicle_id: Uuid,
        document_type: DocumentType,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        description: Option<&str>,
        uploaded_by: Option<Uuid>,
    ) -> Result<VehicleDocumentDto, ServiceError> {
        // Verify vehicle exists
        let _ = self.vehicle_repo.find_by_id(vehicle_id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Veículo não encontrado".to_string()))?;

        self.document_repo
            .create(vehicle_id, document_type, file_name, file_path, file_size, mime_type, description, uploaded_by)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_vehicle_document(&self, id: Uuid) -> Result<bool, ServiceError> {
        self.document_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_vehicle_documents(&self, vehicle_id: Uuid) -> Result<Vec<VehicleDocumentDto>, ServiceError> {
        self.document_repo.list_by_vehicle(vehicle_id).await.map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_license_plate_old_format() {
        assert!(validate_license_plate("ABC1234").is_ok());
        assert!(validate_license_plate("abc1234").is_ok());
        assert!(validate_license_plate("ABC-1234").is_ok());
    }

    #[test]
    fn test_validate_license_plate_mercosul() {
        assert!(validate_license_plate("ABC1D23").is_ok());
        assert!(validate_license_plate("abc1d23").is_ok());
    }

    #[test]
    fn test_validate_license_plate_invalid() {
        assert!(validate_license_plate("AB12345").is_err());
        assert!(validate_license_plate("ABCDEFG").is_err());
        assert!(validate_license_plate("1234567").is_err());
        assert!(validate_license_plate("ABC12").is_err());
        assert!(validate_license_plate("").is_err());
    }

    #[test]
    fn test_validate_chassis_number() {
        assert!(validate_chassis_number("9BWHE21JX24060831").is_ok());
        assert!(validate_chassis_number("1HGBH41JXMN109186").is_ok());
    }

    #[test]
    fn test_validate_chassis_number_invalid() {
        assert!(validate_chassis_number("ABCDE").is_err()); // Too short
        assert!(validate_chassis_number("9BWHE21JX24O60831").is_err()); // Contains O
        assert!(validate_chassis_number("9BWHE21JX24I60831").is_err()); // Contains I
    }

    #[test]
    fn test_validate_renavam_valid() {
        assert!(validate_renavam("00891749802").is_ok());
        assert!(validate_renavam("12345678900").is_ok());
        assert!(validate_renavam("99999999990").is_ok());
    }

    #[test]
    fn test_validate_renavam_invalid() {
        assert!(validate_renavam("12345678901").is_err()); // Invalid check digit
        assert!(validate_renavam("123").is_err()); // Too short
        assert!(validate_renavam("").is_err()); // Empty
    }

    #[test]
    fn test_validate_engine_number() {
        assert!(validate_engine_number("ABC123456").is_ok());
        assert!(validate_engine_number("789-XYZ.001").is_ok());
    }

    #[test]
    fn test_validate_engine_number_invalid() {
        assert!(validate_engine_number("").is_err());
        assert!(validate_engine_number("A".repeat(31).as_str()).is_err());
        assert!(validate_engine_number("AB C 123").is_err()); // spaces
    }

    #[test]
    fn test_normalize_license_plate() {
        assert_eq!(normalize_license_plate("abc-1234"), "ABC1234");
        assert_eq!(normalize_license_plate("ABC1D23"), "ABC1D23");
    }
}
